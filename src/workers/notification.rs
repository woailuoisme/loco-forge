//! # 异步通知队列后台 Worker (NotificationWorker)
//!
//! 该 Worker 负责从后台队列中消费异步通知任务，支持通过邮件 (Email)、Telegram
//! 以及自定义渠道 (Custom Channel) 发送各类通知消息，实现业务逻辑与消息投递的解耦。

use crate::common::notifications::{
    Channel,
    channels::{EmailChannel, TelegramChannel},
    messages::{MailMessage, TelegramMessage},
};
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};

/// 通知处理后台 Worker (NotificationWorker)
pub struct NotificationWorker {
    pub ctx: AppContext,
}

/// 队列作业参数对象 (NotificationJobArgs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationJobArgs {
    /// 目标发送渠道
    pub channel: Channel,
    /// 接收路由目标 (邮箱 / Telegram Chat ID)
    pub recipient: String,
    /// 邮件消息包
    pub mail_message: Option<MailMessage>,
    /// Telegram 消息包
    pub telegram_message: Option<TelegramMessage>,
}

#[async_trait]
impl BackgroundWorker<NotificationJobArgs> for NotificationWorker {
    fn build(ctx: &AppContext) -> Self {
        Self { ctx: ctx.clone() }
    }

    async fn perform(&self, args: NotificationJobArgs) -> Result<()> {
        match args.channel {
            Channel::Email => {
                if let Some(msg) = &args.mail_message {
                    EmailChannel::send(&self.ctx, &args.recipient, msg).await?;
                } else {
                    tracing::warn!("后台队列 Email Job 跳过: 未提供 MailMessage 内容");
                }
            },
            Channel::Telegram => {
                if let Some(msg) = &args.telegram_message {
                    TelegramChannel::send(&self.ctx, &args.recipient, msg).await?;
                } else {
                    tracing::warn!("后台队列 Telegram Job 跳过: 未提供 TelegramMessage 内容");
                }
            },
            Channel::Custom(name) => {
                tracing::info!("自定义后台队列通知渠道暂未配置处理器: {}", name);
            },
        }
        Ok(())
    }
}
