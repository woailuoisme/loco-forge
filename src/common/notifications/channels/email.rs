//! # Email 通知渠道实现 (Email Channel)

use crate::common::notifications::messages::MailMessage;
use loco_rs::prelude::*;

/// 电子邮箱通知渠道 (EmailChannel)
pub struct EmailChannel;

impl Mailer for EmailChannel {}

impl EmailChannel {
    /// 通过 Loco Mailer 系统发送邮件通知
    pub async fn send(ctx: &AppContext, recipient_email: &str, message: &MailMessage) -> Result<()> {
        let email = mailer::Email {
            to: recipient_email.to_string(),
            subject: message.subject.clone(),
            text: message.body.clone().unwrap_or_default(),
            html: message.html.clone().unwrap_or_default(),
            ..Default::default()
        };

        EmailChannel::mail(ctx, &email).await?;

        tracing::info!(
            "邮件通知成功分发 -> to: {}, subject: {}",
            recipient_email,
            message.subject
        );
        Ok(())
    }
}
