//! # 通知中心总线深门面 (NotificationHub Deep Facade)
//!
//! 统一聚合 Mailer (Tera 邮件模板)、Telegram Bot 与后台异步队列 Worker，
//! 对外提供极简的通知调度与认证邮件分发接口。

use loco_rs::prelude::*;

use super::{
    notifiable::Notifiable,
    notification::{Notification, NotificationManager},
};
use crate::{mailers::auth::AuthMailer, models::users};

/// 全局统一通知总线门面
pub struct NotificationHub;

impl NotificationHub {
    /// 同步立即发送多渠道通知
    ///
    /// # Errors
    ///
    /// 当任一指定通道发送失败时返回 `Result`
    pub async fn send<N, T>(ctx: &AppContext, notifiable: &N, notification: &T) -> Result<()>
    where
        N: Notifiable + ?Sized,
        T: Notification + ?Sized,
    {
        NotificationManager::send(ctx, notifiable, notification).await
    }

    /// 异步将通知推入 Background Worker 队列削峰执行
    ///
    /// # Errors
    ///
    /// 当推入队列失败时返回 `Result`
    pub async fn send_async<N, T>(ctx: &AppContext, notifiable: &N, notification: &T) -> Result<()>
    where
        N: Notifiable + ?Sized,
        T: Notification + ?Sized,
    {
        NotificationManager::send_queued(ctx, notifiable, notification).await
    }

    /// 发送注册欢迎与邮箱验证邮件
    ///
    /// # Errors
    ///
    /// 当邮件发送失败时返回 `Result`
    pub async fn send_welcome(ctx: &AppContext, user: &users::Model) -> Result<()> {
        AuthMailer::send_welcome(ctx, user).await
    }

    /// 发送忘记密码重置邮件
    ///
    /// # Errors
    ///
    /// 当邮件发送失败时返回 `Result`
    pub async fn send_forgot_password(ctx: &AppContext, user: &users::Model) -> Result<()> {
        AuthMailer::forgot_password(ctx, user).await
    }

    /// 发送 6 位数字 OTP 登录验证码邮件
    ///
    /// # Errors
    ///
    /// 当邮件发送失败时返回 `Result`
    pub async fn send_otp(ctx: &AppContext, user: &users::Model, code: &str) -> Result<()> {
        AuthMailer::send_otp(ctx, user, code).await
    }
}
