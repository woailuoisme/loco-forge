//! # 高性能文件上传工具 (File Upload Utility)
//!
//! 提供支持单/多文件上传、防 OOM 内存溢出流式读取、扩展名与 MIME 白名单校验及灵活命名/路径策略的文件上传工具。

use axum::extract::multipart::Field;
use chrono::Utc;
use loco_rs::prelude::*;
use sha2::{Digest, Sha256};
use std::{borrow::Cow, collections::HashSet, path::Path as StdPath};
use uuid::Uuid;

use crate::common::consts::storage::{DEFAULT_MAX_FILE_SIZE, DEFAULT_MAX_TOTAL_UPLOAD_SIZE};

/// 文件命名策略枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NamingStrategy {
    /// UUID 随机独立文件名: `{prefix}/{yyyy}/{mm}/{dd}/{uuid}.{ext}`
    #[default]
    Uuid,
    /// 日期与 UUID 组合文件名: `{prefix}/{yyyyMMdd}_{uuid}/{filename}`
    DateUuid,
    /// SHA-256 内容哈希文件名 (自动去重): `{prefix}/{hash_prefix}/{full_hash}.{ext}`
    Hash,
    /// 原始文件名 (防穿越处理后): `{prefix}/{yyyy}/{mm}/{dd}/{sanitized_filename}`
    Original,
}

/// 上传配置选项 Builder
#[derive(Debug, Clone)]
pub struct UploadConfig {
    /// 单文件字节上限 (Byte)
    pub max_file_size: usize,
    /// 单次上传总字节上限 (Byte)
    pub max_total_size: usize,
    /// 业务前缀目录 (如 "avatars", "documents")
    pub prefix: String,
    /// 允许的文件扩展名白名单（全小写集合，高效 O(1) 校验）
    pub allowed_extensions: Option<HashSet<String>>,
    /// 允许的 MIME Content-Type 前缀白名单（如 `["image/", "application/pdf"]`）
    pub allowed_mime_prefixes: Option<Vec<String>>,
    /// 命名策略
    pub naming_strategy: NamingStrategy,
}

impl Default for UploadConfig {
    fn default() -> Self {
        Self {
            max_file_size: DEFAULT_MAX_FILE_SIZE,
            max_total_size: DEFAULT_MAX_TOTAL_UPLOAD_SIZE,
            prefix: "uploads".to_string(),
            allowed_extensions: None,
            allowed_mime_prefixes: None,
            naming_strategy: NamingStrategy::Uuid,
        }
    }
}

impl UploadConfig {
    /// 创建新配置构建器
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置单文件最大字节数
    #[must_use]
    pub fn max_file_size(mut self, size: usize) -> Self {
        self.max_file_size = size;
        self
    }

    /// 设置单次上传总最大字节数
    #[must_use]
    pub fn max_total_size(mut self, size: usize) -> Self {
        self.max_total_size = size;
        self
    }

    /// 设置存储目录前缀
    #[must_use]
    pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = prefix.into();
        self
    }

    /// 设置扩展名白名单 (转换为小写 HashSet 提高查找效率)
    #[must_use]
    pub fn allowed_extensions(mut self, exts: &[&str]) -> Self {
        self.allowed_extensions = Some(exts.iter().map(|s| s.to_lowercase()).collect());
        self
    }

    /// 设置 MIME 前缀白名单 (例如 `&["image/", "application/pdf"]`)
    #[must_use]
    pub fn allowed_mime_prefixes(mut self, mimes: &[&str]) -> Self {
        self.allowed_mime_prefixes = Some(mimes.iter().map(|s| s.to_string()).collect());
        self
    }

    /// 设置命名策略
    #[must_use]
    pub fn naming_strategy(mut self, strategy: NamingStrategy) -> Self {
        self.naming_strategy = strategy;
        self
    }
}

/// 处理完成的单文件结果结构
#[derive(Debug, Clone)]
pub struct ProcessedUpload {
    /// 存储 Key (相对路径)
    pub key: String,
    /// 清理后的文件名
    pub filename: String,
    /// 文件实际字节数
    pub size: usize,
    /// MIME Content-Type
    pub content_type: Option<String>,
    /// 文件二进制数据
    pub bytes: Vec<u8>,
}

/// 高性能文件上传处理工具
pub struct UploadUtil;

impl UploadUtil {
    /// 安全清理文件名（使用 `Cow` 实现无路径分隔符时的零堆分配）
    pub fn sanitize_filename(raw_filename: Option<&str>) -> Cow<'_, str> {
        let name = raw_filename.unwrap_or("unnamed_file");
        if name.contains('\\') || name.contains('/') {
            let normalized = name.replace('\\', "/");
            let safe_name = StdPath::new(&normalized)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unnamed_file");
            Cow::Owned(safe_name.to_string())
        } else {
            Cow::Borrowed(name)
        }
    }

    /// 提取文件扩展名 (零堆分配引用切片)
    #[must_use]
    pub fn get_extension(filename: &str) -> &str {
        StdPath::new(filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
    }

    /// 校验文件扩展名与 MIME 类型
    pub fn validate_file(filename: &str, content_type: Option<&str>, config: &UploadConfig) -> Result<()> {
        let ext = Self::get_extension(filename);
        let ext_lowercase = ext.to_lowercase();

        // 校验扩展名白名单 (O(1) HashSet 查找)
        if let Some(ref allowed_exts) = config.allowed_extensions
            && !allowed_exts.contains(&ext_lowercase)
        {
            tracing::warn!(filename = %filename, ext = %ext, "File extension not allowed");
            return Err(Error::BadRequest(format!("File extension '.{ext}' is not allowed")));
        }

        // 校验 MIME 类型白名单
        if let Some(ref allowed_mimes) = config.allowed_mime_prefixes {
            let ct = content_type.unwrap_or("");
            let matched = allowed_mimes.iter().any(|prefix| ct.starts_with(prefix));
            if !matched {
                tracing::warn!(filename = %filename, content_type = ?content_type, "MIME type not allowed");
                return Err(Error::BadRequest(format!("Content-Type '{ct}' is not allowed")));
            }
        }

        Ok(())
    }

    /// 分块流式读取 Multipart Field 内容（初始容量预分配防内存多次重分配与爆满）
    pub async fn read_field_bounded(field: &mut Field<'_>, max_size: usize) -> Result<Vec<u8>> {
        // 预分配 8KB 初始缓冲区减少前期扩容重分配开销
        let mut buffer = Vec::with_capacity(8192);

        while let Some(chunk) = field.chunk().await.map_err(|err| {
            tracing::warn!(error = %err, "Error reading multipart chunk stream");
            Error::BadRequest(format!("Failed to read stream chunk: {err}"))
        })? {
            if buffer.len() + chunk.len() > max_size {
                tracing::warn!(
                    current_len = buffer.len(),
                    chunk_len = chunk.len(),
                    max_size = max_size,
                    "File exceeded maximum allowed byte limit"
                );
                return Err(Error::BadRequest(format!(
                    "File size exceeds limit of {max_size} bytes"
                )));
            }
            buffer.extend_from_slice(&chunk);
        }

        Ok(buffer)
    }

    /// 根据命名策略生成存储 Key
    pub fn generate_storage_key(filename: &str, bytes: &[u8], config: &UploadConfig) -> String {
        let now = Utc::now();
        let yyyy = now.format("%Y").to_string();
        let mm = now.format("%m").to_string();
        let dd = now.format("%d").to_string();
        let ext = Self::get_extension(filename);
        let ext_suffix = if ext.is_empty() {
            String::new()
        } else {
            format!(".{ext}")
        };

        let prefix = config.prefix.trim_matches('/');

        match config.naming_strategy {
            NamingStrategy::Uuid => {
                let uuid = Uuid::new_v4();
                format!("{prefix}/{yyyy}/{mm}/{dd}/{uuid}{ext_suffix}")
            }
            NamingStrategy::DateUuid => {
                let date_str = now.format("%Y%m%d_%H%M%S").to_string();
                let uuid = Uuid::new_v4();
                format!("{prefix}/{date_str}_{uuid}/{filename}")
            }
            NamingStrategy::Hash => {
                let hash_bytes = Sha256::digest(bytes);
                let hash_result = hash_bytes.iter().fold(String::with_capacity(64), |mut acc, byte| {
                    use std::fmt::Write;
                    let _ = write!(acc, "{byte:02x}");
                    acc
                });
                let hash_sub = &hash_result[..2];
                format!("{prefix}/{hash_sub}/{hash_result}{ext_suffix}")
            }
            NamingStrategy::Original => {
                format!("{prefix}/{yyyy}/{mm}/{dd}/{filename}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(UploadUtil::sanitize_filename(Some("../../etc/passwd")), "passwd");
        assert_eq!(
            UploadUtil::sanitize_filename(Some("C:\\Windows\\system32\\cmd.exe")),
            "cmd.exe"
        );
        assert_eq!(UploadUtil::sanitize_filename(None), "unnamed_file");
        // 零堆分配 Borrowed 检查
        if let Cow::Borrowed(b) = UploadUtil::sanitize_filename(Some("safe_photo.jpg")) {
            assert_eq!(b, "safe_photo.jpg");
        } else {
            panic!("Expected borrowed Cow for safe filename");
        }
    }

    #[test]
    fn test_get_extension() {
        assert_eq!(UploadUtil::get_extension("avatar.PNG"), "PNG");
        assert_eq!(UploadUtil::get_extension("archive.tar.gz"), "gz");
        assert_eq!(UploadUtil::get_extension("noext"), "");
    }

    #[test]
    fn test_validate_file_extensions() {
        let config = UploadConfig::new().allowed_extensions(&["png", "jpg", "pdf"]);
        assert!(UploadUtil::validate_file("image.png", None, &config).is_ok());
        assert!(UploadUtil::validate_file("image.JPG", None, &config).is_ok());
        assert!(UploadUtil::validate_file("script.sh", None, &config).is_err());
    }

    #[test]
    fn test_generate_storage_key_strategies() {
        let bytes = b"hello content";

        let cfg_uuid = UploadConfig::new()
            .prefix("avatars")
            .naming_strategy(NamingStrategy::Uuid);
        let key_uuid = UploadUtil::generate_storage_key("photo.jpg", bytes, &cfg_uuid);
        assert!(key_uuid.starts_with("avatars/"));
        assert!(key_uuid.ends_with(".jpg"));

        let cfg_hash = UploadConfig::new()
            .prefix("files")
            .naming_strategy(NamingStrategy::Hash);
        let key_hash = UploadUtil::generate_storage_key("document.pdf", bytes, &cfg_hash);
        assert!(key_hash.starts_with("files/"));
        assert!(key_hash.ends_with(".pdf"));
    }
}
