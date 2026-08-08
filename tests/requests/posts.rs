use loco_rs::testing::prelude::*;
use locos::{app::App, dtos::common::ApiResponse};
use serde_json::Value;

use super::prepare_data::{auth_header, init_admin_login, init_user_login};

#[tokio::test]
#[serial_test::serial]
async fn can_list_posts_with_pagination() {
    request::<App, _, _>(|request, ctx| async move {
        let user = init_user_login(&request, &ctx).await;
        let (header_name, header_value) = auth_header(&user.token);

        let res = request
            .get("/api/v1/admin/posts?page=1&per_page=5")
            .add_header(header_name, header_value)
            .await;

        res.assert_status_ok();

        let api_res: ApiResponse<Value> = serde_json::from_str(&res.text()).unwrap();
        assert!(api_res.success);
        let data = api_res.data.unwrap();
        assert_eq!(data["meta"]["current_page"], 1);
        assert_eq!(data["meta"]["per_page"], 5);
    })
    .await;
}

#[tokio::test]
#[serial_test::serial]
async fn non_admin_cannot_delete_post() {
    request::<App, _, _>(|request, ctx| async move {
        let admin = init_admin_login(&request, &ctx).await;
        let (admin_name, admin_value) = auth_header(&admin.token);

        // 管理员先创建一篇文章，保证后续删除目标一定存在。
        let create_res = request
            .post("/api/v1/admin/posts")
            .add_header(admin_name.clone(), admin_value.clone())
            .json(&serde_json::json!({
                "title": "NonAdmin Delete Target",
                "content": "Content"
            }))
            .await;
        create_res.assert_status(axum::http::StatusCode::CREATED);
        let create_api_res: ApiResponse<Value> =
            serde_json::from_str(&create_res.text()).unwrap();
        let post_id = create_api_res.data.unwrap()["id"].as_i64().unwrap();

        // 普通用户尝试删除，被 RBAC AdminUser 提取器拦截，返回 403 Forbidden。
        let user = init_user_login(&request, &ctx).await;
        let (header_name, header_value) = auth_header(&user.token);

        let res = request
            .delete(&format!("/api/v1/admin/posts/{post_id}"))
            .add_header(header_name, header_value)
            .await;

        res.assert_status(axum::http::StatusCode::FORBIDDEN);
    })
    .await;
}

#[tokio::test]
#[serial_test::serial]
async fn admin_can_delete_post() {
    request::<App, _, _>(|request, ctx| async move {
        let admin = init_admin_login(&request, &ctx).await;
        let (header_name, header_value) = auth_header(&admin.token);

        // 先创建一个文章
        let create_res = request
            .post("/api/v1/admin/posts")
            .add_header(header_name.clone(), header_value.clone())
            .json(&serde_json::json!({
                "title": "Test Post Title",
                "content": "Test Post Content"
            }))
            .await;
        create_res.assert_status(axum::http::StatusCode::CREATED);
        let create_api_res: ApiResponse<Value> = serde_json::from_str(&create_res.text()).unwrap();
        let post_id = create_api_res.data.unwrap()["id"].as_i64().unwrap();

        // 管理员成功执行删除
        let delete_res = request
            .delete(&format!("/api/v1/admin/posts/{post_id}"))
            .add_header(header_name, header_value)
            .await;

        delete_res.assert_status_ok();
    })
    .await;
}
