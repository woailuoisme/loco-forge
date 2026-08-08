use loco_rs::{task, testing::prelude::*};
use locos::{app::App, models::users};
use uuid::Uuid;

use loco_rs::boot::run_task;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_can_run_user_create() {
    let boot = boot_test::<App>().await.unwrap();

    // 使用带加号的唯一邮箱（test+<uuid>@example.com），既满足邮箱格式校验，
    // 又避免与其他测试/历史运行共享持久化测试库导致的冲突。
    let email = format!("test+{}@example.com", Uuid::new_v4().simple());
    let user = users::Model::find_by_email(&boot.app_context.db, &email).await;
    assert!(user.is_err());

    let vars = task::Vars::from_cli_args(vec![
        ("email".to_string(), email.to_string()),
        ("name".to_string(), "Test User".to_string()),
        ("password".to_string(), "securepassword".to_string()),
    ]);
    assert!(
        run_task::<App>(&boot.app_context, Some(&"user:create".to_string()), &vars)
            .await
            .is_ok()
    );

    let deliveries = boot.app_context.mailer.unwrap().deliveries();
    // 仅校验本次创建触发了欢迎邮件，不依赖全局 mailer 投递计数（会随其他测试累积）。
    assert!(
        deliveries.messages.iter().any(|m| m.contains(&email)),
        "welcome email for {email} should be delivered"
    );

    let user = users::Model::find_by_email(&boot.app_context.db, &email).await;
    assert!(user.is_ok());
}

#[tokio::test]
#[serial]
async fn test_user_email_already_exists() {
    let boot = boot_test::<App>().await.unwrap();
    seed::<App>(&boot.app_context).await.unwrap();

    let email = "user1@example.com";

    let vars = task::Vars::from_cli_args(vec![
        ("email".to_string(), email.to_string()),
        ("name".to_string(), "Test User".to_string()),
        ("password".to_string(), "securepassword".to_string()),
    ]);
    let err = run_task::<App>(&boot.app_context, Some(&"user:create".to_string()), &vars)
        .await
        .expect_err("err");

    assert!(
        err.to_string().contains("Entity already exists"),
        "error should mention duplicate entity, got: {err}"
    );

    let deliveries = boot.app_context.mailer.unwrap().deliveries();
    // 重复创建不应发送欢迎邮件（不依赖全局计数，避免其他测试累积投递造成误报）。
    assert!(
        !deliveries.messages.iter().any(|m| m.contains(email)),
        "no welcome email should be sent for duplicate {email}"
    );
}
