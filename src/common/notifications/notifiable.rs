//! # 通知目标对象抽象 (Notifiable Trait & Anonymous Notifiable)

use super::channel::Channel;
use std::collections::HashMap;

/// 可接收通知的实体接口 (Notifiable Trait)
///
/// 任何领域模型 (如 `Model` / `User`) 实现此 Trait 即可直接接收各类 Notification。
pub trait Notifiable: Send + Sync {
    /// 获取该实体接收 Email 通知的目标邮箱地址
    fn route_notification_for_mail(&self) -> Option<String> {
        None
    }

    /// 获取该实体接收 Telegram 通知的目标 Chat ID / Channel ID
    fn route_notification_for_telegram(&self) -> Option<String> {
        None
    }

    /// 获取自定义 Channel 的目标路由信息
    fn route_notification_for_custom(&self, _channel: &str) -> Option<String> {
        None
    }
}

/// 匿名通知目标对象 (Anonymous Notifiable)
///
/// 用于无需绑定 User/ORM 数据库实体时，直接点对点发送通知 (仿 Laravel `Notification::route(...)`)。
///
/// # 示例 (Example)
/// ```rust,ignore
/// use locos::common::notifications::{AnonymousNotifiable, Channel};
///
/// let notifiable = AnonymousNotifiable::new()
///     .route_mail("admin@example.com")
///     .route_telegram("-100123456789");
/// ```
#[derive(Debug, Clone, Default)]
pub struct AnonymousNotifiable {
    routes: HashMap<Channel, String>,
}

impl AnonymousNotifiable {
    /// 创建一个新的匿名通知对象
    pub fn new() -> Self {
        Self::default()
    }

    /// 链式绑定某渠道的目标接收地址
    pub fn route(mut self, channel: Channel, target: impl Into<String>) -> Self {
        self.routes.insert(channel, target.into());
        self
    }

    /// 快捷绑定邮箱地址
    pub fn route_mail(self, email: impl Into<String>) -> Self {
        self.route(Channel::Email, email)
    }

    /// 快捷绑定 Telegram Chat ID
    pub fn route_telegram(self, chat_id: impl Into<String>) -> Self {
        self.route(Channel::Telegram, chat_id)
    }
}

impl Notifiable for AnonymousNotifiable {
    fn route_notification_for_mail(&self) -> Option<String> {
        self.routes.get(&Channel::Email).cloned()
    }

    fn route_notification_for_telegram(&self) -> Option<String> {
        self.routes.get(&Channel::Telegram).cloned()
    }

    fn route_notification_for_custom(&self, channel: &str) -> Option<String> {
        self.routes
            .get(&Channel::Custom(channel.to_string()))
            .cloned()
    }
}
