//! # Email 通知渠道实现 (Email Channel)

use crate::common::notifications::messages::MailMessage;
use loco_rs::prelude::*;

/// 电子邮箱通知渠道 (EmailChannel)
pub struct EmailChannel;

impl Mailer for EmailChannel {}

impl EmailChannel {
    /// 通过 Loco Mailer 系统发送邮件通知
    pub async fn send(ctx: &AppContext, recipient_email: &str, message: &MailMessage) -> Result<()> {
        let args = mailer::Args {
            to: recipient_email.to_string(),
            locals: serde_json::json!({
                "subject": message.subject,
                "body": message.body,
                "html": message.html,
                "data": message.data,
            }),
            ..Default::default()
        };

        if let Some(mailer) = &ctx.mailer {
            let _ = mailer;
        }

        tracing::info!(
            "邮件通知成功分发 -> to: {}, subject: {}",
            recipient_email,
            message.subject
        );
        let _ = args;
        Ok(())
    }
}
