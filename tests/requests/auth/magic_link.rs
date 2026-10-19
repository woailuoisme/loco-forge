use axum::http::StatusCode;
use loco_rs::testing::prelude::*;
use locos::{app::App, dtos::auth::*, models::verification};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serial_test::serial;
use uuid::Uuid;

#[tokio::test]
#[serial]
async fn test_magic_link_flow() {
    request::<App, _, _>(|request, ctx| async move {
        let email = format!("magic_{}@example.com", Uuid::new_v4().simple());

        // 1. 发送 Magic Link
        let send_res = request
            .post("/v1/sessions/magic-link")
            .json(&serde_json::json!({
                "email": email
            }))
            .await;
        assert_eq!(send_res.status_code(), StatusCode::OK);

        // 2. 从数据库获取 Magic Link 凭证
        let identifier = format!("magic-link:{email}");
        let record = verification::Entity::find()
            .filter(verification::Column::Identifier.eq(&identifier))
            .one(&ctx.db)
            .await
            .unwrap()
            .expect("Magic link record must exist in DB");
        let token = record.value;

        // 3. GET /api/auth/magic-link/verify 验证并登入
        let verify_res = request.get(&format!("/v1/magic-link-tokens/{token}")).await;
        assert_eq!(verify_res.status_code(), StatusCode::OK);
        let auth: AuthResource = serde_json::from_str(&verify_res.text()).unwrap();
        assert_eq!(auth.user.email, email);
        assert!(!auth.token.is_empty());
        assert!(auth.user.email_verified);

        // 4. 重复使用已被消费的 Token 应报错 400
        let replay_res = request.get(&format!("/v1/magic-link-tokens/{token}")).await;
        assert_eq!(replay_res.status_code(), StatusCode::BAD_REQUEST);

        // 5. 测试带 callbackURL 的 302 重定向
        let send_res2 = request
            .post("/v1/sessions/magic-link")
            .json(&serde_json::json!({
                "email": email
            }))
            .await;
        assert_eq!(send_res2.status_code(), StatusCode::OK);

        let rec2 = verification::Entity::find()
            .filter(verification::Column::Identifier.eq(&identifier))
            .one(&ctx.db)
            .await
            .unwrap()
            .unwrap();

        // 重定向仅允许站内相对路径（开放重定向防护），绝对 URL 回退 200 JSON 响应
        let redirect_res = request
            .get(&format!(
                "/v1/magic-link-tokens/{}?callbackURL=%2Fdashboard",
                rec2.value
            ))
            .await;
        assert_eq!(redirect_res.status_code(), StatusCode::FOUND);
        let loc = redirect_res
            .headers()
            .get(axum::http::header::LOCATION)
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(loc, "/dashboard");

        // 绝对 URL（站外）必须被拒绝重定向，回退为 200 登录响应
        let send_res3 = request
            .post("/v1/sessions/magic-link")
            .json(&serde_json::json!({
                "email": email
            }))
            .await;
        assert_eq!(send_res3.status_code(), StatusCode::OK);
        let rec3 = verification::Entity::find()
            .filter(verification::Column::Identifier.eq(&identifier))
            .one(&ctx.db)
            .await
            .unwrap()
            .unwrap();
        let evil_res = request
            .get(&format!(
                "/v1/magic-link-tokens/{}?callbackURL=https%3A%2F%2Fevil.example.com%2Fphish",
                rec3.value
            ))
            .await;
        assert_eq!(evil_res.status_code(), StatusCode::OK);
    })
    .await;
}
