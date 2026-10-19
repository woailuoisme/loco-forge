use axum::http::StatusCode;
use loco_rs::testing::prelude::*;
use locos::{app::App, dtos::auth::*};
use serial_test::serial;
use uuid::Uuid;

// 说明：真实 OAuth2 回调需向 provider 完成 code→token 交换，One Tap 需 Google JWKS 验签，
// 集成测试无法离线伪造。此处覆盖：授权 URL 按配置生成、伪造凭据/无效 code 必须被拒绝。
#[tokio::test]
#[serial]
async fn test_oauth2_and_onetap_flow() {
    request::<App, _, _>(|request, _ctx| async move {
        // 1. 发起 OAuth2 授权地址获取（provider 凭据来自 settings.oauth2.google）
        let oauth_init = request
            .post("/v1/sessions/oauth2")
            .json(&serde_json::json!({
                "providerId": "google"
            }))
            .await;
        assert_eq!(oauth_init.status_code(), StatusCode::OK);
        let oauth_url: OAuth2UrlResource = serde_json::from_str(&oauth_init.text()).unwrap();
        assert!(oauth_url.url.contains("state="));
        assert!(oauth_url.url.contains("client_id=test_client_id"));
        assert!(
            oauth_url
                .url
                .starts_with("https://accounts.google.com/o/oauth2/v2/auth")
        );

        // 提取 state
        let state = oauth_url
            .url
            .split("state=")
            .nth(1)
            .and_then(|s| s.split('&').next())
            .unwrap();

        // 2. 无效授权码必须被拒绝（token 交换失败）
        let cb_resp = request
            .get(&format!(
                "/v1/oauth2-callbacks?code=invalid_{}&state={state}",
                Uuid::new_v4().simple()
            ))
            .await;
        assert!(cb_resp.status_code().is_client_error() || cb_resp.status_code().is_server_error());

        // 3. 伪造 state 必须被拒绝
        let fake_state_resp = request
            .get("/v1/oauth2-callbacks?code=x&state=forged_state_value")
            .await;
        assert_eq!(fake_state_resp.status_code(), StatusCode::BAD_REQUEST);

        // 4. 伪造 One Tap 凭据必须被拒绝（JWT 验签失败）
        let onetap_resp = request
            .post("/v1/one-tap-credentials")
            .json(&serde_json::json!({
                "credential": "forged.token.value"
            }))
            .await;
        assert!(
            onetap_resp.status_code().is_client_error()
                || onetap_resp.status_code().is_server_error()
        );
    })
    .await;
}
