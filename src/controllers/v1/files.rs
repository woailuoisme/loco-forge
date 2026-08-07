use axum::{
    extract::{Multipart, Path, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::Response,
};
use loco_rs::prelude::*;
use serde::Serialize;
use std::path::Path as StdPath;
use uuid::Uuid;

use crate::{
    dtos::common::{ApiResponse, format_res},
    errors::AppError,
};

/// 文件上传响应 Payload
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct FileUploadResponse {
    /// 存储系统中的唯一文件 Key
    pub key: String,
    /// 原始文件名
    pub filename: String,
    /// 文件字节大小
    pub size: usize,
    /// HTTP Content-Type 类型
    pub content_type: Option<String>,
}

/// 单文件/多文件上传接口
#[utoipa::path(
    post,
    path = "/api/v1/files/upload",
    summary = "上传文件",
    description = "通过 multipart/form-data 格式上传单文件或多文件到存储服务",
    responses(
        (status = 200, description = "上传成功", body = ApiResponse<Vec<FileUploadResponse>>),
        (status = 400, description = "请求参数或文件格式错误"),
        (status = 401, description = "未授权")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "files"
)]
#[debug_handler]
pub async fn upload(_auth: auth::JWT, State(ctx): State<AppContext>, mut multipart: Multipart) -> Result<Response> {
    let mut uploaded_files = Vec::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        let filename = field.file_name().unwrap_or("unnamed_file").to_string();

        let content_type = field.content_type().map(|ct| ct.to_string());

        let bytes = match field.bytes().await {
            Ok(b) => b,
            Err(err) => {
                return Ok(AppError::BadRequest(format!("Failed to read file field: {err}")).into_response());
            }
        };

        let file_size = bytes.len();
        if file_size == 0 {
            continue;
        }

        // 生成存储 Key: uploads/<uuid>/<filename>
        let file_uuid = Uuid::new_v4();
        let key = format!("uploads/{file_uuid}/{filename}");
        let path = StdPath::new(&key);

        if let Err(err) = ctx.storage.upload(path, &bytes).await {
            tracing::error!(key = %key, error = %err, "Failed to upload file to storage");
            return Ok(AppError::internal(format!("Storage upload failed: {err}")).into_response());
        }

        uploaded_files.push(FileUploadResponse {
            key,
            filename,
            size: file_size,
            content_type,
        });
    }

    if uploaded_files.is_empty() {
        return Ok(AppError::BadRequest("No files were uploaded".to_string()).into_response());
    }

    format_res::ok(uploaded_files)
}

/// 获取文件内容/下载接口
#[utoipa::path(
    get,
    path = "/api/v1/files/{*key}",
    summary = "获取/下载文件",
    description = "根据文件 Key 获取文件内容",
    params(
        ("key" = String, Path, description = "文件相对路径 Key")
    ),
    responses(
        (status = 200, description = "获取成功"),
        (status = 404, description = "文件不存在")
    ),
    tag = "files"
)]
#[debug_handler]
pub async fn get_file(State(ctx): State<AppContext>, Path(key): Path<String>) -> Result<Response> {
    let path = StdPath::new(&key);
    let bytes: Vec<u8> = match ctx.storage.download(path).await {
        Ok(b) => b,
        Err(_) => {
            return Ok(AppError::NotFound(format!("File not found: {key}")).into_response());
        }
    };

    let mut headers = HeaderMap::new();
    let content_type = guess_content_type(&key);
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));

    Ok((StatusCode::OK, headers, bytes).into_response())
}

/// 简易 Content-Type 推导辅助函数
fn guess_content_type(filename: &str) -> &'static str {
    let lower = filename.to_lowercase();
    if lower.ends_with(".png") {
        "image/png"
    } else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        "image/jpeg"
    } else if lower.ends_with(".gif") {
        "image/gif"
    } else if lower.ends_with(".svg") {
        "image/svg+xml"
    } else if lower.ends_with(".json") {
        "application/json"
    } else if lower.ends_with(".pdf") {
        "application/pdf"
    } else if lower.ends_with(".txt") {
        "text/plain; charset=utf-8"
    } else {
        "application/octet-stream"
    }
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/v1/files")
        .add("/upload", post(upload))
        .add("/{*key}", get(get_file))
}
