use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::Response,
};
use loco_rs::prelude::*;

use crate::{
    common::consts::openapi::TAG_CREDENTIALS,
    dtos::{
        auth::*,
        common::{ApiError, ValidJson, format_res},
    },
    errors::AppResult,
    services::CredentialService,
};

// ─────────────────────────────────────────────────────────────────────────────
// Cookie 与 Token 辅助函数
// ─────────────────────────────────────────────────────────────────────────────

use super::shared::*;

// ─────────────────────────────────────────────────────────────────────────────
// API 密钥管理服务 (ApiKey Plugin Handlers)
// ─────────────────────────────────────────────────────────────────────────────

/// 创建新的 API Key
#[utoipa::path(
    post,
    path = "/v1/api-keys",
    summary = "创建 API 密钥",
    description = "创建密钥并返回明文",
    request_body = CreateApiKeyRequest,
    responses(
        (status = 200, description = "创建成功", body = ApiKeyCreatedResource),
        (status = 400, description = "请求参数错误", body = ApiError),
        (status = 401, description = "未登录", body = ApiError)
    ),
    tag = TAG_CREDENTIALS
)]
#[debug_handler]
pub async fn create_api_key(
    State(ctx): State<AppContext>,
    auth: AuthSession,
    ValidJson(params): ValidJson<CreateApiKeyRequest>,
) -> AppResult<Response> {
    let res = CredentialService::create_api_key(&ctx.db, &auth.user, &params).await?;
    Ok(format_res::ok(res)?)
}

/// 列出当前用户创建的全部 API Key
#[utoipa::path(
    get,
    path = "/v1/api-keys",
    summary = "API 密钥列表",
    description = "列出 API 密钥",
    responses(
        (status = 200, description = "获取成功", body = Vec<ApiKeyResource>),
        (status = 401, description = "未登录", body = ApiError)
    ),
    tag = TAG_CREDENTIALS
)]
#[debug_handler]
pub async fn list_api_keys(
    State(ctx): State<AppContext>,
    auth: AuthSession,
) -> AppResult<Response> {
    let res = CredentialService::list_api_keys(&ctx.db, &auth.user).await?;
    Ok(format_res::ok(res)?)
}

/// 校验 API 密钥有效性
#[utoipa::path(
    post,
    path = "/v1/api-key-checks",
    summary = "校验 API 密钥",
    description = "校验密钥并递增计数",
    request_body = VerifyApiKeyRequest,
    responses(
        (status = 200, description = "校验结果", body = VerifyApiKeyResource),
        (status = 400, description = "请求参数错误", body = ApiError)
    ),
    tag = TAG_CREDENTIALS
)]
#[debug_handler]
pub async fn verify_api_key(
    State(ctx): State<AppContext>,
    headers: HeaderMap,
    ValidJson(params): ValidJson<VerifyApiKeyRequest>,
) -> AppResult<Response> {
    let mut req_params = params;
    if req_params.key.is_empty()
        && let Some(hdr) = headers.get("x-api-key").and_then(|v| v.to_str().ok())
    {
        req_params.key = hdr.to_string();
    }
    let res = CredentialService::verify_api_key(&ctx.db, &req_params).await?;
    Ok(format_res::ok(res)?)
}

/// 删除指定的 API Key
#[utoipa::path(
    delete,
    path = "/v1/api-keys",
    summary = "删除 API 密钥",
    description = "删除指定 API 密钥",
    params(
        ("keyId" = String, Query, description = "API Key ID")
    ),
    responses(
        (status = 200, description = "删除成功", body = StatusResource),
        (status = 401, description = "未登录", body = ApiError),
        (status = 404, description = "密钥不存在或无权操作", body = ApiError)
    ),
    tag = TAG_CREDENTIALS
)]
#[debug_handler]
pub async fn delete_api_key(
    State(ctx): State<AppContext>,
    auth: AuthSession,
    Query(params): Query<DeleteApiKeyRequest>,
) -> AppResult<Response> {
    let res = CredentialService::delete_api_key(&ctx.db, &auth.user, &params).await?;
    Ok(format_res::ok(res)?)
}

/// 更新 API Key 元数据
#[utoipa::path(
    patch,
    path = "/v1/api-keys",
    summary = "更新 API 密钥",
    description = "更新密钥名称或状态",
    request_body = UpdateApiKeyRequest,
    responses(
        (status = 200, description = "更新成功", body = ApiKeyResource),
        (status = 401, description = "未登录", body = ApiError),
        (status = 403, description = "权限拒绝", body = ApiError),
        (status = 404, description = "密钥不存在", body = ApiError)
    ),
    tag = TAG_CREDENTIALS
)]
#[debug_handler]
pub async fn update_api_key(
    State(ctx): State<AppContext>,
    auth: AuthSession,
    ValidJson(params): ValidJson<UpdateApiKeyRequest>,
) -> AppResult<Response> {
    let res = CredentialService::update_api_key(&ctx.db, &auth.user, &params).await?;
    Ok(format_res::ok(res)?)
}
