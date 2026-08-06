//! # 仿 Laravel Notification 通知系统模块
//!
//! 提供 Email 与 Telegram 等多渠道通知分发、强类型消息构建器 (`MailMessage`, `TelegramMessage`)，
//! 以及 `Notifiable` / `AnonymousNotifiable` 路由支持。
//!
//! ## 示例 (Example)
//! ```rust,ignore
//! use locos::common::notifications::{
//!     AnonymousNotifiable, Channel, Notification, NotificationManager,
//!     MailMessage, TelegramMessage,
//! };
//!
//! struct WelcomeNotification;
//!
//! impl Notification for WelcomeNotification {
//!     fn via(&self) -> Vec<Channel> {
//!         vec![Channel::Email, Channel::Telegram]
//!     }
//!
//!     fn to_mail(&self) -> Option<MailMessage> {
//!         Some(MailMessage::new().subject("欢迎注册").body("感谢您的加入！"))
//!     }
//!
//!     fn to_telegram(&self) -> Option<TelegramMessage> {
//!         Some(TelegramMessage::new("🎉 欢迎新用户加入系统！").html())
//!     }
//! }
//!
//! // 匿名发送给指定邮箱和 Chat ID
//! let target = AnonymousNotifiable::new()
//!     .route_mail("user@example.com")
//!     .route_telegram("12345678");
//!
//! NotificationManager::send(&ctx, &target, &WelcomeNotification).await?;
//! ```

pub mod channel;
pub mod channels;
pub mod config;
pub mod messages;
pub mod notifiable;
pub mod notification;

pub use channel::Channel;
pub use channels::{EmailChannel, TelegramChannel};
pub use config::TelegramConfig;
pub use messages::{MailMessage, TelegramInlineButton, TelegramMessage, TelegramParseMode};
pub use notifiable::{AnonymousNotifiable, Notifiable};
pub use notification::{Notification, NotificationManager};
