use axum::{
    Json,
    extract::{FromRef, FromRequestParts},
    http::{HeaderMap, HeaderValue, StatusCode, header::SET_COOKIE, request::Parts},
    response::{IntoResponse, Response},
};
use loco_rs::prelude::*;

use crate::{
    dtos::auth::*,
    errors::AppError,
    models::{session, user},
};

// ─────────────────────────────────────────────────────────────────────────────
// Cookie 与 Token 辅助函数
// ─────────────────────────────────────────────────────────────────────────────

/// 会话 Cookie 有效期 (7 天，与服务端滑动窗口保持一致)
pub const SESSION_COOKIE_MAX_AGE_SECS: i64 = 7 * 86400;

/// 2FA 挑战 Cookie 有效期 (5 分钟，仅覆盖 2FA 校验窗口)
pub const CHALLENGE_COOKIE_MAX_AGE_SECS: i64 = 300;

/// 生产环境启用 `__Secure-` 前缀 + Secure 属性（HTTPS 强制），
/// 开发/测试环境沿用普通名称以便 HTTP 本地调试；读取端两种名称均兼容。
fn secure_cookies_enabled() -> bool {
    std::env::var("LOCO_ENV").is_ok_and(|e| e == "production")
}

/// 按环境返回 Cookie 名称前缀与 Secure 属性片段
fn cookie_security() -> (&'static str, &'static str) {
    if secure_cookies_enabled() {
        ("__Secure-better-auth", "; Secure")
    } else {
        ("better-auth", "")
    }
}

/// 从 HeaderMap 中优先提取 Bearer Token，回退提取 Cookie
#[must_use]
pub fn extract_token_from_headers(headers: &HeaderMap) -> Option<String> {
    // 1. 优先提取 Authorization: Bearer <token>
    if let Some(auth_val) = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        && let Some(token) = auth_val.strip_prefix("Bearer ")
    {
        let token = token.trim();
        if !token.is_empty() {
            return Some(token.to_string());
        }
    }

    // 2. 回退解析 Cookie 中的 better-auth.session_token 或 __Secure-better-auth.session_token
    if let Some(cookie_val) = headers
        .get(axum::http::header::COOKIE)
        .and_then(|v| v.to_str().ok())
    {
        for cookie in cookie_val.split(';') {
            let trimmed = cookie.trim();
            if let Some(token) = trimmed.strip_prefix("better-auth.session_token=") {
                let token = token.trim();
                if !token.is_empty() {
                    return Some(token.to_string());
                }
            } else if let Some(token) = trimmed.strip_prefix("__Secure-better-auth.session_token=")
            {
                let token = token.trim();
                if !token.is_empty() {
                    return Some(token.to_string());
                }
            }
        }
    }

    None
}

/// 提取客户端 IP 与 User-Agent
#[must_use]
pub fn extract_client_info(headers: &HeaderMap) -> (Option<String>, Option<String>) {
    let ip = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(std::string::ToString::to_string)
        });
    let ua = headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .map(std::string::ToString::to_string);
    (ip, ua)
}

/// 构建会话 Cookie HeaderValue (遵循 Better-Auth 命名与现代安全规范)
#[must_use]
pub fn build_session_cookie(token: &str, max_age_seconds: i64) -> HeaderValue {
    let (name, secure) = cookie_security();
    let cookie = format!(
        "{name}.session_token={token}; Path=/; HttpOnly; SameSite=Lax{secure}; Max-Age={max_age_seconds}"
    );
    HeaderValue::from_str(&cookie).expect("Valid cookie header value")
}

/// 构建清除会话 Cookie HeaderValue
#[must_use]
pub fn build_clear_session_cookie() -> HeaderValue {
    let (name, secure) = cookie_security();
    let cookie =
        format!("{name}.session_token=; Path=/; HttpOnly; SameSite=Lax{secure}; Max-Age=0");
    HeaderValue::from_str(&cookie).expect("Valid cookie header value")
}

/// 构建 2FA Challenge Cookie HeaderValue
#[must_use]
pub fn build_challenge_cookie(token: &str, max_age_seconds: i64) -> HeaderValue {
    let (name, secure) = cookie_security();
    let cookie = format!(
        "{name}.challenge={token}; Path=/; HttpOnly; SameSite=Lax{secure}; Max-Age={max_age_seconds}"
    );
    HeaderValue::from_str(&cookie).expect("Valid cookie header value")
}

/// 提取 2FA Challenge 临时校验 Token (优先 Header，回退 Cookie)
#[must_use]
pub fn extract_challenge_token_from_headers(headers: &HeaderMap) -> Option<String> {
    if let Some(val) = headers
        .get("x-better-auth-challenge")
        .and_then(|v| v.to_str().ok())
    {
        let t = val.trim();
        if !t.is_empty() {
            return Some(t.to_string());
        }
    }
    if let Some(cookie_val) = headers
        .get(axum::http::header::COOKIE)
        .and_then(|v| v.to_str().ok())
    {
        for cookie in cookie_val.split(';') {
            let trimmed = cookie.trim();
            if let Some(token) = trimmed.strip_prefix("better-auth.challenge=") {
                let token = token.trim();
                if !token.is_empty() {
                    return Some(token.to_string());
                }
            } else if let Some(token) = trimmed.strip_prefix("__Secure-better-auth.challenge=") {
                let token = token.trim();
                if !token.is_empty() {
                    return Some(token.to_string());
                }
            }
        }
    }
    None
}

/// 统一格式化包含 Session / 2FA Challenge 的登录响应
///
/// 以 `two_factor_redirect` 字段判别分支：存在即写入 2FA 挑战 Cookie，
/// 否则写入常规会话 Cookie。
#[must_use]
pub fn format_sign_in_result(resp: (SignInResponse, Option<String>)) -> Response {
    let (sign_in_resp, challenge) = resp;
    let mut resp_headers = HeaderMap::new();

    if sign_in_resp.two_factor_redirect.is_some() {
        if let Some(ref ch) = challenge {
            resp_headers.insert(
                SET_COOKIE,
                build_challenge_cookie(ch, CHALLENGE_COOKIE_MAX_AGE_SECS),
            );
        }
    } else if let Some(ref token) = sign_in_resp.token
        && !token.is_empty()
    {
        resp_headers.insert(
            SET_COOKIE,
            build_session_cookie(token, SESSION_COOKIE_MAX_AGE_SECS),
        );
    }

    (StatusCode::OK, resp_headers, Json(sign_in_resp)).into_response()
}

/// 可选认证探测：有有效会话则返回 user_id，否则 None（不拒绝请求）
///
/// 供「已登录则携带身份、未登录也可匿名」的端点复用，控制器内禁止直查 DB。
pub async fn extract_optional_auth(ctx: &AppContext, headers: &HeaderMap) -> Option<String> {
    let token = extract_token_from_headers(headers)?;
    let session = session::Entity::find_valid_by_token(&ctx.db, &token)
        .await
        .ok()
        .flatten()?;
    let user = user::Entity::find_by_id(&ctx.db, &session.user_id)
        .await
        .ok()
        .flatten()?;
    (!user.is_banned()).then_some(session.user_id)
}

// ─────────────────────────────────────────────────────────────────────────────
// AuthSession 提取器 (双轨提取 Bearer Token / Cookie 并验证)
// ─────────────────────────────────────────────────────────────────────────────

/// 认证会话提取器
#[derive(Debug, Clone)]
pub struct AuthSession {
    pub token: String,
    pub session: session::Model,
    pub user: user::Model,
}

impl<S> FromRequestParts<S> for AuthSession
where
    S: Send + Sync,
    AppContext: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let ctx = AppContext::from_ref(state);
        let Some(token) = extract_token_from_headers(&parts.headers) else {
            return Err(AppError::unauthorized(
                "Unauthorized: missing session token",
            ));
        };

        let session = session::Entity::find_valid_by_token(&ctx.db, &token)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "认证域内部查询失败");
                AppError::internal("Internal authentication error")
            })?;
        let Some(session) = session else {
            return Err(AppError::unauthorized("Session expired or invalid"));
        };

        // 使用含软删的查询，以便将「已注销」与「不存在」区分开，便于排障
        let user = user::Entity::find_by_id_with_deleted(&ctx.db, &session.user_id)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "认证域内部查询失败");
                AppError::internal("Internal authentication error")
            })?;
        let Some(user) = user else {
            return Err(AppError::unauthorized("User not found"));
        };

        if user.is_deleted() {
            return Err(AppError::forbidden("Account has been deleted"));
        }
        if user.is_banned() {
            return Err(AppError::forbidden("Account is banned"));
        }

        // 7天滑动窗口自动续期
        let session = session.touch_sliding_window(&ctx.db).await.map_err(|e| {
            tracing::error!(error = %e, "认证域内部查询失败");
            AppError::internal("Internal authentication error")
        })?;

        Ok(Self {
            token,
            session,
            user,
        })
    }
}
