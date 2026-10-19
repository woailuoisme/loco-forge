use axum::http::StatusCode;
use loco_rs::testing::prelude::*;
use locos::{app::App, dtos::auth::*, models::passkey};
use serial_test::serial;
use uuid::Uuid;

// 说明：真实 WebAuthn 断言需由浏览器/认证器产生签名，集成测试无法离线伪造。
// 此处覆盖：挑战选项生成、管理 CRUD（直接落库凭据）与伪造凭据必须被拒绝的安全路径。
#[tokio::test]
#[serial]
async fn test_passkey_options_registration_authentication_and_management() {
    request::<App, _, _>(|request, ctx| async move {
        // 1. 注册并登录用户
        let email = format!("passkey_user_{}@example.com", Uuid::new_v4().simple());
        let reg = request
            .post("/v1/users")
            .json(&serde_json::json!({
                "email": email,
                "password": "strong_password_123",
                "name": "Passkey User"
            }))
            .await;
        assert_eq!(reg.status_code(), StatusCode::CREATED);
        let user_auth: AuthResource = serde_json::from_str(&reg.text()).unwrap();

        // 2. 生成注册选项 (WebAuthn creation challenge)
        let reg_options_resp = request
            .post("/v1/passkey-registration-challenges")
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", user_auth.token),
            )
            .json(&serde_json::json!({}))
            .await;
        assert_eq!(reg_options_resp.status_code(), StatusCode::OK);
        let reg_options_val: serde_json::Value =
            serde_json::from_str(&reg_options_resp.text()).unwrap();
        assert!(reg_options_val["publicKey"]["challenge"].is_string());

        // 3. 伪造凭据注册必须被拒绝（attestation 校验失败）
        let fake_cred_id = format!("cred_{}", Uuid::new_v4().simple());
        let verify_reg_resp = request
            .post("/v1/passkeys")
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", user_auth.token),
            )
            .json(&serde_json::json!({
                "response": {
                    "id": fake_cred_id,
                    "rawId": fake_cred_id,
                    "response": {
                        "clientDataJSON": "eyJjaGFsbGVuZ2UiOiJ0ZXN0In0",
                        "attestationObject": "o2NmbXRkbm9uZWdhdHRTdG10oGhhdXRoRGF0YVik",
                        "publicKey": "MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAE"
                    },
                    "type": "public-key"
                },
                "name": "My MacBook TouchID"
            }))
            .await;
        assert_eq!(verify_reg_resp.status_code(), StatusCode::BAD_REQUEST);

        // 4. 直接落库凭据以覆盖管理接口（绕过需真实认证器的注册流程）
        let pk = passkey::Entity::create_passkey(
            &ctx.db,
            &user_auth.user.id,
            &fake_cred_id,
            "{}",
            0,
            Some("My MacBook TouchID".into()),
            Some("platform".into()),
            false,
            Some("internal".into()),
            None,
        )
        .await
        .unwrap();

        // 5. 列出当前用户的 Passkeys
        let list_resp = request
            .get("/v1/passkeys")
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", user_auth.token),
            )
            .await;
        assert_eq!(list_resp.status_code(), StatusCode::OK);
        let passkeys: Vec<PasskeyResource> = serde_json::from_str(&list_resp.text()).unwrap();
        assert_eq!(passkeys.len(), 1);
        assert_eq!(passkeys[0].id, pk.id);

        // 6. 更新 Passkey 备注名
        let update_resp = request
            .patch("/v1/passkeys")
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", user_auth.token),
            )
            .json(&serde_json::json!({
                "id": pk.id,
                "name": "Work Laptop"
            }))
            .await;
        assert_eq!(update_resp.status_code(), StatusCode::OK);
        let updated_pk: PasskeyResource = serde_json::from_str(&update_resp.text()).unwrap();
        assert_eq!(updated_pk.name.as_deref(), Some("Work Laptop"));

        // 7. 认证挑战：未知邮箱统一 404（防枚举）；直接落库的占位凭据无法发起
        //    WebAuthn 断言（需真实认证器），完整登录链路由浏览器端 E2E 覆盖
        let unknown_email_resp = request
            .post("/v1/passkey-authentication-challenges")
            .json(&serde_json::json!({
                "email": format!("ghost_{}@example.com", Uuid::new_v4().simple())
            }))
            .await;
        assert_eq!(unknown_email_resp.status_code(), StatusCode::NOT_FOUND);

        // 8. 伪造断言必须被拒绝（无有效挑战状态 → 400；伪造签名 → 401）
        let verify_auth_resp = request
            .post("/v1/passkey-authentications")
            .json(&serde_json::json!({
                "response": {
                    "id": fake_cred_id,
                    "rawId": fake_cred_id,
                    "response": {
                        "authenticatorData": "SZYN5YgOjGh0NBcPZHZgW4_krrmihjLHmVzzuoMdl2MBAAAA",
                        "clientDataJSON": "eyJjaGFsbGVuZ2UiOiJ0ZXN0In0",
                        "signature": "MEYCIQDa...",
                        "userHandle": user_auth.user.id
                    },
                    "type": "public-key"
                }
            }))
            .await;
        assert!(
            verify_auth_resp.status_code() == StatusCode::BAD_REQUEST
                || verify_auth_resp.status_code() == StatusCode::UNAUTHORIZED
        );

        // 9. 删除 Passkey
        let del_resp = request
            .delete(&format!("/v1/passkeys?id={}", pk.id))
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", user_auth.token),
            )
            .await;
        assert_eq!(del_resp.status_code(), StatusCode::OK);
        let del_stat: StatusResource = serde_json::from_str(&del_resp.text()).unwrap();
        assert!(del_stat.status);
    })
    .await;
}
