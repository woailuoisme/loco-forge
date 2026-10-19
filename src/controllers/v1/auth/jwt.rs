use crate::dtos::common::format_res;
use axum::{extract::State, response::Response};
use loco_rs::prelude::*;

use crate::{
    common::consts::openapi::TAG_SESSION,
    dtos::{auth::*, common::ApiError},
    errors::AppResult,
    services::SessionService,
};

// ─────────────────────────────────────────────────────────────────────────────
// Cookie 与 Token 辅助函数
// ─────────────────────────────────────────────────────────────────────────────

use super::shared::*;

/// 获取用户 JWT 令牌
#[utoipa::path(
    post,
    path = "/v1/access-tokens",
    summary = "签发 JWT 令牌",
    description = "签发短期 JWT 令牌",
    responses(
        (status = 200, description = "签发成功", body = JwtTokenResource),
        (status = 401, description = "未授权", body = ApiError)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = TAG_SESSION
)]
#[debug_handler]
pub async fn get_jwt_token(
    State(ctx): State<AppContext>,
    auth: AuthSession,
) -> AppResult<Response> {
    let res = SessionService::get_jwt_token(&ctx, &auth.user.id).await?;
    Ok(format_res::ok(res)?)
}

/// 获取公开 JWKS 密钥集
#[utoipa::path(
    get,
    path = "/v1/jwks",
    summary = "JWKS 公钥集",
    description = "返回 JWKS 公钥集",
    responses(
        (status = 200, description = "获取成功", body = JwksResource)
    ),
    tag = TAG_SESSION
)]
#[debug_handler]
pub async fn get_jwks(State(ctx): State<AppContext>) -> AppResult<Response> {
    let res = SessionService::get_jwks(&ctx).await?;
    Ok(format_res::ok(res)?)
}
