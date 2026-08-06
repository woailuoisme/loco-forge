//! # Telegram 消息构建器 (Telegram Message Builder)

use serde::{Deserialize, Serialize};

/// Telegram 消息解析模式 (Parse Mode)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TelegramParseMode {
    /// MarkdownV2 语法
    MarkdownV2,
    /// HTML 语法
    HTML,
    /// 传统 Markdown
    Markdown,
}

impl TelegramParseMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MarkdownV2 => "MarkdownV2",
            Self::HTML => "HTML",
            Self::Markdown => "Markdown",
        }
    }
}

/// Telegram 行内按钮 (Inline Keyboard Button)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramInlineButton {
    /// 按钮显示文本
    pub text: String,
    /// 点击后打开的网页 URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// 回调数据 (Callback Data)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callback_data: Option<String>,
}

impl TelegramInlineButton {
    /// 创建带链接的网页按钮
    pub fn url(text: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            url: Some(url.into()),
            callback_data: None,
        }
    }

    /// 创建带回调信号的按钮
    pub fn callback(text: impl Into<String>, callback_data: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            url: None,
            callback_data: Some(callback_data.into()),
        }
    }
}

/// Telegram 消息对象 (TelegramMessage)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TelegramMessage {
    /// 消息文本内容
    pub text: String,
    /// 目标 Chat ID (支持覆盖默认路由的目标)
    pub chat_id: Option<String>,
    /// 解析格式 (MarkdownV2 / HTML)
    pub parse_mode: Option<TelegramParseMode>,
    /// 禁用网页预览
    pub disable_web_page_preview: Option<bool>,
    /// 静音发送 (不弹出通知音)
    pub disable_notification: Option<bool>,
    /// 引用回复的消息 ID
    pub reply_to_message_id: Option<i64>,
    /// 行内键盘按钮二维数组 (Inline Keyboard)
    pub inline_keyboard: Vec<Vec<TelegramInlineButton>>,
}

impl TelegramMessage {
    /// 创建一个新的 TelegramMessage 实例
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            ..Default::default()
        }
    }

    /// 指定或者覆盖目标 Chat ID
    pub fn to_chat(mut self, chat_id: impl Into<String>) -> Self {
        self.chat_id = Some(chat_id.into());
        self
    }

    /// 使用 HTML 格式解析
    pub fn html(mut self) -> Self {
        self.parse_mode = Some(TelegramParseMode::HTML);
        self
    }

    /// 使用 MarkdownV2 格式解析
    pub fn markdown_v2(mut self) -> Self {
        self.parse_mode = Some(TelegramParseMode::MarkdownV2);
        self
    }

    /// 禁用链接预览
    pub fn disable_web_page_preview(mut self, disable: bool) -> Self {
        self.disable_web_page_preview = Some(disable);
        self
    }

    /// 静音发送
    pub fn silent(mut self, silent: bool) -> Self {
        self.disable_notification = Some(silent);
        self
    }

    /// 添加一行行内按钮
    pub fn add_button_row(mut self, row: Vec<TelegramInlineButton>) -> Self {
        self.inline_keyboard.push(row);
        self
    }

    /// 添加单个按钮 (独占一行)
    pub fn add_button(mut self, button: TelegramInlineButton) -> Self {
        self.inline_keyboard.push(vec![button]);
        self
    }
}
