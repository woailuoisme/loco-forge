use loco_rs::testing::prelude::*;
use locos::{
    app::App,
    models::{account::Account, session::Session, user::User},
};
use serial_test::serial;
use uuid::Uuid;

#[tokio::test]
#[serial]
async fn test_user_creation_and_lookup() {
    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    let email = format!("user_{}@example.com", Uuid::new_v4().simple());
    let username = format!("user_{}", &Uuid::new_v4().simple().to_string()[..8]);

    let created = User::create_user(
        &boot.app_context.db,
        &email,
        "Test User",
        None,
        None,
        Some(username.clone()),
    )
    .await
    .expect("create user");

    assert_eq!(created.email, email);
    assert_eq!(created.name, "Test User");
    assert_eq!(created.username, Some(username.clone()));
    assert!(!created.is_admin());

    let found_by_email = User::find_by_email(&boot.app_context.db, &email)
        .await
        .expect("find by email")
        .expect("user exists");
    assert_eq!(found_by_email.id, created.id);

    let found_by_username = User::find_by_username(&boot.app_context.db, &username)
        .await
        .expect("find by username")
        .expect("user exists");
    assert_eq!(found_by_username.id, created.id);

    let found_by_id = User::find_by_id(&boot.app_context.db, &created.id)
        .await
        .expect("find by id")
        .expect("user exists");
    assert_eq!(found_by_id.email, email);
}

#[tokio::test]
#[serial]
async fn test_anonymous_user_creation() {
    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    let anon = User::create_anonymous(&boot.app_context.db, None)
        .await
        .expect("create anonymous user");

    assert!(anon.email.starts_with("anon_"));
    assert!(anon.email.ends_with("@anonymous.local"));
    assert!(anon.name.starts_with("Anonymous "));
    assert!(!anon.email_verified);
    assert!(anon.is_anonymous);

    let named_anon = User::create_anonymous(&boot.app_context.db, Some("Custom Anon"))
        .await
        .expect("create named anonymous user");
    assert_eq!(named_anon.name, "Custom Anon");
    assert!(named_anon.is_anonymous);
}

#[tokio::test]
#[serial]
async fn test_user_ban_management() {
    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    let email = format!("ban_test_{}@example.com", Uuid::new_v4().simple());
    let created = User::create_user(&boot.app_context.db, &email, "Ban Target", None, None, None)
        .await
        .expect("create user");

    assert!(!created.banned);

    let banned = created
        .set_banned(&boot.app_context.db, true, Some("Violation of TOS"), None)
        .await
        .expect("set banned");

    assert!(banned.banned);
    assert_eq!(banned.ban_reason, Some("Violation of TOS".to_string()));

    let unbanned = banned
        .set_banned(&boot.app_context.db, false, None, None)
        .await
        .expect("unban user");
    assert!(!unbanned.banned);
    assert_eq!(unbanned.ban_reason, None);
}

#[tokio::test]
#[serial]
async fn test_account_password_and_verification() {
    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    let email = format!("acc_{}@example.com", Uuid::new_v4().simple());
    let user = User::create_user(&boot.app_context.db, &email, "Acc User", None, None, None)
        .await
        .expect("create user");

    let account =
        Account::create_credential(&boot.app_context.db, &user.id, &user.id, "super_secret_pw")
            .await
            .expect("create account");

    assert_eq!(account.user_id, user.id);
    assert_eq!(account.provider_id, "credential");
    assert!(account.verify_password("super_secret_pw"));
    assert!(!account.verify_password("wrong_password"));

    let found = Account::find_credential_by_user(&boot.app_context.db, &user.id)
        .await
        .expect("find account")
        .expect("account exists");
    assert_eq!(found.id, account.id);
}

#[tokio::test]
#[serial]
async fn test_session_lifecycle_and_revocation() {
    let boot = boot_test::<App>()
        .await
        .expect("Failed to boot test application");

    let email = format!("sess_{}@example.com", Uuid::new_v4().simple());
    let user = User::create_user(&boot.app_context.db, &email, "Sess User", None, None, None)
        .await
        .expect("create user");

    let sess1 = Session::create_session(
        &boot.app_context.db,
        &user.id,
        Some("127.0.0.1".into()),
        Some("Mozilla/5.0 Test".into()),
    )
    .await
    .expect("create session 1");

    let sess2 = Session::create_session(
        &boot.app_context.db,
        &user.id,
        Some("127.0.0.1".into()),
        Some("Mozilla/5.0 Test".into()),
    )
    .await
    .expect("create session 2");

    assert!(!sess1.is_expired());
    assert!(!sess2.is_expired());

    // 查找会话
    let found1 = Session::find_valid_by_token(&boot.app_context.db, &sess1.token)
        .await
        .expect("find session")
        .expect("session exists");
    assert_eq!(found1.id, sess1.id);

    // 查询所有活跃会话
    let active_sessions = Session::find_active_by_user(&boot.app_context.db, &user.id)
        .await
        .expect("find all");
    assert_eq!(active_sessions.len(), 2);

    // 吊销其他会话 (保留 sess1)
    let revoked = Session::revoke_other_sessions(&boot.app_context.db, &user.id, &sess1.token)
        .await
        .expect("revoke other sessions");
    assert_eq!(revoked, 1);

    assert!(
        Session::find_valid_by_token(&boot.app_context.db, &sess2.token)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        Session::find_valid_by_token(&boot.app_context.db, &sess1.token)
            .await
            .unwrap()
            .is_some()
    );

    // 单个会话吊销
    let revoked_single = Session::revoke_by_token(&boot.app_context.db, &sess1.token)
        .await
        .expect("revoke single");
    assert!(revoked_single);
    assert!(
        Session::find_valid_by_token(&boot.app_context.db, &sess1.token)
            .await
            .unwrap()
            .is_none()
    );
}
