use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode, header::SET_COOKIE},
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

// ─────────────────────────────────────────────────────────────────────────────
// Phase 2: 验证与多因素认证处理函数 (Email OTP / Magic Link / Phone / 2FA)
// ─────────────────────────────────────────────────────────────────────────────

// ──────────────────── Email OTP ────────────────────

/// 发送邮箱验证码 OTP
#[utoipa::path(
    post,
    path = "/v1/verification-codes",
    summary = "发送邮箱验证码",
    description = "发送 6 位邮箱验证码",
    request_body = SendEmailOtpRequest,
    responses(
        (status = 200, description = "验证码发送成功", body = StatusResource),
        (status = 422, description = "参数校验失败", body = ApiError)
    ),
    tag = TAG_SIGNIN
)]
#[debug_handler]
pub async fn send_verification_otp(
    State(ctx): State<AppContext>,
    ValidJson(params): ValidJson<SendEmailOtpRequest>,
) -> AppResult<Response> {
    let res = VerificationService::send_email_otp(&ctx.db, &params).await?;
    Ok(format_res::ok(res)?)
}

/// 检查邮箱 OTP 验证码
#[utoipa::path(
    post,
    path = "/v1/verification-code-checks",
    summary = "校验邮箱验证码",
    description = "预校验邮箱验证码",
    request_body = CheckEmailOtpRequest,
    responses(
        (status = 200, description = "验证码有效", body = StatusResource),
        (status = 400, description = "验证码无效或已过期", body = ApiError)
    ),
    tag = TAG_SIGNIN
)]
#[debug_handler]
pub async fn check_verification_otp(
    State(ctx): State<AppContext>,
    ValidJson(params): ValidJson<CheckEmailOtpRequest>,
) -> AppResult<Response> {
    let res = VerificationService::check_email_otp(&ctx.db, &params).await?;
    Ok(format_res::ok(res)?)
}

/// 邮箱 OTP 验证码登录
#[utoipa::path(
    post,
    path = "/v1/sessions/email-otp",
    summary = "邮箱验证码登录",
    description = "校验邮箱验证码",
    request_body = SignInEmailOtpRequest,
    responses(
        (status = 200, description = "登录成功", body = AuthResource),
        (status = 400, description = "验证码无效或已过期", body = ApiError),
        (status = 403, description = "账号已被封禁", body = ApiError)
    ),
    tag = TAG_SIGNIN
)]
#[debug_handler]
pub async fn sign_in_email_otp(
    State(ctx): State<AppContext>,
    headers: HeaderMap,
    ValidJson(params): ValidJson<SignInEmailOtpRequest>,
) -> AppResult<Response> {
    let (ip, ua) = extract_client_info(&headers);
    let res = VerificationService::sign_in_email_otp(&ctx.db, &params, ip, ua).await?;

    let mut resp_headers = HeaderMap::new();
    resp_headers.insert(
        SET_COOKIE,
        build_session_cookie(&res.token, SESSION_COOKIE_MAX_AGE_SECS),
    );
    Ok((StatusCode::OK, resp_headers, Json(res)).into_response())
}

/// 验证邮箱
#[utoipa::path(
    post,
    path = "/v1/email-verifications",
    summary = "验证邮箱",
    description = "邮箱验证码验证邮箱",
    request_body = VerifyEmailOtpRequest,
    responses(
        (status = 200, description = "邮箱验证成功", body = VerifyEmailOtpResource),
        (status = 400, description = "验证码无效或已过期", body = ApiError),
        (status = 404, description = "用户不存在", body = ApiError)
    ),
    tag = TAG_SIGNIN
)]
#[debug_handler]
pub async fn verify_email(
    State(ctx): State<AppContext>,
    ValidJson(params): ValidJson<VerifyEmailOtpRequest>,
) -> AppResult<Response> {
    let res = VerificationService::verify_email_otp(&ctx.db, &params).await?;
    Ok(format_res::ok(res)?)
}

/// 邮箱验证码重置密码
#[utoipa::path(
    patch,
    path = "/v1/users/me/password",
    summary = "邮箱验证码重置密码",
    description = "重置密码",
    request_body = ResetPasswordEmailOtpRequest,
    responses(
        (status = 200, description = "密码重置成功", body = StatusResource),
        (status = 400, description = "验证码无效或已过期", body = ApiError),
        (status = 404, description = "用户不存在", body = ApiError)
    ),
    tag = TAG_SIGNIN
)]
#[debug_handler]
pub async fn reset_password(
    State(ctx): State<AppContext>,
    ValidJson(params): ValidJson<ResetPasswordEmailOtpRequest>,
) -> AppResult<Response> {
    let res = VerificationService::reset_password_email_otp(&ctx.db, &params).await?;
    Ok(format_res::ok(res)?)
}

// ──────────────────── Phone Number ────────────────────

/// 发送手机验证码 OTP
#[utoipa::path(
    post,
    path = "/v1/phone-verification-codes",
    summary = "发送手机号短信验证码",
    description = "向指定手机号发送 6 位短信验证码",
    request_body = SendPhoneOtpRequest,
    responses(
        (status = 200, description = "短信验证码发送成功", body = StatusResource),
        (status = 422, description = "参数校验失败", body = ApiError)
    ),
    tag = TAG_SIGNIN
)]
#[debug_handler]
pub async fn send_phone_otp(
    State(ctx): State<AppContext>,
    ValidJson(params): ValidJson<SendPhoneOtpRequest>,
) -> AppResult<Response> {
    let res = VerificationService::send_phone_otp(&ctx.db, &params).await?;
    Ok(format_res::ok(res)?)
}

/// 校验手机验证码 (登录/注册或修改手机号)
#[utoipa::path(
    post,
    path = "/v1/phone-verifications",
    summary = "校验手机号验证码",
    description = "校验短信验证码",
    request_body = VerifyPhoneNumberRequest,
    responses(
        (status = 200, description = "手机号验证成功", body = SignInResponse),
        (status = 400, description = "验证码无效或已过期", body = ApiError),
        (status = 401, description = "修改手机号需先登录", body = ApiError)
    ),
    tag = TAG_SIGNIN
)]
#[debug_handler]
pub async fn verify_phone_number(
    State(ctx): State<AppContext>,
    headers: HeaderMap,
    ValidJson(params): ValidJson<VerifyPhoneNumberRequest>,
) -> AppResult<Response> {
    let (ip, ua) = extract_client_info(&headers);
    let current_user_id = extract_optional_auth(&ctx, &headers).await;

    let res = VerificationService::verify_phone_number(
        &ctx.db,
        current_user_id.as_deref(),
        &params,
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

/// 手机号密码登录
#[utoipa::path(
    post,
    path = "/v1/sessions/phone-number",
    summary = "手机号密码登录",
    description = "手机号密码登录",
    request_body = SignInPhoneNumberRequest,
    responses(
        (status = 200, description = "登录成功", body = SignInResponse),
        (status = 401, description = "手机号或密码错误", body = ApiError),
        (status = 403, description = "账号已被封禁", body = ApiError)
    ),
    tag = TAG_SIGNIN
)]
#[debug_handler]
pub async fn sign_in_phone_number(
    State(ctx): State<AppContext>,
    headers: HeaderMap,
    ValidJson(params): ValidJson<SignInPhoneNumberRequest>,
) -> AppResult<Response> {
    let (ip, ua) = extract_client_info(&headers);
    let res = VerificationService::sign_in_phone_number(&ctx.db, &params, ip, ua).await?;
    Ok(format_sign_in_result(res))
}
