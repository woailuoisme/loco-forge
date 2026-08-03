use crate::{
    mailers::auth::AuthMailer,
    models::{
        _entities::users,
        users::{LoginParams, RegisterParams},
    },
    views::auth::{CurrentResponse, LoginResponse},
};
use loco_rs::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

pub static EMAIL_DOMAIN_RE: OnceLock<Regex> = OnceLock::new();

fn get_allow_email_domain_re() -> &'static Regex {
    EMAIL_DOMAIN_RE.get_or_init(|| Regex::new(r"@example\.com$|@gmail\.com$").expect("Failed to compile regex"))
}

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
    path = "/api/auth/register",
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
    let res = users::Model::create_with_password(&ctx.db, &params).await;

    let user = match res {
        Ok(user) => user,
        Err(err) => {
            tracing::info!(
                message = err.to_string(),
                user_email = &params.email,
                "could not register user",
            );
            return format::json(());
        }
    };

    let user = user.into_active_model().set_email_verification_sent(&ctx.db).await?;

    AuthMailer::send_welcome(&ctx, &user).await?;

    format::json(())
}

/// 邮箱验证
#[utoipa::path(
    get,
    path = "/api/auth/verify/{token}",
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
    let Ok(user) = users::Model::find_by_verification_token(&ctx.db, &token).await else {
        return unauthorized("invalid token");
    };

    if user.email_verified_at.is_some() {
        tracing::info!(pid = user.pid.to_string(), "user already verified");
    } else {
        let active_model = user.into_active_model();
        let user = active_model.verified(&ctx.db).await?;
        tracing::info!(pid = user.pid.to_string(), "user verified");
    }

    format::json(())
}

/// 忘记密码
#[utoipa::path(
    post,
    path = "/api/auth/forgot",
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
    let Ok(user) = users::Model::find_by_email(&ctx.db, &params.email).await else {
        // we don't want to expose our users email. if the email is invalid we still
        // returning success to the caller
        return format::json(());
    };

    let user = user.into_active_model().set_forgot_password_sent(&ctx.db).await?;

    AuthMailer::forgot_password(&ctx, &user).await?;

    format::json(())
}

/// 重置密码
#[utoipa::path(
    post,
    path = "/api/auth/reset",
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
    let Ok(user) = users::Model::find_by_reset_token(&ctx.db, &params.token).await else {
        // we don't want to expose our users email. if the email is invalid we still
        // returning success to the caller
        tracing::info!("reset token not found");

        return format::json(());
    };
    user.into_active_model()
        .reset_password(&ctx.db, &params.password)
        .await?;

    format::json(())
}

/// 用户登录
#[utoipa::path(
    post,
    path = "/api/auth/login",
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
    let Ok(user) = users::Model::find_by_email(&ctx.db, &params.email).await else {
        tracing::debug!(email = params.email, "login attempt with non-existent email");
        return unauthorized("Invalid credentials!");
    };

    let valid = user.verify_password(&params.password);

    if !valid {
        return unauthorized("unauthorized!");
    }

    let jwt_secret = ctx.config.get_jwt_config()?;

    let token = user
        .generate_jwt(&jwt_secret.secret, jwt_secret.expiration)
        .or_else(|_| unauthorized("unauthorized!"))?;

    format::json(LoginResponse::new(&user, &token))
}

/// 获取当前用户信息
#[utoipa::path(
    get,
    path = "/api/auth/current",
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
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    format::json(CurrentResponse::new(&user))
}

/// 请求魔术链接登录
#[utoipa::path(
    post,
    path = "/api/auth/magic-link",
    summary = "请求魔术链接登录",
    description = "向指定邮箱发送免密登录魔术链接",
    request_body = MagicLinkParams,
    responses(
        (status = 200, description = "魔术链接发送请求已受理")
    ),
    tag = "auth"
)]
pub async fn magic_link(State(ctx): State<AppContext>, Json(params): Json<MagicLinkParams>) -> Result<Response> {
    let email_regex = get_allow_email_domain_re();
    if !email_regex.is_match(&params.email) {
        tracing::debug!(
            email = params.email,
            "The provided email is invalid or does not match the allowed domains"
        );
        return bad_request("invalid request");
    }

    let Ok(user) = users::Model::find_by_email(&ctx.db, &params.email).await else {
        // we don't want to expose our users email. if the email is invalid we still
        // returning success to the caller
        tracing::debug!(email = params.email, "user not found by email");
        return format::empty_json();
    };

    let user = user.into_active_model().create_magic_link(&ctx.db).await?;
    AuthMailer::send_magic_link(&ctx, &user).await?;

    format::empty_json()
}

/// 验证魔术链接
#[utoipa::path(
    get,
    path = "/api/auth/magic-link/{token}",
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
    let Ok(user) = users::Model::find_by_magic_token(&ctx.db, &token).await else {
        // we don't want to expose our users email. if the email is invalid we still
        // returning success to the caller
        return unauthorized("unauthorized!");
    };

    let user = user.into_active_model().clear_magic_link(&ctx.db).await?;

    let jwt_secret = ctx.config.get_jwt_config()?;

    let token = user
        .generate_jwt(&jwt_secret.secret, jwt_secret.expiration)
        .or_else(|_| unauthorized("unauthorized!"))?;

    format::json(LoginResponse::new(&user, &token))
}

/// 重发邮箱验证邮件
#[utoipa::path(
    post,
    path = "/api/auth/resend-verification-mail",
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
    let Ok(user) = users::Model::find_by_email(&ctx.db, &params.email).await else {
        tracing::info!(email = params.email, "User not found for resend verification");
        return format::json(());
    };

    if user.email_verified_at.is_some() {
        tracing::info!(pid = user.pid.to_string(), "User already verified, skipping resend");
        return format::json(());
    }

    let user = user.into_active_model().set_email_verification_sent(&ctx.db).await?;

    AuthMailer::send_welcome(&ctx, &user).await?;
    tracing::info!(pid = user.pid.to_string(), "Verification email re-sent");

    format::json(())
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/auth")
        .add("/register", post(register))
        .add("/verify/{token}", get(verify))
        .add("/login", post(login))
        .add("/forgot", post(forgot))
        .add("/reset", post(reset))
        .add("/current", get(current))
        .add("/magic-link", post(magic_link))
        .add("/magic-link/{token}", get(magic_link_verify))
        .add("/resend-verification-mail", post(resend_verification_email))
}
