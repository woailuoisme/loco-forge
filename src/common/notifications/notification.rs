//! # 核心 Notification Trait 与分发管理器 (Notification & NotificationManager)

use super::{
    channel::Channel,
    channels::{EmailChannel, TelegramChannel},
    messages::{MailMessage, TelegramMessage},
    notifiable::Notifiable,
};
use async_trait::async_trait;
use loco_rs::prelude::*;

/// 通知基类接口 (Notification Trait)
///
/// 类似 Laravel `Notification` 类，定义指定通知发送的渠道及对应渠道的消息内容。
#[async_trait]
pub trait Notification: Send + Sync {
    /// 定义此通知发送的目标渠道列表 (例如: vec![Channel::Email, Channel::Telegram])
    fn via(&self) -> Vec<Channel>;

    /// 构造 Email 渠道的消息主体 (可选实现)
    fn to_mail(&self) -> Option<MailMessage> {
        None
    }

    /// 构造 Telegram 渠道的消息主体 (可选实现)
    fn to_telegram(&self) -> Option<TelegramMessage> {
        None
    }
}

/// 通知分发调度器 (NotificationManager)
pub struct NotificationManager;

impl NotificationManager {
    /// 立即同步/异步分发通知给指定的 Notifiable 目标
    ///
    /// 依次遍历 `notification.via()` 定义的所有渠道进行分发。
    ///
    /// # 示例 (Example)
    /// ```rust,ignore
    /// NotificationManager::send(&ctx, &user, &my_notification).await?;
    /// ```
    pub async fn send<N, T>(ctx: &AppContext, notifiable: &N, notification: &T) -> Result<()>
    where
        N: Notifiable + ?Sized,
        T: Notification + ?Sized,
    {
        let channels = notification.via();

        for channel in channels {
            match channel {
                Channel::Email => {
                    let Some(mail_msg) = notification.to_mail() else {
                        continue;
                    };
                    let Some(email) = notifiable.route_notification_for_mail() else {
                        tracing::warn!("跳过 Email 通知发送: Notifiable 目标未路由邮箱地址");
                        continue;
                    };
                    EmailChannel::send(ctx, &email, &mail_msg).await?;
                }
                Channel::Telegram => {
                    let Some(tg_msg) = notification.to_telegram() else {
                        continue;
                    };
                    let target_chat = notifiable.route_notification_for_telegram().unwrap_or_default();
                    TelegramChannel::send(ctx, &target_chat, &tg_msg).await?;
                }
                Channel::Custom(name) => {
                    tracing::info!("自定义通知渠道暂无扩展处理器: {}", name);
                }
            }
        }

        Ok(())
    }

    /// 将通知入列推送至 Loco Background Worker 在后台线程并发延迟分发
    ///
    /// # 示例 (Example)
    /// ```rust,ignore
    /// NotificationManager::send_queued(&ctx, &user, &my_notification).await?;
    /// ```
    pub async fn send_queued<N, T>(ctx: &AppContext, notifiable: &N, notification: &T) -> Result<()>
    where
        N: Notifiable + ?Sized,
        T: Notification + ?Sized,
    {
        let channels = notification.via();

        for channel in channels {
            match channel {
                Channel::Email => {
                    let Some(mail_msg) = notification.to_mail() else {
                        continue;
                    };
                    let Some(email) = notifiable.route_notification_for_mail() else {
                        continue;
                    };
                    let job_args = crate::workers::notification::NotificationJobArgs {
                        channel: Channel::Email,
                        recipient: email,
                        mail_message: Some(mail_msg),
                        telegram_message: None,
                    };
                    crate::workers::notification::NotificationWorker::perform_later(ctx, job_args).await?;
                }
                Channel::Telegram => {
                    let Some(tg_msg) = notification.to_telegram() else {
                        continue;
                    };
                    let target_chat = notifiable.route_notification_for_telegram().unwrap_or_default();
                    let job_args = crate::workers::notification::NotificationJobArgs {
                        channel: Channel::Telegram,
                        recipient: target_chat,
                        mail_message: None,
                        telegram_message: Some(tg_msg),
                    };
                    crate::workers::notification::NotificationWorker::perform_later(ctx, job_args).await?;
                }
                Channel::Custom(name) => {
                    tracing::info!("自定义通知渠道暂无队列分发: {}", name);
                }
            }
        }

        Ok(())
    }
}
