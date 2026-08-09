//! NotificationWorker 单元测试：验证后台通知任务的多渠道发送行为。
//!
//! 测试使用 test.yaml 中的 stub mailer（内存投递），通过 `ctx.mailer.deliveries()`
//! 断言邮件投递计数，避免依赖真实 SMTP 服务。

use loco_rs::bgworker::BackgroundWorker;
use loco_rs::testing::prelude::*;
use locos::{
    app::App,
    common::notifications::{Channel, MailMessage},
    workers::notification::{NotificationJobArgs, NotificationWorker},
};
use serial_test::serial;

/// Email 渠道应通过 stub mailer 成功投递一封邮件
#[tokio::test]
#[serial]
async fn test_notification_worker_email_delivers() {
    let boot = boot_test::<App>().await.unwrap();
    let worker = NotificationWorker::build(&boot.app_context);

    let args = NotificationJobArgs {
        channel: Channel::Email,
        recipient: "alice@example.com".to_string(),
        mail_message: Some(MailMessage::new().subject("测试邮件").body("正文内容")),
        telegram_message: None,
    };

    let res = worker.perform(args).await;
    assert!(
        res.is_ok(),
        "NotificationWorker email perform should succeed"
    );

    let deliveries = boot.app_context.mailer.unwrap().deliveries();
    assert_eq!(deliveries.count, 1, "应投递一封邮件");
    assert!(
        deliveries
            .messages
            .iter()
            .any(|m| m.contains("alice@example.com")),
        "投递内容应包含收件人地址"
    );
}

/// 缺少 MailMessage 时，Email Job 应跳过且不报错
#[tokio::test]
#[serial]
async fn test_notification_worker_email_skips_without_message() {
    let boot = boot_test::<App>().await.unwrap();
    let worker = NotificationWorker::build(&boot.app_context);

    let args = NotificationJobArgs {
        channel: Channel::Email,
        recipient: "alice@example.com".to_string(),
        mail_message: None,
        telegram_message: None,
    };

    let res = worker.perform(args).await;
    assert!(res.is_ok(), "缺少 MailMessage 时不应报错");

    let deliveries = boot.app_context.mailer.unwrap().deliveries();
    assert_eq!(deliveries.count, 0, "无 MailMessage 时不应投递邮件");
}
