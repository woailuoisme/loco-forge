use serde::Serialize;
use utoipa::ToSchema;

/// 文件上传响应 Payload
#[derive(Debug, Serialize, ToSchema)]
pub struct FileUploadResource {
    /// 存储系统中的唯一文件 Key
    pub key: String,
    /// 原始文件名
    pub filename: String,
    /// 文件字节大小
    pub size: usize,
    /// HTTP Content-Type 类型
    pub content_type: Option<String>,
}
