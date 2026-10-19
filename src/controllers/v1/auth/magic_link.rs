use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderMap, HeaderValue, StatusCode, header::SET_COOKIE},
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
    services::VerificationService,
};

// ─────────────────────────────────────────────────────────────────────────────
// Cookie 与 Token 辅助函数
// ─────────────────────────────────────────────────────────────────────────────

use super::shared::*;

// ──────────────────── Magic Link ────────────────────

/// 发送 Magic Link 免密登录链接
#[utoipa::path(
    post,
    path = "/v1/sessions/magic-link",
    summary = "发送魔法链接",
    description = "向用户邮箱发送免密登录链接 Token",
    request_body = SignInMagicLinkRequest,
    responses(
        (status = 200, description = "Magic Link 发送成功", body = StatusResource),
        (status = 422, description = "参数校验失败", body = ApiError)
    ),
    tag = TAG_SIGNIN
)]
#[debug_handler]
pub async fn sign_in_magic_link(
    State(ctx): State<AppContext>,
    ValidJson(params): ValidJson<SignInMagicLinkRequest>,
) -> AppResult<Response> {
    let res = VerificationService::send_magic_link(&ctx.db, &params).await?;
    Ok(format_res::ok(res)?)
}

/// 校验 Magic Link Token 并登入
#[utoipa::path(
    get,
    path = "/v1/magic-link-tokens/{token}",
    summary = "校验魔法链接",
    description = "校验链接并签发会话",
    params(
        ("token" = String, Query, description = "Magic Link 校验凭证"),
        ("callbackURL" = Option<String>, Query, description = "登录后重定向目标 URL")
    ),
    responses(
        (status = 200, description = "登录成功", body = AuthResource),
        (status = 302, description = "重定向至目标页面"),
        (status = 400, description = "Token 无效或已过期", body = ApiError)
    ),
    tag = TAG_SIGNIN
)]
#[debug_handler]
pub async fn verify_magic_link(
    State(ctx): State<AppContext>,
    headers: HeaderMap,
    Path(token): Path<String>,
    Query(query): Query<VerifyMagicLinkQuery>,
) -> AppResult<Response> {
    let (ip, ua) = extract_client_info(&headers);
    let res = VerificationService::verify_magic_link(&ctx.db, &token, ip, ua).await?;

    let mut resp_headers = HeaderMap::new();
    resp_headers.insert(
        SET_COOKIE,
        build_session_cookie(&res.token, SESSION_COOKIE_MAX_AGE_SECS),
    );

    // 重定向仅允许站内相对路径，杜绝开放重定向
    if let Some(ref url) = query.callback_url {
        let trimmed = url.trim();
        if is_safe_redirect_target(trimmed)
            && let Ok(loc) = HeaderValue::from_str(trimmed)
        {
            resp_headers.insert(axum::http::header::LOCATION, loc);
            return Ok((StatusCode::FOUND, resp_headers, ()).into_response());
        }
    }

    Ok((StatusCode::OK, resp_headers, Json(res)).into_response())
}

/// 校验重定向目标是否为站内相对路径（以 `/` 开头且非协议相对 `//`）
#[must_use]
fn is_safe_redirect_target(url: &str) -> bool {
    url.starts_with('/')
        && !url.starts_with("//")
        && !url.starts_with("/\\")
        && !url.contains("://")
        && !url.is_empty()
}
