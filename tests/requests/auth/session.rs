use axum::http::StatusCode;
use loco_rs::testing::prelude::*;
use locos::{app::App, dtos::auth::*};
use serial_test::serial;
use uuid::Uuid;

#[tokio::test]
#[serial]
async fn test_sign_up_email_success_and_cookie() {
    request::<App, _, _>(|request, _ctx| async move {
        let email = format!("signup_{}@example.com", Uuid::new_v4().simple());
        let payload = serde_json::json!({
            "email": email,
            "password": "strong_password_123",
            "name": "Better Auth User",
            "username": format!("u_{}", &Uuid::new_v4().simple().to_string()[..8])
        });

        let response = request.post("/v1/users").json(&payload).await;
        assert_eq!(response.status_code(), StatusCode::CREATED);

        // 验证 Set-Cookie 包含 better-auth.session_token
        let set_cookie = response
            .headers()
            .get("set-cookie")
            .and_then(|v| v.to_str().ok());
        assert!(set_cookie.is_some(), "Set-Cookie header should be present");
        assert!(set_cookie.unwrap().contains("better-auth.session_token="));

        // 验证 Naked JSON 结构 (不嵌套在 data 字段)
        let res: AuthSessionResource = serde_json::from_str(&response.text()).unwrap();
        assert_eq!(res.user.email, email);
        assert_eq!(res.user.name, "Better Auth User");
        assert!(!res.token.is_empty());
        assert_eq!(res.session.as_ref().unwrap().user_id, res.user.id);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn test_sign_up_email_duplicate_returns_409() {
    request::<App, _, _>(|request, _ctx| async move {
        let email = format!("duplicate_{}@example.com", Uuid::new_v4().simple());
        let payload = serde_json::json!({
            "email": email,
            "password": "strong_password_123",
            "name": "First User"
        });

        let res1 = request.post("/v1/users").json(&payload).await;
        assert_eq!(res1.status_code(), StatusCode::CREATED);

        // 再次以相同邮箱注册
        let res2 = request.post("/v1/users").json(&payload).await;
        assert_eq!(res2.status_code(), StatusCode::CONFLICT);

        // 验证错误格式符合 api.md: { "code": "...", "message": "..." }
        let err_json: serde_json::Value = serde_json::from_str(&res2.text()).unwrap();
        assert!(err_json.get("code").is_some());
        assert_eq!(err_json["code"], "CONFLICT");
    })
    .await;
}

#[tokio::test]
#[serial]
async fn test_sign_up_email_short_password_returns_422() {
    request::<App, _, _>(|request, _ctx| async move {
        let email = format!("short_pw_{}@example.com", Uuid::new_v4().simple());
        let payload = serde_json::json!({
            "email": email,
            "password": "123", // 少于 6 个字符
            "name": "Short PW User"
        });

        let response = request.post("/v1/users").json(&payload).await;
        assert_eq!(response.status_code(), StatusCode::UNPROCESSABLE_ENTITY);

        let err_json: serde_json::Value = serde_json::from_str(&response.text()).unwrap();
        assert!(err_json.get("code").is_some());
    })
    .await;
}

#[tokio::test]
#[serial]
async fn test_sign_in_email_flow() {
    request::<App, _, _>(|request, _ctx| async move {
        let email = format!("signin_{}@example.com", Uuid::new_v4().simple());
        let password = "my_secret_password";

        // 先注册
        let signup_payload = serde_json::json!({
            "email": email,
            "password": password,
            "name": "Sign In Test User"
        });
        let signup_res = request.post("/v1/users").json(&signup_payload).await;
        assert_eq!(signup_res.status_code(), StatusCode::CREATED);

        // 正确密码登录
        let login_payload = serde_json::json!({
            "email": email,
            "password": password
        });
        let login_res = request
            .post("/v1/sessions/email")
            .json(&login_payload)
            .await;
        assert_eq!(login_res.status_code(), StatusCode::OK);
        let res: AuthSessionResource = serde_json::from_str(&login_res.text()).unwrap();
        assert_eq!(res.user.email, email);

        // 错误密码登录 -> 401
        let wrong_pw_payload = serde_json::json!({
            "email": email,
            "password": "wrong_password"
        });
        let wrong_res = request
            .post("/v1/sessions/email")
            .json(&wrong_pw_payload)
            .await;
        assert_eq!(wrong_res.status_code(), StatusCode::UNAUTHORIZED);

        // 不存在的邮箱 -> 401
        let no_user_payload = serde_json::json!({
            "email": "nonexistent_email_12345@example.com",
            "password": password
        });
        let no_user_res = request
            .post("/v1/sessions/email")
            .json(&no_user_payload)
            .await;
        assert_eq!(no_user_res.status_code(), StatusCode::UNAUTHORIZED);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn test_sign_in_username_flow() {
    request::<App, _, _>(|request, _ctx| async move {
        let email = format!("username_test_{}@example.com", Uuid::new_v4().simple());
        let username = format!("user_{}", &Uuid::new_v4().simple().to_string()[..8]);
        let password = "password_for_username";

        // 注册带用户名
        let signup_payload = serde_json::json!({
            "email": email,
            "password": password,
            "name": "Username Test",
            "username": username
        });
        let signup_res = request.post("/v1/users").json(&signup_payload).await;
        assert_eq!(signup_res.status_code(), StatusCode::CREATED);

        // 用户名密码登录
        let login_payload = serde_json::json!({
            "username": username,
            "password": password
        });
        let login_res = request
            .post("/v1/sessions/username")
            .json(&login_payload)
            .await;
        assert_eq!(login_res.status_code(), StatusCode::OK);
        let res: AuthSessionResource = serde_json::from_str(&login_res.text()).unwrap();
        assert_eq!(res.user.email, email);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn test_sign_in_anonymous_flow() {
    request::<App, _, _>(|request, _ctx| async move {
        let payload = serde_json::json!({});
        let response = request.post("/v1/sessions/anonymous").json(&payload).await;
        assert_eq!(response.status_code(), StatusCode::OK);

        let res: AuthSessionResource = serde_json::from_str(&response.text()).unwrap();
        assert!(res.user.email.starts_with("anon_"));
        assert!(res.user.email.ends_with("@anonymous.local"));
        assert!(res.user.name.starts_with("Anonymous"));
        assert!(!res.token.is_empty());
    })
    .await;
}

#[tokio::test]
#[serial]
async fn test_get_session_authenticated_and_unauthenticated() {
    request::<App, _, _>(|request, _ctx| async move {
        // 未认证访问当前会话：401 AUTH_TOKEN_INVALID
        let unauth_res = request.get("/v1/sessions/current").await;
        assert_eq!(unauth_res.status_code(), StatusCode::UNAUTHORIZED);
        let unauth_json: serde_json::Value = serde_json::from_str(&unauth_res.text()).unwrap();
        assert_eq!(unauth_json["code"], "AUTH_TOKEN_INVALID");

        // 注册新用户
        let email = format!("session_check_{}@example.com", Uuid::new_v4().simple());
        let signup_payload = serde_json::json!({
            "email": email,
            "password": "password_123456",
            "name": "Session Check User"
        });
        let signup_res = request.post("/v1/users").json(&signup_payload).await;
        let auth_res: AuthSessionResource = serde_json::from_str(&signup_res.text()).unwrap();

        // 1. 使用 Bearer Token 访问 get-session
        let bearer_res = request
            .get("/v1/sessions/current")
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", auth_res.token),
            )
            .await;
        assert_eq!(bearer_res.status_code(), StatusCode::OK);
        let session_data: SessionContextResource =
            serde_json::from_str(&bearer_res.text()).unwrap();
        assert_eq!(session_data.user.email, email);

        // 2. 使用 Cookie 访问 get-session
        let cookie_res = request
            .get("/v1/sessions/current")
            .add_header(
                axum::http::header::COOKIE,
                format!("better-auth.session_token={}", auth_res.token),
            )
            .await;
        assert_eq!(cookie_res.status_code(), StatusCode::OK);
        let session_data_cookie: SessionContextResource =
            serde_json::from_str(&cookie_res.text()).unwrap();
        assert_eq!(session_data_cookie.user.email, email);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn test_sign_out_flow() {
    request::<App, _, _>(|request, _ctx| async move {
        let email = format!("signout_{}@example.com", Uuid::new_v4().simple());
        let signup_payload = serde_json::json!({
            "email": email,
            "password": "password_123456",
            "name": "Sign Out User"
        });
        let signup_res = request.post("/v1/users").json(&signup_payload).await;
        let auth_res: AuthSessionResource = serde_json::from_str(&signup_res.text()).unwrap();

        // 执行 sign-out
        let signout_res = request
            .delete("/v1/sessions/current")
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", auth_res.token),
            )
            .await;
        assert_eq!(signout_res.status_code(), StatusCode::NO_CONTENT);

        // 验证 Set-Cookie 包含 Max-Age=0 (清空 Cookie)
        let set_cookie = signout_res
            .headers()
            .get("set-cookie")
            .and_then(|v| v.to_str().ok());
        assert!(set_cookie.is_some());
        assert!(set_cookie.unwrap().contains("Max-Age=0"));

        // 登出后再调用当前会话接口应 401
        let check_res = request
            .get("/v1/sessions/current")
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", auth_res.token),
            )
            .await;
        assert_eq!(check_res.status_code(), StatusCode::UNAUTHORIZED);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn test_multi_session_management() {
    request::<App, _, _>(|request, _ctx| async move {
        let email = format!("multisess_{}@example.com", Uuid::new_v4().simple());
        let password = "password_123456";

        // 注册获取会话 1
        let signup_payload = serde_json::json!({
            "email": email,
            "password": password,
            "name": "Multi Session User"
        });
        let signup_res = request.post("/v1/users").json(&signup_payload).await;
        let auth1: AuthSessionResource = serde_json::from_str(&signup_res.text()).unwrap();

        // 再次登录获取会话 2
        let login_payload = serde_json::json!({
            "email": email,
            "password": password
        });
        let login_res = request
            .post("/v1/sessions/email")
            .json(&login_payload)
            .await;
        let auth2: AuthSessionResource = serde_json::from_str(&login_res.text()).unwrap();

        // 列表查看会话 (应至少有 2 个会话)
        let list_res = request
            .get("/v1/sessions")
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", auth1.token),
            )
            .await;
        assert_eq!(list_res.status_code(), StatusCode::OK);
        let sessions: Vec<SessionResource> = serde_json::from_str(&list_res.text()).unwrap();
        assert!(sessions.len() >= 2);

        // 吊销指定会话 (吊销会话 2)
        let revoke_res = request
            .delete(&format!(
                "/v1/sessions/{}",
                auth2.session.as_ref().unwrap().id
            ))
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", auth1.token),
            )
            .await;
        assert_eq!(revoke_res.status_code(), StatusCode::NO_CONTENT);

        // 会话 2 调用 get-session 应返回 null
        let check2 = request
            .get("/v1/sessions/current")
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", auth2.token),
            )
            .await;
        assert_eq!(check2.status_code(), StatusCode::UNAUTHORIZED);

        // 会话 1 依然有效
        let check1 = request
            .get("/v1/sessions/current")
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", auth1.token),
            )
            .await;
        assert_ne!(check1.text().trim(), "null");
    })
    .await;
}
