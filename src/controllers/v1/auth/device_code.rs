use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use loco_rs::prelude::*;

use crate::{
    common::consts::openapi::TAG_SIGNIN,
    dtos::{
        auth::*,
        common::{ApiError, ValidJson, format_res},
    },
    errors::AppResult,
    services::SessionService,
};

// ─────────────────────────────────────────────────────────────────────────────
// Cookie 与 Token 辅助函数
// ─────────────────────────────────────────────────────────────────────────────

use super::shared::*;

/// 申请设备授权码与用户配对码 (RFC 8628)
#[utoipa::path(
    post,
    path = "/v1/device-authorizations",
    summary = "申请设备授权码",
    description = "设备申请授权码",
    request_body = DeviceCodeRequest,
    responses(
        (status = 200, description = "申请成功", body = DeviceCodeResource)
    ),
    tag = TAG_SIGNIN
)]
#[debug_handler]
pub async fn device_code(
    State(ctx): State<AppContext>,
    ValidJson(params): ValidJson<DeviceCodeRequest>,
) -> AppResult<Response> {
    let res = SessionService::create_device_code(&ctx.db, &params).await?;
    Ok(format_res::ok(res)?)
}

/// 查询设备用户码授权状态
#[utoipa::path(
    get,
    path = "/v1/device-authorizations/{user_code}",
    summary = "查询用户配对码状态",
    description = "查询配对码授权状态",
    params(
        ("user_code" = String, Path, description = "用户设备配对码")
    ),
    responses(
        (status = 200, description = "查询成功", body = DeviceCheckResource),
        (status = 404, description = "配对码不存在", body = ApiError)
    ),
    tag = TAG_SIGNIN
)]
#[debug_handler]
pub async fn device_check(
    State(ctx): State<AppContext>,
    Path(user_code): Path<String>,
) -> AppResult<Response> {
    let res = SessionService::check_device_code(&ctx.db, &user_code).await?;
    Ok(format_res::ok(res)?)
}

/// 登录用户批准设备授权
#[utoipa::path(
    post,
    path = "/v1/device-authorization-approvals",
    summary = "批准设备授权",
    description = "确认配对码并授权",
    request_body = DeviceApproveRequest,
    responses(
        (status = 200, description = "批准成功", body = StatusResource),
        (status = 400, description = "设备码已过期或无效", body = ApiError),
        (status = 401, description = "未授权", body = ApiError)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = TAG_SIGNIN
)]
#[debug_handler]
pub async fn device_approve(
    State(ctx): State<AppContext>,
    auth: AuthSession,
    ValidJson(params): ValidJson<DeviceApproveRequest>,
) -> AppResult<Response> {
    let res =
        SessionService::approve_device_code(&ctx.db, &auth.user.id, &params.user_code).await?;
    Ok(format_res::ok(res)?)
}

/// 登录用户拒绝设备授权
#[utoipa::path(
    post,
    path = "/v1/device-authorization-denials",
    summary = "拒绝设备授权",
    description = "登录态用户拒绝该设备的授权申请",
    request_body = DeviceDenyRequest,
    responses(
        (status = 200, description = "拒绝成功", body = StatusResource),
        (status = 400, description = "设备码已过期或无效", body = ApiError),
        (status = 401, description = "未授权", body = ApiError)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = TAG_SIGNIN
)]
#[debug_handler]
pub async fn device_deny(
    State(ctx): State<AppContext>,
    auth: AuthSession,
    ValidJson(params): ValidJson<DeviceDenyRequest>,
) -> AppResult<Response> {
    let res = SessionService::deny_device_code(&ctx.db, &auth.user.id, &params.user_code).await?;
    Ok(format_res::ok(res)?)
}

/// 设备轮询获取 Access Token (RFC 8628)
#[utoipa::path(
    post,
    path = "/v1/device-authorization-tokens",
    summary = "设备轮询获取令牌",
    description = "轮询设备码换取令牌",
    request_body = DeviceTokenRequest,
    responses(
        (status = 200, description = "授权通过并签发会话令牌", body = DeviceTokenResource),
        (status = 400, description = "授权等待或被拒绝", body = ApiError)
    ),
    tag = TAG_SIGNIN
)]
#[debug_handler]
pub async fn device_token(
    State(ctx): State<AppContext>,
    headers: HeaderMap,
    ValidJson(params): ValidJson<DeviceTokenRequest>,
) -> AppResult<Response> {
    let (ip, ua) = extract_client_info(&headers);
    let res = SessionService::poll_device_token(&ctx.db, &params, ip, ua).await?;
    // 设备轮询客户端非浏览器，token 仅经响应体交付，不写浏览器 Cookie
    Ok((StatusCode::OK, Json(res)).into_response())
}
