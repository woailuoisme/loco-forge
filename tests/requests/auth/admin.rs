use axum::http::StatusCode;
use loco_rs::testing::prelude::*;
use locos::{app::App, dtos::auth::*, dtos::common::PaginatedResource, models::user};
use serial_test::serial;
use uuid::Uuid;

#[tokio::test]
#[serial]
async fn test_admin_management_and_impersonation() {
    request::<App, _, _>(|request, ctx| async move {
        // 1. 注册超级管理员与普通用户
        let admin_email = format!("admin_{}@example.com", Uuid::new_v4().simple());
        let admin_reg = request
            .post("/v1/users")
            .add_header("x-forwarded-for", "10.30.1.1")
            .json(&serde_json::json!({
                "email": admin_email,
                "password": "strong_password_123",
                "name": "Platform Admin"
            }))
            .await;
        assert_eq!(admin_reg.status_code(), StatusCode::CREATED);
        let admin_auth: AuthSessionResource = serde_json::from_str(&admin_reg.text()).unwrap();

        // 提升为平台 admin 角色
        let admin_model = user::Entity::find_by_id(&ctx.db, &admin_auth.user.id)
            .await
            .unwrap()
            .unwrap();
        admin_model.set_role(&ctx.db, "admin").await.unwrap();

        let user_email = format!("user_{}@example.com", Uuid::new_v4().simple());
        let user_reg = request
            .post("/v1/users")
            .add_header("x-forwarded-for", "10.30.1.2")
            .json(&serde_json::json!({
                "email": user_email,
                "password": "strong_password_123",
                "name": "Normal User"
            }))
            .await;
        assert_eq!(user_reg.status_code(), StatusCode::CREATED);
        let user_auth: AuthSessionResource = serde_json::from_str(&user_reg.text()).unwrap();

        // 2. 非管理员调用管理员接口应返回 403 Forbidden
        let forbidden_resp = request
            .get("/v1/admin/accounts")
            .add_header("x-forwarded-for", "10.30.1.3")
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", user_auth.token),
            )
            .await;
        assert_eq!(forbidden_resp.status_code(), StatusCode::FORBIDDEN);

        // 3. 管理员分页获取用户列表
        let list_resp = request
            .get("/v1/admin/accounts?limit=10")
            .add_header("x-forwarded-for", "10.30.1.4")
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", admin_auth.token),
            )
            .await;
        assert_eq!(list_resp.status_code(), StatusCode::OK);
        let list_data: PaginatedResource<UserResource> =
            serde_json::from_str(&list_resp.text()).unwrap();
        assert!(list_data.meta.total >= 2);
        assert!(!list_data.items.is_empty());

        // 4. 管理员设置用户角色
        let role_resp = request
            .patch("/v1/admin/user-roles")
            .add_header("x-forwarded-for", "10.30.1.5")
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", admin_auth.token),
            )
            .json(&serde_json::json!({
                "userId": user_auth.user.id,
                "role": "moderator"
            }))
            .await;
        assert_eq!(role_resp.status_code(), StatusCode::OK);
        let updated_user: UserResource = serde_json::from_str(&role_resp.text()).unwrap();
        assert_eq!(updated_user.role, "moderator");

        // 5. 管理员封禁用户
        let ban_resp = request
            .patch("/v1/admin/user-bans")
            .add_header("x-forwarded-for", "10.30.1.6")
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", admin_auth.token),
            )
            .json(&serde_json::json!({
                "userId": user_auth.user.id,
                "banReason": "Policy Violation",
                "banExpiresIn": 3600
            }))
            .await;
        assert_eq!(ban_resp.status_code(), StatusCode::OK);
        let banned_user: UserResource = serde_json::from_str(&ban_resp.text()).unwrap();
        assert!(banned_user.banned);

        // 被封禁用户的原会话已被撤销，当前会话接口返回 401
        let check_session = request
            .get("/v1/sessions/current")
            .add_header("x-forwarded-for", "10.30.1.7")
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", user_auth.token),
            )
            .await;
        assert_eq!(check_session.status_code(), StatusCode::UNAUTHORIZED);

        let list_auth_session = request
            .get("/v1/sessions")
            .add_header("x-forwarded-for", "10.30.1.8")
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", user_auth.token),
            )
            .await;
        assert_eq!(list_auth_session.status_code(), StatusCode::UNAUTHORIZED);

        // 6. 管理员解封用户
        let unban_resp = request
            .delete("/v1/admin/user-bans")
            .add_header("x-forwarded-for", "10.30.1.9")
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", admin_auth.token),
            )
            .json(&serde_json::json!({
                "userId": user_auth.user.id
            }))
            .await;
        assert_eq!(unban_resp.status_code(), StatusCode::OK);
        let unbanned_user: UserResource = serde_json::from_str(&unban_resp.text()).unwrap();
        assert!(!unbanned_user.banned);

        // 7. 管理员模拟用户登录 (Impersonation)
        let imp_resp = request
            .post("/v1/admin/impersonation-sessions")
            .add_header("x-forwarded-for", "10.30.1.10")
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", admin_auth.token),
            )
            .json(&serde_json::json!({
                "userId": user_auth.user.id
            }))
            .await;
        assert_eq!(imp_resp.status_code(), StatusCode::OK);
        let imp_auth: AuthResource = serde_json::from_str(&imp_resp.text()).unwrap();
        assert_eq!(imp_auth.user.id, user_auth.user.id);

        // 模拟会话验证
        let imp_session_resp = request
            .get("/v1/sessions/current")
            .add_header("x-forwarded-for", "10.30.1.11")
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", imp_auth.token),
            )
            .await;
        assert_eq!(imp_session_resp.status_code(), StatusCode::OK);
        let imp_ctx: SessionContextResource =
            serde_json::from_str(&imp_session_resp.text()).unwrap();
        assert_eq!(imp_ctx.user.id, user_auth.user.id);
        assert_eq!(
            imp_ctx.session.impersonated_by.as_deref(),
            Some(admin_auth.user.id.as_str())
        );

        // 8. 停止模拟并恢复管理员原身份
        let stop_resp = request
            .delete("/v1/admin/impersonation-sessions")
            .add_header("x-forwarded-for", "10.30.1.12")
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", imp_auth.token),
            )
            .await;
        assert_eq!(stop_resp.status_code(), StatusCode::OK);
        let restored_auth: AuthResource = serde_json::from_str(&stop_resp.text()).unwrap();
        assert_eq!(restored_auth.user.id, admin_auth.user.id);

        // 9. 管理员物理删除用户
        let rm_resp = request
            .delete("/v1/admin/accounts")
            .add_header("x-forwarded-for", "10.30.1.13")
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", restored_auth.token),
            )
            .json(&serde_json::json!({
                "userId": user_auth.user.id
            }))
            .await;
        assert_eq!(rm_resp.status_code(), StatusCode::OK);
        let rm_status: StatusResource = serde_json::from_str(&rm_resp.text()).unwrap();
        assert!(rm_status.status);
    })
    .await;
}
