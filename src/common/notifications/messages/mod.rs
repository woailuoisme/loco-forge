pub mod mail;
pub mod telegram;

pub use mail::MailMessage;
pub use telegram::{TelegramInlineButton, TelegramMessage, TelegramParseMode};
