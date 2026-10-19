use axum::http::StatusCode;
use loco_rs::testing::prelude::*;
use locos::{app::App, dtos::auth::*, models::verification};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serial_test::serial;
use uuid::Uuid;

// ─────────────────────────────────────────────────────────────────────────────
// Phase 2 综合集成测试 (Email OTP / Magic Link / Phone / 2FA)
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn test_email_otp_lifecycle_send_check_signin_verify_reset() {
    request::<App, _, _>(|request, ctx| async move {
        let email = format!("emailotp_{}@example.com", Uuid::new_v4().simple());

        // 1. 发送登录验证码
        let send_res = request
            .post("/v1/verification-codes")
            .json(&serde_json::json!({
                "email": email,
                "type": "sign-in"
            }))
            .await;
        assert_eq!(send_res.status_code(), StatusCode::OK);
        let status_res: StatusResource = serde_json::from_str(&send_res.text()).unwrap();
        assert!(status_res.status);

        // 2. 从数据库读取生成的验证码
        let identifier = format!("email-otp:sign-in:{email}");
        let record = verification::Entity::find()
            .filter(verification::Column::Identifier.eq(&identifier))
            .one(&ctx.db)
            .await
            .unwrap()
            .expect("OTP record must exist in DB");
        let otp = record.value;
        assert_eq!(otp.len(), 6);

        // 3. check-verification-otp (预检不消耗)
        let check_bad = request
            .post("/v1/verification-code-checks")
            .json(&serde_json::json!({
                "email": email,
                "type": "sign-in",
                "otp": "000000"
            }))
            .await;
        assert_eq!(check_bad.status_code(), StatusCode::BAD_REQUEST);

        let check_good = request
            .post("/v1/verification-code-checks")
            .json(&serde_json::json!({
                "email": email,
                "type": "sign-in",
                "otp": otp
            }))
            .await;
        assert_eq!(check_good.status_code(), StatusCode::OK);

        // 4. sign-in/email-otp (首次使用自动注册并签发会话)
        let signin_res = request
            .post("/v1/sessions/email-otp")
            .json(&serde_json::json!({
                "email": email,
                "otp": otp,
                "name": "OTP User"
            }))
            .await;
        assert_eq!(signin_res.status_code(), StatusCode::OK);
        let auth: AuthResource = serde_json::from_str(&signin_res.text()).unwrap();
        assert_eq!(auth.user.email, email);
        assert!(!auth.token.is_empty());
        assert!(auth.user.email_verified);

        // 5. 验证邮箱 OTP (email-verification)
        let send_verify = request
            .post("/v1/verification-codes")
            .json(&serde_json::json!({
                "email": email,
                "type": "email-verification"
            }))
            .await;
        assert_eq!(send_verify.status_code(), StatusCode::OK);

        let verify_id = format!("email-otp:email-verification:{email}");
        let verify_rec = verification::Entity::find()
            .filter(verification::Column::Identifier.eq(&verify_id))
            .one(&ctx.db)
            .await
            .unwrap()
            .unwrap();

        let verify_res = request
            .post("/v1/email-verifications")
            .json(&serde_json::json!({
                "email": email,
                "otp": verify_rec.value
            }))
            .await;
        assert_eq!(verify_res.status_code(), StatusCode::OK);
        let verified: VerifyEmailOtpResource = serde_json::from_str(&verify_res.text()).unwrap();
        assert!(verified.status);
        assert!(verified.user.email_verified);

        // 6. 重置密码 OTP (forget-password)
        let send_fp = request
            .post("/v1/verification-codes")
            .json(&serde_json::json!({
                "email": email,
                "type": "forget-password"
            }))
            .await;
        assert_eq!(send_fp.status_code(), StatusCode::OK);

        let fp_id = format!("email-otp:forget-password:{email}");
        let fp_rec = verification::Entity::find()
            .filter(verification::Column::Identifier.eq(&fp_id))
            .one(&ctx.db)
            .await
            .unwrap()
            .unwrap();

        let new_password = "new_strong_password_456";
        let reset_res = request
            .patch("/v1/users/me/password")
            .json(&serde_json::json!({
                "email": email,
                "otp": fp_rec.value,
                "password": new_password
            }))
            .await;
        assert_eq!(reset_res.status_code(), StatusCode::OK);

        // 使用新密码登录检验
        let login_res = request
            .post("/v1/sessions/email")
            .json(&serde_json::json!({
                "email": email,
                "password": new_password
            }))
            .await;
        assert_eq!(login_res.status_code(), StatusCode::OK);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn test_phone_number_flow() {
    request::<App, _, _>(|request, ctx| async move {
        let phone = format!("+86138{:08}", rand::random::<u32>() % 100_000_000);

        // 1. 发送短信验证码
        let send_res = request
            .post("/v1/phone-verification-codes")
            .json(&serde_json::json!({
                "phoneNumber": phone
            }))
            .await;
        assert_eq!(send_res.status_code(), StatusCode::OK);

        // 2. 从数据库获取 SMS OTP
        let identifier = format!("phone-otp:{phone}");
        let record = verification::Entity::find()
            .filter(verification::Column::Identifier.eq(&identifier))
            .one(&ctx.db)
            .await
            .unwrap()
            .expect("SMS OTP must exist in DB");
        let otp = record.value;

        // 3. 校验短信验证码登入 (自动创建手机号用户)
        let verify_res = request
            .post("/v1/phone-verifications")
            .json(&serde_json::json!({
                "phoneNumber": phone,
                "code": otp
            }))
            .await;
        assert_eq!(verify_res.status_code(), StatusCode::OK);
        let sign_in: SignInResponse = serde_json::from_str(&verify_res.text()).unwrap();
        let user = sign_in.user.expect("Expected SignInResponse to carry user");
        let auth_token = sign_in
            .token
            .expect("Expected SignInResponse to carry token");
        assert_eq!(user.phone_number.as_deref(), Some(phone.as_str()));
        assert!(user.phone_number_verified);

        // 4. 已登录用户修改绑定新手机号
        let new_phone = format!("+86139{:08}", rand::random::<u32>() % 100_000_000);
        let send_new = request
            .post("/v1/phone-verification-codes")
            .json(&serde_json::json!({
                "phoneNumber": new_phone
            }))
            .await;
        assert_eq!(send_new.status_code(), StatusCode::OK);

        let new_id = format!("phone-otp:{new_phone}");
        let new_rec = verification::Entity::find()
            .filter(verification::Column::Identifier.eq(&new_id))
            .one(&ctx.db)
            .await
            .unwrap()
            .unwrap();

        let update_res = request
            .post("/v1/phone-verifications")
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {auth_token}"),
            )
            .json(&serde_json::json!({
                "phoneNumber": new_phone,
                "code": new_rec.value,
                "updatePhoneNumber": true
            }))
            .await;
        assert_eq!(update_res.status_code(), StatusCode::OK);
    })
    .await;
}
