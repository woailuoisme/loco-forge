use axum::http::StatusCode;
use loco_rs::testing::prelude::*;
use locos::{app::App, dtos::auth::*};
use serial_test::serial;
use uuid::Uuid;

#[tokio::test]
#[serial]
async fn test_jwt_and_jwks_flow() {
    request::<App, _, _>(|request, _ctx| async move {
        // 1. 注册用户
        let reg = request
            .post("/v1/users")
            .json(&serde_json::json!({
                "email": format!("jwt_user_{}@example.com", Uuid::new_v4().simple()),
                "password": "strong_password_123",
                "name": "JWT User"
            }))
            .await;
        assert_eq!(reg.status_code(), StatusCode::CREATED);
        let auth: AuthResource = serde_json::from_str(&reg.text()).unwrap();

        // 2. 签发 JWT 令牌
        let token_resp = request
            .post("/v1/access-tokens")
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", auth.token),
            )
            .await;
        assert_eq!(token_resp.status_code(), StatusCode::OK);
        let jwt_data: JwtTokenResource = serde_json::from_str(&token_resp.text()).unwrap();
        assert!(!jwt_data.token.is_empty());

        // 3. 获取公开 JWKS
        let jwks_resp = request.get("/v1/jwks").await;
        assert_eq!(jwks_resp.status_code(), StatusCode::OK);
        let jwks_data: JwksResource = serde_json::from_str(&jwks_resp.text()).unwrap();
        assert!(!jwks_data.keys.is_empty());
        assert_eq!(jwks_data.keys[0]["kty"], "oct");
    })
    .await;
}
