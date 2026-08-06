//! # Telegram Bot 通知渠道实现 (Telegram Channel)

use crate::common::notifications::{config::TelegramConfig, messages::TelegramMessage};
use loco_rs::prelude::*;
use serde_json::json;
use std::sync::LazyLock;
use std::time::Duration;

// 共享的复用 HTTP 客户端 (支持 HTTP 连接池与 Keep-Alive 复用，避免频繁创建客户端造成 Socket 瓶颈)
static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("Failed to initialize Telegram HTTP client")
});

/// Telegram Bot 消息渠道 (TelegramChannel)
pub struct TelegramChannel;

impl TelegramChannel {
    /// 通过 Telegram Bot API 发送消息通知
    pub async fn send(ctx: &AppContext, default_chat_id: &str, message: &TelegramMessage) -> Result<()> {
        let Some(config) = TelegramConfig::from_ctx(ctx) else {
            tracing::warn!("Telegram Bot 未配置 (缺少 TELEGRAM_BOT_TOKEN 或 YAML 配置)，跳过通知发送");
            return Ok(());
        };

        // 确定最终目标 Chat ID
        let target_chat_id = message
            .chat_id
            .as_deref()
            .or(if default_chat_id.is_empty() {
                None
            } else {
                Some(default_chat_id)
            })
            .or(config.chat_id.as_deref())
            .filter(|id| !id.is_empty())
            .ok_or_else(|| {
                tracing::error!("Telegram 发送失败: 未提供有效的 Chat ID");
                Error::string("Telegram Chat ID is missing")
            })?;

        // 构造发送目标 URL: https://api.telegram.org/bot<token>/sendMessage
        let url = format!(
            "{}/bot{}/sendMessage",
            config.api_url.trim_end_matches('/'),
            config.bot_token
        );

        // 构造 Telegram API 请求 Payload
        let mut payload = json!({
            "chat_id": target_chat_id,
            "text": message.text,
        });

        if let Some(mode) = &message.parse_mode {
            payload["parse_mode"] = json!(mode.as_str());
        }
        if let Some(disable) = message.disable_web_page_preview {
            payload["disable_web_page_preview"] = json!(disable);
        }
        if let Some(silent) = message.disable_notification {
            payload["disable_notification"] = json!(silent);
        }
        if let Some(reply_id) = message.reply_to_message_id {
            payload["reply_to_message_id"] = json!(reply_id);
        }

        // 格式化行内键盘 Inline Keyboard (直接序列化，无开销)
        if !message.inline_keyboard.is_empty() {
            payload["reply_markup"] = json!({ "inline_keyboard": &message.inline_keyboard });
        }

        // 复用全局 HTTP 连接池发送请求
        let response = HTTP_CLIENT.post(&url).json(&payload).send().await.map_err(|e| {
            tracing::error!("Telegram HTTP 请求网络错误 -> chat_id({}): {:?}", target_chat_id, e);
            Error::string(&format!("Telegram request error: {e}"))
        })?;

        if !response.status().is_success() {
            let status = response.status();
            let err_body = response.text().await.unwrap_or_default();
            tracing::error!("Telegram API 响应状态错误 HTTP {}: {}", status, err_body);
            return Err(Error::string(&format!("Telegram API error HTTP {status}: {err_body}")));
        }

        tracing::info!("Telegram 消息成功发送至 -> Chat ID({})", target_chat_id);
        Ok(())
    }
}
