use axum::http::StatusCode;
use loco_rs::testing::prelude::*;
use locos::{app::App, dtos::auth::*};
use serial_test::serial;
use uuid::Uuid;

#[tokio::test]
#[serial]
async fn test_device_authorization_flow() {
    request::<App, _, _>(|request, _ctx| async move {
        // 1. 无头设备申请设备授权码
        let code_resp = request
            .post("/v1/device-authorizations")
            .json(&serde_json::json!({
                "clientId": "cli-terminal",
                "scope": "read write"
            }))
            .await;
        assert_eq!(code_resp.status_code(), StatusCode::OK);
        let dev_code: DeviceCodeResource = serde_json::from_str(&code_resp.text()).unwrap();
        assert!(!dev_code.device_code.is_empty());
        assert!(!dev_code.user_code.is_empty());

        // 2. 前端验证界面查询 user_code 状态
        let check_resp = request
            .get(&format!("/v1/device-authorizations/{}", dev_code.user_code))
            .await;
        assert_eq!(check_resp.status_code(), StatusCode::OK);
        let check_data: DeviceCheckResource = serde_json::from_str(&check_resp.text()).unwrap();
        assert_eq!(check_data.status, "pending");
        assert_eq!(check_data.client_id.as_deref(), Some("cli-terminal"));

        // 3. 设备端轮询令牌 -> 尚未批准应返回 400 authorization_pending
        let poll_pending = request
            .post("/v1/device-authorization-tokens")
            .json(&serde_json::json!({
                "grantType": "urn:ietf:params:oauth:grant-type:device_code",
                "deviceCode": dev_code.device_code
            }))
            .await;
        assert_eq!(poll_pending.status_code(), StatusCode::BAD_REQUEST);
        let pending_json: serde_json::Value = serde_json::from_str(&poll_pending.text()).unwrap();
        assert_eq!(pending_json["message"], "authorization_pending");

        // 4. 用户登录并批准设备
        let reg_resp = request
            .post("/v1/users")
            .json(&serde_json::json!({
                "email": format!("device_user_{}@example.com", Uuid::new_v4().simple()),
                "password": "strong_password_123",
                "name": "Device Approver"
            }))
            .await;
        let auth_res: AuthResource = serde_json::from_str(&reg_resp.text()).unwrap();

        let approve_resp = request
            .post("/v1/device-authorization-approvals")
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", auth_res.token),
            )
            .json(&serde_json::json!({
                "userCode": dev_code.user_code
            }))
            .await;
        assert_eq!(approve_resp.status_code(), StatusCode::OK);
        let approve_stat: StatusResource = serde_json::from_str(&approve_resp.text()).unwrap();
        assert!(approve_stat.status);

        // 5. 设备端再次轮询令牌 -> 授权成功签发 Access Token
        let poll_success = request
            .post("/v1/device-authorization-tokens")
            .json(&serde_json::json!({
                "grantType": "urn:ietf:params:oauth:grant-type:device_code",
                "deviceCode": dev_code.device_code
            }))
            .await;
        assert_eq!(poll_success.status_code(), StatusCode::OK);
        let token_data: DeviceTokenResource = serde_json::from_str(&poll_success.text()).unwrap();
        assert!(!token_data.access_token.is_empty());
        assert_eq!(token_data.user.id, auth_res.user.id);
    })
    .await;
}
