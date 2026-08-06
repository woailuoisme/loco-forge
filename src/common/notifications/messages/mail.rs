//! # 邮件消息构建器 (Mail Message Builder)

use serde::{Deserialize, Serialize};

/// 邮件通知消息对象 (MailMessage)
///
/// 支持明文正文、HTML 正文以及 Loco/Tera 模版渲染模式。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MailMessage {
    /// 邮件主题
    pub subject: String,
    /// 邮件纯文本正文 (Plain Text)
    pub body: Option<String>,
    /// 邮件 HTML 正文
    pub html: Option<String>,
    /// 邮件模版路径 (用于 Tera 模版渲染)
    pub template: Option<String>,
    /// 渲染模版所需的数据上下文
    pub data: Option<serde_json::Value>,
    /// 发件人自定义 Tag 或分类
    pub tags: Vec<String>,
}

impl MailMessage {
    /// 创建一个新的 MailMessage 实例
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置邮件主题 (Subject)
    pub fn subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = subject.into();
        self
    }

    /// 设置纯文本正文 (Plain Text Body)
    pub fn body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    /// 设置 HTML 正文
    pub fn html(mut self, html: impl Into<String>) -> Self {
        self.html = Some(html.into());
        self
    }

    /// 设置模版名称与上下文数据 (Tera Template)
    pub fn template(mut self, template: impl Into<String>, data: serde_json::Value) -> Self {
        self.template = Some(template.into());
        self.data = Some(data);
        self
    }

    /// 添加自定义标签 (Tag)
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }
}
