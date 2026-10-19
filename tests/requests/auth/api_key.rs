use axum::http::StatusCode;
use loco_rs::testing::prelude::*;
use locos::{app::App, dtos::auth::*};
use serial_test::serial;
use uuid::Uuid;

#[tokio::test]
#[serial]
async fn test_api_key_lifecycle_and_verification() {
    request::<App, _, _>(|request, _ctx| async move {
        // 1. 用户注册
        let user_email = format!("apikey_{}@example.com", Uuid::new_v4().simple());
        let reg = request
            .post("/v1/users")
            .add_header("x-forwarded-for", "10.30.3.1")
            .json(&serde_json::json!({
                "email": user_email,
                "password": "strong_password_123",
                "name": "API Key User"
            }))
            .await;
        assert_eq!(reg.status_code(), StatusCode::CREATED);
        let auth: AuthSessionResource = serde_json::from_str(&reg.text()).unwrap();

        // 2. 创建新 API Key (设置配额 5 次)
        let create_resp = request
            .post("/v1/api-keys")
            .add_header("x-forwarded-for", "10.30.3.2")
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", auth.token),
            )
            .json(&serde_json::json!({
                "name": "Production Key",
                "prefix": "sk_",
                "remaining": 5,
                "expiresIn": 86400,
                "permissions": { "read": true, "write": false }
            }))
            .await;
        assert_eq!(create_resp.status_code(), StatusCode::OK);
        let key_created: ApiKeyCreatedResource = serde_json::from_str(&create_resp.text()).unwrap();
        assert!(key_created.key.starts_with("sk_"));
        assert_eq!(key_created.api_key.name.as_deref(), Some("Production Key"));
        assert_eq!(key_created.api_key.remaining, Some(5));
        let raw_key = key_created.key;
        let key_id = key_created.api_key.id;

        // 3. 列出 API Key (安全设计：返回元数据，不暴露明文 Key)
        let list_resp = request
            .get("/v1/api-keys")
            .add_header("x-forwarded-for", "10.30.3.3")
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", auth.token),
            )
            .await;
        assert_eq!(list_resp.status_code(), StatusCode::OK);
        let keys: Vec<ApiKeyResource> = serde_json::from_str(&list_resp.text()).unwrap();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].id, key_id);

        // 4. 外部客户端免登校验 API Key (消费 1 次配额)
        let verify_resp = request
            .post("/v1/api-key-checks")
            .add_header("x-forwarded-for", "10.30.3.4")
            .json(&serde_json::json!({
                "key": raw_key
            }))
            .await;
        assert_eq!(verify_resp.status_code(), StatusCode::OK);
        let verify_data: VerifyApiKeyResource = serde_json::from_str(&verify_resp.text()).unwrap();
        assert!(verify_data.valid);
        let consumed_key = verify_data.key.unwrap();
        assert_eq!(consumed_key.remaining, Some(4));
        assert_eq!(consumed_key.request_count, 1);

        // 5. 更新 API Key (禁用)
        let update_resp = request
            .patch("/v1/api-keys")
            .add_header("x-forwarded-for", "10.30.3.5")
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", auth.token),
            )
            .json(&serde_json::json!({
                "keyId": key_id,
                "enabled": false
            }))
            .await;
        assert_eq!(update_resp.status_code(), StatusCode::OK);
        let updated_key: ApiKeyResource = serde_json::from_str(&update_resp.text()).unwrap();
        assert!(!updated_key.enabled);

        // 6. 校验被禁用的 API Key 应返回 valid: false
        let verify_disabled = request
            .post("/v1/api-key-checks")
            .add_header("x-forwarded-for", "10.30.3.6")
            .json(&serde_json::json!({
                "key": raw_key
            }))
            .await;
        assert_eq!(verify_disabled.status_code(), StatusCode::OK);
        let verify_disabled_data: VerifyApiKeyResource =
            serde_json::from_str(&verify_disabled.text()).unwrap();
        assert!(!verify_disabled_data.valid);

        // 7. 删除 API Key
        let del_resp = request
            .delete(&format!("/v1/api-keys?keyId={key_id}"))
            .add_header("x-forwarded-for", "10.30.3.7")
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", auth.token),
            )
            .await;
        assert_eq!(del_resp.status_code(), StatusCode::OK);
        let del_status: StatusResource = serde_json::from_str(&del_resp.text()).unwrap();
        assert!(del_status.status);

        // 8. 校验已被删除的 Key 返回 valid: false
        let verify_deleted = request
            .post("/v1/api-key-checks")
            .add_header("x-forwarded-for", "10.30.3.8")
            .json(&serde_json::json!({
                "key": raw_key
            }))
            .await;
        assert_eq!(verify_deleted.status_code(), StatusCode::OK);
        let verify_deleted_data: VerifyApiKeyResource =
            serde_json::from_str(&verify_deleted.text()).unwrap();
        assert!(!verify_deleted_data.valid);
    })
    .await;
}
