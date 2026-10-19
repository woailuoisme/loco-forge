//! # 文件存储与管理服务 (File & Storage Service)

use std::path::Path as StdPath;

use axum::extract::Multipart;
use loco_rs::prelude::*;

use crate::{
    common::utils::upload::{UploadConfig, UploadUtil},
    dtos::files::FileUploadResource,
    errors::{AppError, AppResult},
};

/// 文件存储与管理服务
pub struct FileService;

impl FileService {
    /// 校验 key 的规范性，防御路径穿越攻击（零堆分配）
    #[must_use]
    pub fn sanitize_key(key: &str) -> Option<&str> {
        if key.contains('\\') {
            return None;
        }
        let path = StdPath::new(key);
        if path.is_absolute()
            || path.components().any(|c| {
                matches!(
                    c,
                    std::path::Component::ParentDir | std::path::Component::RootDir
                )
            })
        {
            return None;
        }
        Some(key)
    }

    /// 使用默认配置执行文件上传
    pub async fn upload(
        ctx: &AppContext,
        multipart: Multipart,
    ) -> AppResult<Vec<FileUploadResource>> {
        Self::upload_with_config(ctx, multipart, &UploadConfig::default()).await
    }

    /// 使用自定义配置（包含文件限制、命名策略、白名单校验、防 OOM 流式读取）执行上传
    pub async fn upload_with_config(
        ctx: &AppContext,
        mut multipart: Multipart,
        config: &UploadConfig,
    ) -> AppResult<Vec<FileUploadResource>> {
        let mut uploaded_files = Vec::new();
        let mut total_uploaded_bytes: usize = 0;

        while let Ok(Some(mut field)) = multipart.next_field().await {
            let raw_name_buf = field.file_name().map(ToString::to_string);
            let filename_cow = UploadUtil::sanitize_filename(raw_name_buf.as_deref());
            let content_type = field.content_type().map(ToString::to_string);

            // 1. 后缀与 MIME 白名单预检
            if let Err(err) =
                UploadUtil::validate_file(&filename_cow, content_type.as_deref(), config)
            {
                tracing::warn!(filename = %filename_cow, error = %err, "Upload file validation failed");
                return Err(AppError::bad_request(err.to_string()));
            }

            // 2. 防 OOM 分块流式读取（受限于 config.max_file_size）
            let bytes = UploadUtil::read_field_bounded(&mut field, config.max_file_size)
                .await
                .map_err(|e| AppError::bad_request(e.to_string()))?;
            let file_size = bytes.len();
            if file_size == 0 {
                tracing::debug!(filename = %filename_cow, "Skipping zero-byte file upload");
                continue;
            }

            // 3. 单次 Multipart 总大小拦截
            total_uploaded_bytes += file_size;
            if total_uploaded_bytes > config.max_total_size {
                tracing::warn!(
                    total_uploaded_bytes = total_uploaded_bytes,
                    max_total_size = config.max_total_size,
                    "Total multipart upload size exceeded"
                );
                return Err(AppError::bad_request(format!(
                    "Total upload size exceeds limit of {} bytes",
                    config.max_total_size
                )));
            }

            // 4. 根据策略生成 Storage Key
            let key = UploadUtil::generate_storage_key(&filename_cow, &bytes, config);
            let path = StdPath::new(&key);

            // 5. 写入存储层
            let content_bytes = bytes::Bytes::from(bytes);
            if let Err(err) = ctx.storage.upload(path, &content_bytes).await {
                tracing::error!(key = %key, error = %err, "Failed to write file to storage provider");
                return Err(AppError::internal(
                    "Failed to write file to storage provider",
                ));
            }

            tracing::info!(key = %key, size = file_size, "File uploaded successfully");

            uploaded_files.push(FileUploadResource {
                key,
                filename: filename_cow.into_owned(),
                size: file_size,
                content_type,
            });
        }

        if uploaded_files.is_empty() {
            return Err(AppError::bad_request("No valid files were uploaded"));
        }

        Ok(uploaded_files)
    }

    /// 获取文件二进制数据及由 mime_guess 自动推导的 RFC Content-Type
    pub async fn download(ctx: &AppContext, raw_key: &str) -> AppResult<(&'static str, Vec<u8>)> {
        let key = Self::sanitize_key(raw_key).ok_or_else(|| {
            tracing::warn!(key = %raw_key, "Detected potential path traversal attempt in file download");
            AppError::bad_request("Invalid file path key")
        })?;

        let path = StdPath::new(key);
        let bytes = ctx.storage.download(path).await.map_err(|err| {
            tracing::warn!(key = %key, error = %err, "File not found or unreadable from storage provider");
            AppError::not_found("File not found")
        })?;

        let content_type = mime_guess::from_path(key)
            .first_raw()
            .unwrap_or("application/octet-stream");

        Ok((content_type, bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_service_sanitize_key_valid() {
        assert_eq!(
            FileService::sanitize_key("uploads/2026/image.png"),
            Some("uploads/2026/image.png")
        );
        assert_eq!(
            FileService::sanitize_key("documents/contract.pdf"),
            Some("documents/contract.pdf")
        );
    }

    #[test]
    fn test_file_service_sanitize_key_prevents_path_traversal() {
        assert_eq!(FileService::sanitize_key("/etc/passwd"), None);
        assert_eq!(FileService::sanitize_key("uploads/../etc/passwd"), None);
        assert_eq!(FileService::sanitize_key("..\\Windows\\System32"), None);
    }
}
