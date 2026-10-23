use loco_rs::{task, testing::prelude::*};
use locos::{app::App, models::user};
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
    let user = user::User::find_by_email(&boot.app_context.db, &email)
        .await
        .unwrap();
    assert!(user.is_none());

    let vars = task::Vars::from_cli_args(vec![
        ("email".to_string(), email.clone()),
        ("name".to_string(), "Test User".to_string()),
        ("password".to_string(), "securepassword".to_string()),
    ]);
    run_task::<App>(&boot.app_context, Some(&"user:create".to_string()), &vars)
        .await
        .unwrap();

    let user = user::User::find_by_email(&boot.app_context.db, &email)
        .await
        .unwrap();
    assert!(user.is_some());
}

#[tokio::test]
#[serial]
async fn test_user_email_already_exists() {
    let boot = boot_test::<App>().await.unwrap();

    let email = format!("user+{}@example.com", Uuid::new_v4().simple());

    let vars = task::Vars::from_cli_args(vec![
        ("email".to_string(), email.clone()),
        ("name".to_string(), "Test User".to_string()),
        ("password".to_string(), "securepassword".to_string()),
    ]);
    // 第一次创建成功
    run_task::<App>(&boot.app_context, Some(&"user:create".to_string()), &vars)
        .await
        .unwrap();

    // 第二次创建相同邮箱，应返回错误
    let err = run_task::<App>(&boot.app_context, Some(&"user:create".to_string()), &vars)
        .await
        .expect_err("err");

    assert!(
        err.to_string().contains("already exists"),
        "error should mention duplicate entity, got: {err}"
    );
}
