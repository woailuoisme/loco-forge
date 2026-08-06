//! # 通知渠道定义 (Notification Channels)

use serde::{Deserialize, Serialize};

/// 通知支持的渠道类型 (Channel)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Channel {
    /// 电子邮件渠道
    Email,
    /// Telegram Bot 消息渠道
    Telegram,
    /// 自定义渠道类型
    Custom(String),
}
