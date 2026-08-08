use axum::{
    extract::{Multipart, Path, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::Response,
};
use loco_rs::prelude::*;

use crate::{
    common::consts::openapi::TAG_FILES,
    dtos::{
        common::{ApiResponse, format_res},
        files::FileUploadResource,
    },
    services::FileService,
};

/// 单文件/多文件上传接口
#[utoipa::path(
    post,
    path = "/api/v1/files/upload",
    summary = "上传文件",
    description = "通过 multipart/form-data 格式上传单文件或多文件到存储服务",
    responses(
        (status = 200, description = "上传成功", body = ApiResponse<Vec<FileUploadResource>>),
        (status = 400, description = "请求参数或文件格式错误"),
        (status = 401, description = "未授权")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = TAG_FILES
)]
#[debug_handler]
pub async fn upload(_auth: auth::JWT, State(ctx): State<AppContext>, multipart: Multipart) -> Result<Response> {
    let uploaded_files = FileService::upload(&ctx, multipart).await?;
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
        (status = 400, description = "非法的路径 Key"),
        (status = 404, description = "文件不存在")
    ),
    tag = TAG_FILES
)]
#[debug_handler]
pub async fn get_file(State(ctx): State<AppContext>, Path(key): Path<String>) -> Result<Response> {
    let (content_type, bytes) = FileService::download(&ctx, &key).await?;

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));

    Ok((StatusCode::OK, headers, bytes).into_response())
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/v1/files")
        .add("/upload", post(upload))
        .add("/{*key}", get(get_file))
}
