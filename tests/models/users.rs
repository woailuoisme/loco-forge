use chrono::{Duration, offset::Local};
use insta::assert_debug_snapshot;
use loco_rs::testing::prelude::*;
use locos::{
    app::App,
    models::users::{self, Model, RegisterRequest},
};
use sea_orm::{ActiveModelTrait, ActiveValue, IntoActiveModel};
use secrecy::SecretString;
use serial_test::serial;

use crate::configure_insta;

#[tokio::test]
#[serial]
async fn test_can_validate_model() {
    configure_insta!("users");

    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    let invalid_user = users::ActiveModel {
        name: ActiveValue::set("1".to_string()),
        email: ActiveValue::set("invalid-email".to_string()),
        ..Default::default()
    };

    let res = invalid_user.insert(&boot.app_context.db).await;

    assert_debug_snapshot!(res);
}

#[tokio::test]
#[serial]
async fn can_create_with_password() {
    configure_insta!();

    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    let params = RegisterRequest {
        email: "test@framework.com".to_string(),
        password: secrecy::SecretString::from("1234"),
        name: "framework".to_string(),
    };

    let res = Model::create_with_password(&boot.app_context.db, &params).await;

    insta::with_settings!({
        filters => cleanup_user_model()
    }, {
        assert_debug_snapshot!(res);
    });
}
#[tokio::test]
#[serial]
async fn handle_create_with_password_with_duplicate() {
    configure_insta!();

    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");
    seed::<App>(&boot.app_context)
        .await
        .expect("Failed to seed database");

    let new_user = Model::create_with_password(
        &boot.app_context.db,
        &RegisterRequest {
            email: "user1@example.com".to_string(),
            password: secrecy::SecretString::from("1234"),
            name: "framework".to_string(),
        },
    )
    .await;

    assert_debug_snapshot!(new_user);
}

#[tokio::test]
#[serial]
async fn can_find_by_email() {
    configure_insta!();

    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");
    seed::<App>(&boot.app_context)
        .await
        .expect("Failed to seed database");

    let existing_user = Model::find_by_email(&boot.app_context.db, "user1@example.com").await;
    let non_existing_user_results =
        Model::find_by_email(&boot.app_context.db, "un@existing-email.com").await;

    // 仅快照稳定字段，避免受其他测试修改同用户的影响。
    let stable = existing_user.as_ref().map(|u| {
        (
            u.pid.to_string(),
            u.email.clone(),
            u.name.clone(),
            u.role.clone(),
        )
    });
    assert_debug_snapshot!(stable);
    assert_debug_snapshot!(non_existing_user_results);
}

#[tokio::test]
#[serial]
async fn can_find_by_pid() {
    configure_insta!();

    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");
    seed::<App>(&boot.app_context)
        .await
        .expect("Failed to seed database");

    let existing_user =
        Model::find_by_pid(&boot.app_context.db, "11111111-1111-1111-1111-111111111111").await;
    let non_existing_user_results =
        Model::find_by_pid(&boot.app_context.db, "23232323-2323-2323-2323-232323232323").await;

    // 仅快照稳定字段（pid/email/name/role），避免受其他测试修改同 PID 用户的影响。
    let stable = existing_user.as_ref().map(|u| {
        (
            u.pid.to_string(),
            u.email.clone(),
            u.name.clone(),
            u.role.clone(),
        )
    });
    assert_debug_snapshot!(stable);
    assert_debug_snapshot!(non_existing_user_results);
}

#[tokio::test]
#[serial]
async fn can_verification_token() {
    configure_insta!();

    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");
    seed::<App>(&boot.app_context)
        .await
        .expect("Failed to seed database");

    let user = Model::find_by_pid(&boot.app_context.db, "11111111-1111-1111-1111-111111111111")
        .await
        .expect("Failed to find user by PID");

    let result = user
        .into_active_model()
        .set_email_verification_sent(&boot.app_context.db)
        .await;

    assert!(result.is_ok(), "Failed to set email verification sent");

    let user = Model::find_by_pid(&boot.app_context.db, "11111111-1111-1111-1111-111111111111")
        .await
        .expect("Failed to find user by PID after setting verification sent");

    assert!(
        user.email_verification_sent_at.is_some(),
        "Expected email verification sent timestamp to be present"
    );
    assert!(
        user.email_verification_token.is_some(),
        "Expected email verification token to be present"
    );
}

#[tokio::test]
#[serial]
async fn can_set_forgot_password_sent() {
    configure_insta!();

    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");
    seed::<App>(&boot.app_context)
        .await
        .expect("Failed to seed database");

    let user = Model::find_by_pid(&boot.app_context.db, "11111111-1111-1111-1111-111111111111")
        .await
        .expect("Failed to find user by PID");

    let result = user
        .into_active_model()
        .set_forgot_password_sent(&boot.app_context.db)
        .await;

    assert!(result.is_ok(), "Failed to set forgot password sent");

    let user = Model::find_by_pid(&boot.app_context.db, "11111111-1111-1111-1111-111111111111")
        .await
        .expect("Failed to find user by PID after setting forgot password sent");

    assert!(
        user.reset_sent_at.is_some(),
        "Expected reset sent timestamp to be present"
    );
    assert!(
        user.reset_token.is_some(),
        "Expected reset token to be present"
    );
}

#[tokio::test]
#[serial]
async fn can_verified() {
    configure_insta!();

    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");
    seed::<App>(&boot.app_context)
        .await
        .expect("Failed to seed database");

    let user = Model::find_by_pid(&boot.app_context.db, "11111111-1111-1111-1111-111111111111")
        .await
        .expect("Failed to find user by PID");

    let result = user
        .into_active_model()
        .verified(&boot.app_context.db)
        .await;

    assert!(result.is_ok(), "Failed to mark email as verified");

    let user = Model::find_by_pid(&boot.app_context.db, "11111111-1111-1111-1111-111111111111")
        .await
        .expect("Failed to find user by PID after verification");

    assert!(
        user.email_verified_at.is_some(),
        "Expected email to be verified"
    );
}

#[tokio::test]
#[serial]
async fn can_reset_password() {
    configure_insta!();

    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");
    seed::<App>(&boot.app_context)
        .await
        .expect("Failed to seed database");

    let user = Model::find_by_pid(&boot.app_context.db, "11111111-1111-1111-1111-111111111111")
        .await
        .expect("Failed to find user by PID");

    let result = user
        .clone()
        .into_active_model()
        .reset_password(&boot.app_context.db, "new-password")
        .await;

    assert!(result.is_ok(), "Failed to reset password");

    let user = Model::find_by_pid(&boot.app_context.db, "11111111-1111-1111-1111-111111111111")
        .await
        .expect("Failed to find user by PID after password reset");

    assert!(
        user.verify_password("new-password"),
        "Password verification failed for new password"
    );
}

#[tokio::test]
#[serial]
async fn test_clean_expired_tokens_batch() {
    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");
    seed::<App>(&boot.app_context)
        .await
        .expect("Failed to seed database");

    // 设置过期 reset_token
    let user = Model::find_by_pid(&boot.app_context.db, "11111111-1111-1111-1111-111111111111")
        .await
        .expect("Failed to find user");

    let mut active: users::ActiveModel = user.into();
    let old_time = Local::now() - Duration::hours(2);
    active.reset_token = ActiveValue::Set(Some("expired_token".to_string()));
    active.reset_sent_at = ActiveValue::Set(Some(old_time.into()));
    active
        .update(&boot.app_context.db)
        .await
        .expect("Failed to update user token");

    let cleaned = Model::clean_expired_tokens(&boot.app_context.db)
        .await
        .expect("clean_expired_tokens should succeed");

    assert!(cleaned >= 1, "Should clean at least 1 expired token");

    let refetched =
        Model::find_by_pid(&boot.app_context.db, "11111111-1111-1111-1111-111111111111")
            .await
            .expect("Failed to refetch user");

    assert!(
        refetched.reset_token.is_none(),
        "Expired reset token should be cleared"
    );
}

#[tokio::test]
#[serial]
async fn test_create_and_verify_otp() {
    let boot = boot_test::<App>().await.unwrap();
    seed::<App>(&boot.app_context).await.unwrap();

    let user = Model::find_by_email(&boot.app_context.db, "user1@example.com")
        .await
        .unwrap();

    let (updated_user, code) = user
        .into_active_model()
        .create_otp(&boot.app_context.db)
        .await
        .expect("create_otp should succeed");

    assert_eq!(code.len(), 6, "OTP code should be 6 digits");
    assert!(
        code.chars().all(|c| c.is_ascii_digit()),
        "OTP code should be numeric"
    );

    let found_user = Model::find_by_otp(&boot.app_context.db, "user1@example.com", &code)
        .await
        .expect("find_by_otp should succeed");
    assert_eq!(found_user.id, updated_user.id);

    let cleared_user = found_user
        .into_active_model()
        .clear_otp(&boot.app_context.db)
        .await
        .expect("clear_otp should succeed");
    assert!(cleared_user.magic_link_token.is_none());
    assert!(cleared_user.magic_link_expiration.is_none());
}

#[tokio::test]
#[serial]
async fn test_user_entity_paginated_and_batch_delete() {
    let boot = boot_test::<App>().await.unwrap();
    seed::<App>(&boot.app_context).await.unwrap();

    let (users_list, total) = users::Entity::find_all_paginated(&boot.app_context.db, 1, 10)
        .await
        .expect("find_all_paginated should succeed");

    assert!(total >= 1);
    assert!(!users_list.is_empty());

    // 测试空切片防护
    let deleted_empty = users::Entity::delete_by_ids(&boot.app_context.db, &[])
        .await
        .expect("delete_by_ids with empty slice should succeed");
    assert_eq!(deleted_empty, 0);
}

#[tokio::test]
#[serial]
async fn test_user_credentials_state_machine() {
    let boot = boot_test::<App>().await.unwrap();

    let reg = RegisterRequest {
        name: "State Machine User".to_string(),
        email: "state_machine@example.com".to_string(),
        password: SecretString::from("password123"),
    };
    let user = users::Model::create_with_password(&boot.app_context.db, &reg)
        .await
        .expect("create user failed");

    // 1. Create OTP
    let (user_with_otp, otp) = user
        .into_active_model()
        .create_otp(&boot.app_context.db)
        .await
        .expect("create_otp failed");
    assert_eq!(otp.len(), 6);
    assert_eq!(user_with_otp.magic_link_token, Some(otp.clone()));

    // 2. Clear OTP
    let user_cleared_otp = user_with_otp
        .into_active_model()
        .clear_otp(&boot.app_context.db)
        .await
        .expect("clear_otp failed");
    assert_eq!(user_cleared_otp.magic_link_token, None);

    // 3. Set Email Verification Sent
    let user_with_verify = user_cleared_otp
        .into_active_model()
        .set_email_verification_sent(&boot.app_context.db)
        .await
        .expect("set_email_verification_sent failed");
    assert!(user_with_verify.email_verification_token.is_some());
    assert!(user_with_verify.email_verification_sent_at.is_some());

    // 4. Mark Verified
    let user_verified = user_with_verify
        .into_active_model()
        .verified(&boot.app_context.db)
        .await
        .expect("verified failed");
    assert!(user_verified.email_verified_at.is_some());
    assert_eq!(user_verified.email_verification_token, None);

    // 5. Set Forgot Password Sent
    let user_with_reset = user_verified
        .into_active_model()
        .set_forgot_password_sent(&boot.app_context.db)
        .await
        .expect("set_forgot_password_sent failed");
    assert!(user_with_reset.reset_token.is_some());
    assert!(user_with_reset.reset_sent_at.is_some());

    // 6. Reset Password
    let user_reset = user_with_reset
        .into_active_model()
        .reset_password(&boot.app_context.db, "new_password_123")
        .await
        .expect("reset_password failed");
    assert_eq!(user_reset.reset_token, None);
    assert!(user_reset.verify_password("new_password_123"));
}
