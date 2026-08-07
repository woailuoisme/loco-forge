use loco_rs::testing::prelude::*;
use locos::app::App;

#[tokio::test]
#[serial_test::serial]
async fn auth_endpoint_rate_limiting_triggers_429() {
    request::<App, _, _>(|request, _ctx| async move {
        // 快速连续对 /api/v1/auth/login 发送超额请求 (超过 5 次突发阈值)
        let login_payload = serde_json::json!({
            "email": "rate_limit_test@example.com",
            "password": "wrong_password"
        });

        let mut hit_429 = false;

        for _ in 0..12 {
            let res = request.post("/api/v1/auth/login").json(&login_payload).await;
            if res.status_code() == axum::http::StatusCode::TOO_MANY_REQUESTS {
                hit_429 = true;
                // 验证返回包含 Retry-After 头与 429 状态码
                assert!(res.headers().contains_key("retry-after"));
                break;
            }
        }

        assert!(
            hit_429,
            "Rate limiter should trigger 429 Too Many Requests when burst exceeded"
        );
    })
    .await;
}
