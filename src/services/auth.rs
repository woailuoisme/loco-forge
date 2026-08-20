use crate::{
    common::{consts::auth::ALLOW_EMAIL_DOMAIN_PATTERN, notifications::NotificationHub},
    dtos::auth::{CurrentUserResource, LoginResource},
    errors::{AppError, AppResult},
    models::{
        _entities::users,
        users::{LoginRequest, RegisterRequest},
    },
};
use loco_rs::prelude::*;
use regex::Regex;
use std::sync::OnceLock;

pub static EMAIL_DOMAIN_RE: OnceLock<Regex> = OnceLock::new();

fn get_allow_email_domain_re() -> &'static Regex {
    EMAIL_DOMAIN_RE
        .get_or_init(|| Regex::new(ALLOW_EMAIL_DOMAIN_PATTERN).expect("Failed to compile regex"))
}

pub struct AuthService;

impl AuthService {
    /// 用户注册业务逻辑
    ///
    /// # Errors
    ///
    /// 当发送邮件或更新状态失败时返回 `AppResult`
    pub async fn register(ctx: &AppContext, params: &RegisterRequest) -> AppResult<()> {
        let res = users::Model::create_with_password(&ctx.db, params).await;

        let user = match res {
            Ok(user) => user,
            Err(err) => {
                tracing::info!(
                    message = err.to_string(),
                    user_email = &params.email,
                    "could not register user",
                );
                return Ok(());
            },
        };

        let user = user
            .into_active_model()
            .set_email_verification_sent(&ctx.db)
            .await?;
        NotificationHub::send_welcome(ctx, &user).await?;
        Ok(())
    }

    /// 邮箱验证业务逻辑
    ///
    /// # Errors
    ///
    /// 当 Token 无效时返回 401 Unauthorized
    pub async fn verify(ctx: &AppContext, token: &str) -> AppResult<()> {
        let Ok(user) = users::Model::find_by_verification_token(&ctx.db, token).await else {
            return Err(AppError::unauthorized("Invalid token"));
        };

        if user.email_verified_at.is_some() {
            tracing::info!(pid = user.pid.to_string(), "user already verified");
        } else {
            let active_model = user.into_active_model();
            let user = active_model.verified(&ctx.db).await?;
            tracing::info!(pid = user.pid.to_string(), "user verified");
        }

        Ok(())
    }

    /// 忘记密码业务逻辑
    ///
    /// # Errors
    ///
    /// 当发送重置邮件失败时返回 `AppResult`
    pub async fn forgot(ctx: &AppContext, email: &str) -> AppResult<()> {
        let Ok(user) = users::Model::find_by_email(&ctx.db, email).await else {
            return Ok(());
        };

        let user = user
            .into_active_model()
            .set_forgot_password_sent(&ctx.db)
            .await?;
        NotificationHub::send_forgot_password(ctx, &user).await?;
        Ok(())
    }

    /// 重置密码业务逻辑
    ///
    /// # Errors
    ///
    /// 当重置密码失败时返回 `AppResult`
    pub async fn reset(ctx: &AppContext, token: &str, password: &str) -> AppResult<()> {
        let Ok(user) = users::Model::find_by_reset_token(&ctx.db, token).await else {
            tracing::info!("reset token not found");
            return Ok(());
        };

        user.into_active_model()
            .reset_password(&ctx.db, password)
            .await?;
        Ok(())
    }

    /// 用户登录业务逻辑
    ///
    /// # Errors
    ///
    /// 当邮箱不存在或密码错误时返回 401 Unauthorized
    pub async fn login(ctx: &AppContext, params: &LoginRequest) -> AppResult<LoginResource> {
        use secrecy::ExposeSecret;

        let Ok(user) = users::Model::find_by_email(&ctx.db, &params.email).await else {
            tracing::debug!(
                email = params.email,
                "login attempt with non-existent email"
            );
            return Err(AppError::unauthorized("Invalid credentials"));
        };

        if !user.verify_password(params.password.expose_secret()) {
            return Err(AppError::unauthorized("Unauthorized"));
        }

        let jwt_secret = ctx.config.get_jwt_config()?;
        let Ok(token) = user.generate_jwt(&jwt_secret.secret, jwt_secret.expiration) else {
            return Err(AppError::unauthorized("Unauthorized"));
        };

        Ok(LoginResource::new(&user, &token))
    }

    /// 获取当前登录用户信息
    ///
    /// # Errors
    ///
    /// 当用户不存在时返回 `AppResult`
    pub async fn current(ctx: &AppContext, pid: &str) -> AppResult<CurrentUserResource> {
        let user = users::Model::find_by_pid(&ctx.db, pid).await?;
        Ok(CurrentUserResource::new(&user))
    }

    /// 发送 6 位数字 OTP 验证码业务逻辑
    ///
    /// # Errors
    ///
    /// 当邮箱格式不合规时返回 400 Bad Request
    pub async fn send_otp(ctx: &AppContext, email: &str) -> AppResult<()> {
        let email_regex = get_allow_email_domain_re();
        if !email_regex.is_match(email) {
            tracing::debug!(
                email = email,
                "The provided email is invalid or does not match the allowed domains"
            );
            return Err(AppError::bad_request("Invalid request"));
        }

        let Ok(user) = users::Model::find_by_email(&ctx.db, email).await else {
            tracing::debug!(email = email, "user not found by email");
            return Ok(());
        };

        let (user, code) = user.into_active_model().create_otp(&ctx.db).await?;
        NotificationHub::send_otp(ctx, &user, &code).await?;
        Ok(())
    }

    /// 校验 6 位数字 OTP 验证码业务逻辑
    ///
    /// # Errors
    ///
    /// 当验证码错误或过期时返回 401 Unauthorized
    pub async fn verify_otp(ctx: &AppContext, email: &str, code: &str) -> AppResult<LoginResource> {
        let Ok(user) = users::Model::find_by_otp(&ctx.db, email, code).await else {
            return Err(AppError::unauthorized("Unauthorized"));
        };

        let user = user.into_active_model().clear_otp(&ctx.db).await?;
        let jwt_secret = ctx.config.get_jwt_config()?;
        let Ok(token) = user.generate_jwt(&jwt_secret.secret, jwt_secret.expiration) else {
            return Err(AppError::unauthorized("Unauthorized"));
        };

        Ok(LoginResource::new(&user, &token))
    }

    /// 重发邮箱验证邮件业务逻辑
    ///
    /// # Errors
    ///
    /// 当发送邮件失败时返回 `AppResult`
    pub async fn resend_verification(ctx: &AppContext, email: &str) -> AppResult<()> {
        let Ok(user) = users::Model::find_by_email(&ctx.db, email).await else {
            tracing::info!(email = email, "User not found for resend verification");
            return Ok(());
        };

        if user.email_verified_at.is_some() {
            tracing::info!(
                pid = user.pid.to_string(),
                "User already verified, skipping resend"
            );
            return Ok(());
        }

        let user = user
            .into_active_model()
            .set_email_verification_sent(&ctx.db)
            .await?;
        NotificationHub::send_welcome(ctx, &user).await?;
        tracing::info!(pid = user.pid.to_string(), "Verification email re-sent");

        Ok(())
    }
}
