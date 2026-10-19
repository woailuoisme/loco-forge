//! 后台管理员认证接口 (Admin Auth Endpoints)
//!
//! 挂载于 `/v1/admin/auth/*`，受专用严格限流保护 (见
//! [`admin_login_rate_limit_layer`](crate::setup::middleware::rate_limit::admin_login_rate_limit_layer))。

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode, header::SET_COOKIE},
    response::{IntoResponse, Response},
};
use loco_rs::prelude::*;

use crate::{
    common::consts::openapi::TAG_ADMIN,
    controllers::v1::auth::{
        AuthSession, build_clear_session_cookie, extract_client_info, format_sign_in_result,
    },
    dtos::{
        admin::{AdminLoginRequest, AdminMeResource},
        auth::SignInResponse,
        common::ValidJson,
    },
    errors::AppResult,
    services::AdminAuthService,
};

/// 后台管理员登录
#[utoipa::path(
    post,
    path = "/v1/admin/auth/login",
    summary = "后台登录",
    description = "仅限管理员账号；非管理员与密码错误返回同一文案以杜绝账号枚举",
    request_body = AdminLoginRequest,
    responses(
        (status = 200, description = "登录成功", body = SignInResponse),
        (status = 401, description = "凭据无效或账号非管理员"),
        (status = 429, description = "尝试过于频繁")
    ),
    tag = TAG_ADMIN
)]
#[debug_handler]
pub async fn login(
    State(ctx): State<AppContext>,
    headers: HeaderMap,
    ValidJson(params): ValidJson<AdminLoginRequest>,
) -> AppResult<Response> {
    let (ip, ua) = extract_client_info(&headers);
    let res = AdminAuthService::login(&ctx.db, &params, ip, ua).await?;
    Ok(format_sign_in_result(res))
}

/// 后台管理员登出
#[utoipa::path(
    post,
    path = "/v1/admin/auth/logout",
    summary = "后台登出",
    description = "吊销当前会话并清除 Cookie",
    responses(
        (status = 204, description = "登出成功，无响应体"),
        (status = 401, description = "未授权")
    ),
    security(("bearer_auth" = [])),
    tag = TAG_ADMIN
)]
#[debug_handler]
pub async fn logout(State(ctx): State<AppContext>, auth: AuthSession) -> AppResult<Response> {
    require_admin(&auth)?;
    AdminAuthService::logout(&ctx.db, &auth.user.id, &auth.token).await?;

    let mut resp_headers = HeaderMap::new();
    resp_headers.insert(SET_COOKIE, build_clear_session_cookie());
    Ok((StatusCode::NO_CONTENT, resp_headers).into_response())
}

/// 获取当前管理员信息
#[utoipa::path(
    get,
    path = "/v1/admin/auth/me",
    summary = "当前管理员",
    description = "返回管理员完整运维投影与当前会话详情",
    responses(
        (status = 200, description = "获取成功", body = AdminMeResource),
        (status = 401, description = "未授权"),
        (status = 403, description = "需要管理员权限")
    ),
    security(("bearer_auth" = [])),
    tag = TAG_ADMIN
)]
#[debug_handler]
pub async fn me(_state: State<AppContext>, auth: AuthSession) -> AppResult<Response> {
    require_admin(&auth)?;
    Ok((
        StatusCode::OK,
        Json(AdminAuthService::me(&auth.user, &auth.session)),
    )
        .into_response())
}

/// 管理员角色守卫（与 AdminUser 提取器同语义，供需 session/token 的端点复用）
fn require_admin(auth: &AuthSession) -> AppResult<()> {
    if !auth.user.is_admin() {
        return Err(crate::errors::AppError::forbidden(
            "Access denied: admin role required",
        ));
    }
    Ok(())
}
