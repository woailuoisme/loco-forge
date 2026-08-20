// auth mailer
#![allow(non_upper_case_globals)]

use loco_rs::prelude::*;
use serde_json::json;

use crate::{common::consts::app::APP_NAME, models::users};

static welcome: Dir<'_> = include_dir!("src/mailers/auth/welcome");
static forgot: Dir<'_> = include_dir!("src/mailers/auth/forgot");
static otp: Dir<'_> = include_dir!("src/mailers/auth/otp");

#[allow(clippy::module_name_repetitions)]
pub struct AuthMailer {}
impl Mailer for AuthMailer {}
impl AuthMailer {
    /// Sending welcome email the the given user
    ///
    /// # Errors
    ///
    /// When email sending is failed
    pub async fn send_welcome(ctx: &AppContext, user: &users::Model) -> Result<()> {
        Self::mail_template(
            ctx,
            &welcome,
            mailer::Args {
                to: user.email.clone(),
                locals: json!({
                  "app_name": APP_NAME,
                  "name": user.name,
                  "verifyToken": user.email_verification_token,
                  "host": ctx.config.server.full_url()
                }),
                ..Default::default()
            },
        )
        .await?;

        Ok(())
    }

    /// Sending forgot password email
    ///
    /// # Errors
    ///
    /// When email sending is failed
    pub async fn forgot_password(ctx: &AppContext, user: &users::Model) -> Result<()> {
        Self::mail_template(
            ctx,
            &forgot,
            mailer::Args {
                to: user.email.clone(),
                locals: json!({
                  "app_name": APP_NAME,
                  "name": user.name,
                  "resetToken": user.reset_token,
                  "host": ctx.config.server.full_url()
                }),
                ..Default::default()
            },
        )
        .await?;

        Ok(())
    }

    /// 发送 6 位数字 OTP 登录验证码邮件
    ///
    /// # Errors
    ///
    /// 当邮件渲染或投递失败时返回 `Result`
    pub async fn send_otp(ctx: &AppContext, user: &users::Model, code: &str) -> Result<()> {
        Self::mail_template(
            ctx,
            &otp,
            mailer::Args {
                to: user.email.clone(),
                locals: json!({
                  "app_name": APP_NAME,
                  "name": user.name,
                  "code": code,
                  "token": code,
                  "host": ctx.config.server.full_url()
                }),
                ..Default::default()
            },
        )
        .await?;

        Ok(())
    }
}
