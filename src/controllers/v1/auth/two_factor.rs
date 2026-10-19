use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode, header::SET_COOKIE},
    response::{IntoResponse, Response},
};
use loco_rs::prelude::*;
use secrecy::ExposeSecret;

use crate::{
    common::consts::openapi::TAG_CREDENTIALS,
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

// ──────────────────── Two-Factor (2FA) ────────────────────

/// 启用两步验证 (生成 TOTP Secret 与 10 组备用码)
#[utoipa::path(
    post,
    path = "/v1/users/me/two-factor",
    summary = "开启两步验证",
    description = "开启两步验证",
    request_body = TwoFactorPasswordRequest,
    responses(
        (status = 200, description = "生成两步验证凭证成功", body = TwoFactorEnableResource),
        (status = 401, description = "密码验证错误或未授权", body = ApiError)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = TAG_CREDENTIALS
)]
#[debug_handler]
pub async fn two_factor_enable(
    State(ctx): State<AppContext>,
    auth: AuthSession,
    ValidJson(params): ValidJson<TwoFactorPasswordRequest>,
) -> AppResult<Response> {
    let res = VerificationService::enable_two_factor(
        &ctx.db,
        &auth.user.id,
        params.password.expose_secret(),
    )
    .await?;
    Ok(format_res::ok(res)?)
}

/// 校验 TOTP 动态码 (支持已登录激活或未登录登录挑战)
#[utoipa::path(
    post,
    path = "/v1/totp-checks",
    summary = "校验动态口令",
    description = "校验 TOTP 并签发会话",
    request_body = VerifyTotpRequest,
    responses(
        (status = 200, description = "TOTP 校验成功", body = SignInResponse),
        (status = 400, description = "验证码错误或 2FA 未初始化", body = ApiError),
        (status = 401, description = "缺少认证会话或 Challenge 凭证", body = ApiError),
        (status = 403, description = "连续错误次数过多导致锁定", body = ApiError)
    ),
    tag = TAG_CREDENTIALS
)]
#[debug_handler]
pub async fn two_factor_verify_totp(
    State(ctx): State<AppContext>,
    headers: HeaderMap,
    ValidJson(params): ValidJson<VerifyTotpRequest>,
) -> AppResult<Response> {
    let (ip, ua) = extract_client_info(&headers);
    let authed_user_id = extract_optional_auth(&ctx, &headers).await;
    let challenge_token = extract_challenge_token_from_headers(&headers);

    let res = VerificationService::verify_totp(
        &ctx.db,
        authed_user_id.as_deref(),
        challenge_token.as_deref(),
        &params.code,
        ip,
        ua,
    )
    .await?;

    let mut resp_headers = HeaderMap::new();
    if let Some(ref token) = res.token
        && !token.is_empty()
    {
        resp_headers.insert(
            SET_COOKIE,
            build_session_cookie(token, SESSION_COOKIE_MAX_AGE_SECS),
        );
    }

    Ok((StatusCode::OK, resp_headers, Json(res)).into_response())
}

/// 校验备用恢复码
#[utoipa::path(
    post,
    path = "/v1/backup-code-checks",
    summary = "校验恢复码",
    description = "使用恢复码登录",
    request_body = VerifyBackupCodeRequest,
    responses(
        (status = 200, description = "备用恢复码校验成功并登录", body = SignInResponse),
        (status = 400, description = "恢复码错误或无效", body = ApiError),
        (status = 401, description = "缺少 Challenge 凭证", body = ApiError),
        (status = 403, description = "连续错误次数过多导致锁定", body = ApiError)
    ),
    tag = TAG_CREDENTIALS
)]
#[debug_handler]
pub async fn two_factor_verify_backup_code(
    State(ctx): State<AppContext>,
    headers: HeaderMap,
    ValidJson(params): ValidJson<VerifyBackupCodeRequest>,
) -> AppResult<Response> {
    let (ip, ua) = extract_client_info(&headers);
    let authed_user_id = extract_optional_auth(&ctx, &headers).await;
    let challenge_token = extract_challenge_token_from_headers(&headers);

    let res = VerificationService::verify_backup_code(
        &ctx.db,
        authed_user_id.as_deref(),
        challenge_token.as_deref(),
        &params.code,
        ip,
        ua,
    )
    .await?;

    let mut resp_headers = HeaderMap::new();
    if let Some(ref token) = res.token
        && !token.is_empty()
    {
        resp_headers.insert(
            SET_COOKIE,
            build_session_cookie(token, SESSION_COOKIE_MAX_AGE_SECS),
        );
    }

    Ok((StatusCode::OK, resp_headers, Json(res)).into_response())
}

/// 禁用两步验证
#[utoipa::path(
    delete,
    path = "/v1/users/me/two-factor",
    summary = "关闭两步验证",
    description = "关闭两步验证",
    request_body = TwoFactorPasswordRequest,
    responses(
        (status = 200, description = "两步验证关闭成功", body = StatusResource),
        (status = 401, description = "密码验证错误或未授权", body = ApiError)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = TAG_CREDENTIALS
)]
#[debug_handler]
pub async fn two_factor_disable(
    State(ctx): State<AppContext>,
    auth: AuthSession,
    ValidJson(params): ValidJson<TwoFactorPasswordRequest>,
) -> AppResult<Response> {
    let res = VerificationService::disable_two_factor(
        &ctx.db,
        &auth.user.id,
        params.password.expose_secret(),
    )
    .await?;
    Ok(format_res::ok(res)?)
}

/// 重新生成 10 组备用恢复码
#[utoipa::path(
    post,
    path = "/v1/backup-codes",
    summary = "重生成恢复码",
    description = "重生成恢复码",
    request_body = TwoFactorPasswordRequest,
    responses(
        (status = 200, description = "新备用码生成成功", body = TwoFactorBackupCodesResource),
        (status = 401, description = "密码验证错误或未授权", body = ApiError)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = TAG_CREDENTIALS
)]
#[debug_handler]
pub async fn two_factor_generate_backup_codes(
    State(ctx): State<AppContext>,
    auth: AuthSession,
    ValidJson(params): ValidJson<TwoFactorPasswordRequest>,
) -> AppResult<Response> {
    let res = VerificationService::generate_backup_codes(
        &ctx.db,
        &auth.user.id,
        params.password.expose_secret(),
    )
    .await?;
    Ok(format_res::ok(res)?)
}
