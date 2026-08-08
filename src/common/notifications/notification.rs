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

/// 单渠道分发载荷（已解析的消息与收件人目标）
enum ChannelDispatch {
    Email {
        message: MailMessage,
        recipient: String,
    },
    Telegram {
        message: TelegramMessage,
        recipient: String,
    },
}

/// 通知分发调度器 (NotificationManager)
pub struct NotificationManager;

impl NotificationManager {
    /// 解析指定渠道的消息与收件人目标
    ///
    /// - 消息未实现或收件人缺失时返回 `None`（调用方跳过该渠道）。
    /// - `warn_on_missing_email_route` 控制邮件路由缺失时是否输出告警日志
    ///   （`send` 需要告警，`send_queued` 保持静默跳过，维持原有行为差异）。
    fn resolve_dispatch<N, T>(
        notifiable: &N,
        notification: &T,
        channel: &Channel,
        warn_on_missing_email_route: bool,
    ) -> Option<ChannelDispatch>
    where
        N: Notifiable + ?Sized,
        T: Notification + ?Sized,
    {
        match channel {
            Channel::Email => {
                let message = notification.to_mail()?;
                let recipient = match notifiable.route_notification_for_mail() {
                    Some(recipient) => recipient,
                    None => {
                        if warn_on_missing_email_route {
                            tracing::warn!("跳过 Email 通知发送: Notifiable 目标未路由邮箱地址");
                        }
                        return None;
                    }
                };
                Some(ChannelDispatch::Email { message, recipient })
            }
            Channel::Telegram => {
                let message = notification.to_telegram()?;
                Some(ChannelDispatch::Telegram {
                    message,
                    recipient: notifiable
                        .route_notification_for_telegram()
                        .unwrap_or_default(),
                })
            }
            Channel::Custom(_) => None,
        }
    }

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
        for channel in notification.via() {
            if let Channel::Custom(name) = &channel {
                tracing::info!("自定义通知渠道暂无扩展处理器: {}", name);
                continue;
            }
            let Some(dispatch) = Self::resolve_dispatch(notifiable, notification, &channel, true)
            else {
                continue;
            };
            match dispatch {
                ChannelDispatch::Email { message, recipient } => {
                    EmailChannel::send(ctx, &recipient, &message).await?;
                }
                ChannelDispatch::Telegram { message, recipient } => {
                    TelegramChannel::send(ctx, &recipient, &message).await?;
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
        for channel in notification.via() {
            if let Channel::Custom(name) = &channel {
                tracing::info!("自定义通知渠道暂无队列分发: {}", name);
                continue;
            }
            let Some(dispatch) = Self::resolve_dispatch(notifiable, notification, &channel, false)
            else {
                continue;
            };
            let job_args = match dispatch {
                ChannelDispatch::Email { message, recipient } => {
                    crate::workers::notification::NotificationJobArgs {
                        channel: Channel::Email,
                        recipient,
                        mail_message: Some(message),
                        telegram_message: None,
                    }
                }
                ChannelDispatch::Telegram { message, recipient } => {
                    crate::workers::notification::NotificationJobArgs {
                        channel: Channel::Telegram,
                        recipient,
                        mail_message: None,
                        telegram_message: Some(message),
                    }
                }
            };
            crate::workers::notification::NotificationWorker::perform_later(ctx, job_args).await?;
        }

        Ok(())
    }
}
