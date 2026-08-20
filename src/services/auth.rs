use crate::{
    common::consts::auth::ALLOW_EMAIL_DOMAIN_PATTERN,
    dtos::auth::{CurrentUserResource, LoginResource},
    mailers::auth::AuthMailer,
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
    pub async fn register(ctx: &AppContext, params: &RegisterRequest) -> Result<()> {
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
        AuthMailer::send_welcome(ctx, &user).await?;
        Ok(())
    }

    /// 邮箱验证业务逻辑
    pub async fn verify(ctx: &AppContext, token: &str) -> Result<bool> {
        let Ok(user) = users::Model::find_by_verification_token(&ctx.db, token).await else {
            return Ok(false);
        };

        if user.email_verified_at.is_some() {
            tracing::info!(pid = user.pid.to_string(), "user already verified");
        } else {
            let active_model = user.into_active_model();
            let user = active_model.verified(&ctx.db).await?;
            tracing::info!(pid = user.pid.to_string(), "user verified");
        }

        Ok(true)
    }

    /// 忘记密码业务逻辑
    pub async fn forgot(ctx: &AppContext, email: &str) -> Result<()> {
        let Ok(user) = users::Model::find_by_email(&ctx.db, email).await else {
            return Ok(());
        };

        let user = user
            .into_active_model()
            .set_forgot_password_sent(&ctx.db)
            .await?;
        AuthMailer::forgot_password(ctx, &user).await?;
        Ok(())
    }

    /// 重置密码业务逻辑
    pub async fn reset(ctx: &AppContext, token: &str, password: &str) -> Result<()> {
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
    pub async fn login(
        ctx: &AppContext,
        params: &LoginRequest,
    ) -> Result<Result<LoginResource, &'static str>> {
        use secrecy::ExposeSecret;

        let Ok(user) = users::Model::find_by_email(&ctx.db, &params.email).await else {
            tracing::debug!(
                email = params.email,
                "login attempt with non-existent email"
            );
            return Ok(Err("Invalid credentials"));
        };

        if !user.verify_password(params.password.expose_secret()) {
            return Ok(Err("Unauthorized"));
        }

        let jwt_secret = ctx.config.get_jwt_config()?;
        let Ok(token) = user.generate_jwt(&jwt_secret.secret, jwt_secret.expiration) else {
            return Ok(Err("Unauthorized"));
        };

        Ok(Ok(LoginResource::new(&user, &token)))
    }

    /// 获取当前登录用户信息
    pub async fn current(ctx: &AppContext, pid: &str) -> Result<CurrentUserResource> {
        let user = users::Model::find_by_pid(&ctx.db, pid).await?;
        Ok(CurrentUserResource::new(&user))
    }

    /// 请求魔术链接登录业务逻辑
    pub async fn magic_link(ctx: &AppContext, email: &str) -> Result<bool> {
        let email_regex = get_allow_email_domain_re();
        if !email_regex.is_match(email) {
            tracing::debug!(
                email = email,
                "The provided email is invalid or does not match the allowed domains"
            );
            return Ok(false);
        }

        let Ok(user) = users::Model::find_by_email(&ctx.db, email).await else {
            tracing::debug!(email = email, "user not found by email");
            return Ok(true);
        };

        let user = user.into_active_model().create_magic_link(&ctx.db).await?;
        AuthMailer::send_magic_link(ctx, &user).await?;
        Ok(true)
    }

    /// 验证魔术链接登录业务逻辑
    pub async fn magic_link_verify(ctx: &AppContext, token: &str) -> Result<Option<LoginResource>> {
        let Ok(user) = users::Model::find_by_magic_token(&ctx.db, token).await else {
            return Ok(None);
        };

        let user = user.into_active_model().clear_magic_link(&ctx.db).await?;
        let jwt_secret = ctx.config.get_jwt_config()?;
        let Ok(token) = user.generate_jwt(&jwt_secret.secret, jwt_secret.expiration) else {
            return Ok(None);
        };

        Ok(Some(LoginResource::new(&user, &token)))
    }

    /// 重发邮箱验证邮件业务逻辑
    pub async fn resend_verification(ctx: &AppContext, email: &str) -> Result<()> {
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
        AuthMailer::send_welcome(ctx, &user).await?;
        tracing::info!(pid = user.pid.to_string(), "Verification email re-sent");

        Ok(())
    }
}
