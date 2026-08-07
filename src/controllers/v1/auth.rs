use crate::{
    dtos::common::format_res,
    models::users::{LoginParams, RegisterParams},
    services::AuthService,
    views::auth::{CurrentResponse, LoginResponse},
};
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, utoipa::ToSchema)]
pub struct ForgotParams {
    /// 注册邮箱
    pub email: String,
}

#[derive(Debug, Deserialize, Serialize, utoipa::ToSchema)]
pub struct ResetParams {
    /// 重置密码 Token
    pub token: String,
    /// 新密码
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize, utoipa::ToSchema)]
pub struct MagicLinkParams {
    /// 注册邮箱
    pub email: String,
}

#[derive(Debug, Deserialize, Serialize, utoipa::ToSchema)]
pub struct ResendVerificationParams {
    /// 注册邮箱
    pub email: String,
}

/// 用户注册
#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    summary = "用户注册",
    description = "注册新用户并发送邮箱验证邮件",
    request_body = RegisterParams,
    responses(
        (status = 200, description = "注册请求已受理")
    ),
    tag = "auth"
)]
#[debug_handler]
pub async fn register(State(ctx): State<AppContext>, Json(params): Json<RegisterParams>) -> Result<Response> {
    AuthService::register(&ctx, &params).await?;
    format_res::empty_msg("注册请求已受理")
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
        (status = 200, description = "邮箱验证成功"),
        (status = 401, description = "验证 Token 无效或已过期")
    ),
    tag = "auth"
)]
#[debug_handler]
pub async fn verify(State(ctx): State<AppContext>, Path(token): Path<String>) -> Result<Response> {
    if !AuthService::verify(&ctx, &token).await? {
        return format_res::unauthorized("invalid token");
    }
    format_res::empty_msg("邮箱验证成功")
}

/// 忘记密码
#[utoipa::path(
    post,
    path = "/api/v1/auth/forgot",
    summary = "忘记密码",
    description = "提交邮箱请求发送密码重置邮件",
    request_body = ForgotParams,
    responses(
        (status = 200, description = "重置密码邮件请求已受理")
    ),
    tag = "auth"
)]
#[debug_handler]
pub async fn forgot(State(ctx): State<AppContext>, Json(params): Json<ForgotParams>) -> Result<Response> {
    AuthService::forgot(&ctx, &params.email).await?;
    format_res::empty_msg("重置密码邮件已发送")
}

/// 重置密码
#[utoipa::path(
    post,
    path = "/api/v1/auth/reset",
    summary = "重置密码",
    description = "提交重置 Token 与新密码完成修改",
    request_body = ResetParams,
    responses(
        (status = 200, description = "密码重置成功")
    ),
    tag = "auth"
)]
#[debug_handler]
pub async fn reset(State(ctx): State<AppContext>, Json(params): Json<ResetParams>) -> Result<Response> {
    AuthService::reset(&ctx, &params.token, &params.password).await?;
    format_res::empty_msg("密码重置成功")
}

/// 用户登录
#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    summary = "用户登录",
    description = "校验邮箱密码并返回登录凭证 JWT",
    request_body = LoginParams,
    responses(
        (status = 200, description = "登录成功", body = LoginResponse),
        (status = 401, description = "邮箱或密码错误")
    ),
    tag = "auth"
)]
#[debug_handler]
pub async fn login(State(ctx): State<AppContext>, Json(params): Json<LoginParams>) -> Result<Response> {
    match AuthService::login(&ctx, &params).await? {
        Ok(res) => format_res::ok(res),
        Err(msg) => format_res::unauthorized(msg),
    }
}

/// 获取当前用户信息
#[utoipa::path(
    get,
    path = "/api/v1/auth/current",
    summary = "获取当前用户信息",
    description = "根据 JWT 获取当前登录用户的个人资料",
    responses(
        (status = 200, description = "获取成功", body = CurrentResponse),
        (status = 401, description = "未授权或 Token 无效")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "auth"
)]
#[debug_handler]
pub async fn current(auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    let res = AuthService::current(&ctx, &auth.claims.pid).await?;
    format_res::ok(res)
}

/// 请求魔术链接登录
#[utoipa::path(
    post,
    path = "/api/v1/auth/magic-link",
    summary = "请求魔术链接登录",
    description = "向指定邮箱发送免密登录魔术链接",
    request_body = MagicLinkParams,
    responses(
        (status = 200, description = "魔术链接发送请求已受理")
    ),
    tag = "auth"
)]
pub async fn magic_link(State(ctx): State<AppContext>, Json(params): Json<MagicLinkParams>) -> Result<Response> {
    if !AuthService::magic_link(&ctx, &params.email).await? {
        return format_res::bad_request("invalid request");
    }
    format_res::empty_msg("魔术链接已发送")
}

/// 验证魔术链接
#[utoipa::path(
    get,
    path = "/api/v1/auth/magic-link/{token}",
    summary = "验证魔术链接",
    description = "校验魔术链接 Token 并完成免密登录",
    params(
        ("token" = String, Path, description = "魔术链接 Token")
    ),
    responses(
        (status = 200, description = "验证登录成功", body = LoginResponse),
        (status = 401, description = "魔术链接无效或已过期")
    ),
    tag = "auth"
)]
pub async fn magic_link_verify(Path(token): Path<String>, State(ctx): State<AppContext>) -> Result<Response> {
    let Some(res) = AuthService::magic_link_verify(&ctx, &token).await? else {
        return format_res::unauthorized("unauthorized!");
    };
    format_res::ok(res)
}

/// 重发邮箱验证邮件
#[utoipa::path(
    post,
    path = "/api/v1/auth/resend-verification-mail",
    summary = "重发验证邮件",
    description = "重新向未验证的用户发送验证邮件",
    request_body = ResendVerificationParams,
    responses(
        (status = 200, description = "重发验证邮件成功")
    ),
    tag = "auth"
)]
#[debug_handler]
pub async fn resend_verification_email(
    State(ctx): State<AppContext>,
    Json(params): Json<ResendVerificationParams>,
) -> Result<Response> {
    AuthService::resend_verification(&ctx, &params.email).await?;
    format_res::empty_msg("验证邮件已发送")
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
        .add("/magic-link", post(magic_link))
        .add("/magic-link/{token}", get(magic_link_verify))
        .add("/resend-verification-mail", post(resend_verification_email))
        .layer(crate::setup::middleware::rate_limit::auth_rate_limit_layer())
}
