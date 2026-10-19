use axum::http::{HeaderName, HeaderValue};
use loco_rs::{TestServer, app::AppContext};
use locos::{dtos::auth::AuthSessionResource, models::user};

#[allow(dead_code)]
const USER_EMAIL: &str = "test@loco.com";
const USER_PASSWORD: &str = "123456";

pub(crate) struct LoggedInUser {
    pub user: user::Model,
    pub token: String,
}

#[allow(dead_code)]
pub(crate) async fn init_user_login(request: &TestServer, ctx: &AppContext) -> LoggedInUser {
    init_user_login_with_email(request, ctx, USER_EMAIL).await
}

/// 以唯一随机邮箱注册并登录（持久化 DB 下可重复运行，不依赖 truncate）
#[allow(dead_code)]
pub(crate) async fn init_user_login_unique(request: &TestServer, ctx: &AppContext) -> LoggedInUser {
    let email = format!("test_{}@loco.com", uuid::Uuid::new_v4().simple());
    init_user_login_with_email(request, ctx, &email).await
}

pub(crate) async fn init_user_login_with_email(
    request: &TestServer,
    ctx: &AppContext,
    email: &str,
) -> LoggedInUser {
    let register_payload = serde_json::json!({
        "name": "loco",
        "email": email,
        "password": USER_PASSWORD
    });

    let response = request.post("/v1/users").json(&register_payload).await;

    let res: AuthSessionResource = serde_json::from_str(&response.text()).unwrap();

    LoggedInUser {
        user: user::User::find_by_email(&ctx.db, email)
            .await
            .unwrap()
            .unwrap(),
        token: res.token,
    }
}

#[allow(dead_code)]
pub(crate) async fn init_admin_login(request: &TestServer, ctx: &AppContext) -> LoggedInUser {
    use sea_orm::{ActiveModelTrait, IntoActiveModel};
    let logged = init_user_login_with_email(request, ctx, "admin@loco.com").await;
    let mut active = logged.user.into_active_model();
    active.role = sea_orm::ActiveValue::Set(locos::common::consts::auth::ROLE_ADMIN.to_string());
    let admin_user = active.update(&ctx.db).await.unwrap();
    LoggedInUser {
        user: admin_user,
        token: logged.token,
    }
}

#[allow(dead_code)]
pub(crate) fn auth_header(token: &str) -> (HeaderName, HeaderValue) {
    let auth_header_value = HeaderValue::from_str(&format!("Bearer {token}")).unwrap();

    (HeaderName::from_static("authorization"), auth_header_value)
}
