use crate::{
    common::consts::openapi::TAG_AUTH,
    dtos::{
        auth::{
            CurrentUserResource, ForgotPasswordRequest, LoginRequest, LoginResource,
            RegisterRequest, ResendVerificationRequest, ResetPasswordRequest, SendOtpRequest,
            VerifyOtpRequest,
        },
        common::{ApiError, ApiResponse, ValidJson},
    },
    errors::AppResult,
    services::AuthService,
};
use loco_rs::prelude::*;

/// 用户注册
#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    summary = "用户注册",
    description = "注册新用户并发送邮箱验证邮件",
    request_body = RegisterRequest,
    responses(
        (status = 200, description = "注册请求已受理", body = ApiResponse<serde_json::Value>)
    ),
    tag = TAG_AUTH
)]
#[debug_handler]
pub async fn register(
    State(ctx): State<AppContext>,
    ValidJson(params): ValidJson<RegisterRequest>,
) -> AppResult<ApiResponse<serde_json::Value>> {
    AuthService::register(&ctx, &params).await?;
    Ok(ApiResponse::success_empty("注册请求已受理"))
}

/// 邮箱验证
#[utoipa::path(
    get,
    path = "/api/v1/auth/verify/{token}",
    summary = "邮箱验证",
    description = "通过验证 Token 完成邮箱激活",
    params(
        ("token" = String, Path, description = "邮箱验证 Token")
    ),
    responses(
        (status = 200, description = "邮箱验证成功", body = ApiResponse<serde_json::Value>),
        (status = 401, description = "验证 Token 无效或已过期", body = ApiError)
    ),
    tag = TAG_AUTH
)]
#[debug_handler]
pub async fn verify(
    State(ctx): State<AppContext>,
    Path(token): Path<String>,
) -> AppResult<ApiResponse<serde_json::Value>> {
    AuthService::verify(&ctx, &token).await?;
    Ok(ApiResponse::success_empty("邮箱验证成功"))
}

/// 忘记密码
#[utoipa::path(
    post,
    path = "/api/v1/auth/forgot",
    summary = "忘记密码",
    description = "提交邮箱请求发送密码重置邮件",
    request_body = ForgotPasswordRequest,
    responses(
        (status = 200, description = "重置密码邮件请求已受理", body = ApiResponse<serde_json::Value>)
    ),
    tag = TAG_AUTH
)]
#[debug_handler]
pub async fn forgot(
    State(ctx): State<AppContext>,
    ValidJson(params): ValidJson<ForgotPasswordRequest>,
) -> AppResult<ApiResponse<serde_json::Value>> {
    AuthService::forgot(&ctx, &params.email).await?;
    Ok(ApiResponse::success_empty("重置密码邮件已发送"))
}

/// 重置密码
#[utoipa::path(
    post,
    path = "/api/v1/auth/reset",
    summary = "重置密码",
    description = "提交重置 Token 与新密码完成修改",
    request_body = ResetPasswordRequest,
    responses(
        (status = 200, description = "密码重置成功", body = ApiResponse<serde_json::Value>)
    ),
    tag = TAG_AUTH
)]
#[debug_handler]
pub async fn reset(
    State(ctx): State<AppContext>,
    ValidJson(params): ValidJson<ResetPasswordRequest>,
) -> AppResult<ApiResponse<serde_json::Value>> {
    use secrecy::ExposeSecret;
    AuthService::reset(&ctx, &params.token, params.password.expose_secret()).await?;
    Ok(ApiResponse::success_empty("密码重置成功"))
}

/// 用户登录
#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    summary = "用户登录",
    description = "校验邮箱密码并返回登录凭证 JWT",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "登录成功", body = ApiResponse<LoginResource>),
        (status = 401, description = "邮箱或密码错误", body = ApiError)
    ),
    tag = TAG_AUTH
)]
#[debug_handler]
pub async fn login(
    State(ctx): State<AppContext>,
    ValidJson(params): ValidJson<LoginRequest>,
) -> AppResult<ApiResponse<LoginResource>> {
    let res = AuthService::login(&ctx, &params).await?;
    Ok(ApiResponse::ok(res))
}

/// 获取当前用户信息
#[utoipa::path(
    get,
    path = "/api/v1/auth/current",
    summary = "获取当前用户信息",
    description = "根据 JWT 获取当前登录用户的个人资料",
    responses(
        (status = 200, description = "获取成功", body = ApiResponse<CurrentUserResource>),
        (status = 401, description = "未授权或 Token 无效", body = ApiError)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = TAG_AUTH
)]
#[debug_handler]
pub async fn current(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
) -> AppResult<ApiResponse<CurrentUserResource>> {
    let res = AuthService::current(&ctx, &auth.claims.pid).await?;
    Ok(ApiResponse::ok(res))
}

/// 发送 6 位数字登录验证码 (OTP)
#[utoipa::path(
    post,
    path = "/api/v1/auth/otp/send",
    summary = "发送登录验证码",
    description = "向指定邮箱发送 6 位数字登录验证码",
    request_body = SendOtpRequest,
    responses(
        (status = 200, description = "验证码发送请求已受理", body = ApiResponse<serde_json::Value>)
    ),
    tag = TAG_AUTH
)]
#[debug_handler]
pub async fn send_otp(
    State(ctx): State<AppContext>,
    ValidJson(params): ValidJson<SendOtpRequest>,
) -> AppResult<ApiResponse<serde_json::Value>> {
    AuthService::send_otp(&ctx, &params.email).await?;
    Ok(ApiResponse::success_empty("验证码已发送"))
}

/// 校验 6 位数字验证码登录 (OTP Verify)
#[utoipa::path(
    post,
    path = "/api/v1/auth/otp/verify",
    summary = "校验验证码并登录",
    description = "校验 6 位数字验证码并返回登录凭证 JWT",
    request_body = VerifyOtpRequest,
    responses(
        (status = 200, description = "验证登录成功", body = ApiResponse<LoginResource>),
        (status = 401, description = "验证码无效或已过期", body = ApiError)
    ),
    tag = TAG_AUTH
)]
#[debug_handler]
pub async fn verify_otp(
    State(ctx): State<AppContext>,
    ValidJson(params): ValidJson<VerifyOtpRequest>,
) -> AppResult<ApiResponse<LoginResource>> {
    let res = AuthService::verify_otp(&ctx, &params.email, &params.code).await?;
    Ok(ApiResponse::ok(res))
}

/// 重发邮箱验证邮件
#[utoipa::path(
    post,
    path = "/api/v1/auth/resend-verification-mail",
    summary = "重发验证邮件",
    description = "重新向未验证的用户发送验证邮件",
    request_body = ResendVerificationRequest,
    responses(
        (status = 200, description = "重发验证邮件成功", body = ApiResponse<serde_json::Value>)
    ),
    tag = TAG_AUTH
)]
#[debug_handler]
pub async fn resend_verification_email(
    State(ctx): State<AppContext>,
    ValidJson(params): ValidJson<ResendVerificationRequest>,
) -> AppResult<ApiResponse<serde_json::Value>> {
    AuthService::resend_verification(&ctx, &params.email).await?;
    Ok(ApiResponse::success_empty("验证邮件已发送"))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/v1/auth")
        .add("/register", post(register))
        .add("/verify/{token}", get(verify))
        .add("/login", post(login))
        .add("/forgot", post(forgot))
        .add("/reset", post(reset))
        .add("/current", get(current))
        .add("/otp/send", post(send_otp))
        .add("/otp/verify", post(verify_otp))
        .add("/resend-verification-mail", post(resend_verification_email))
        .layer(crate::setup::middleware::rate_limit::auth_rate_limit_layer())
}
