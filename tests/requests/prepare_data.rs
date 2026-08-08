use axum::http::{HeaderName, HeaderValue};
use loco_rs::{TestServer, app::AppContext};
use locos::{models::users, views::auth::LoginResponse};

const USER_EMAIL: &str = "test@loco.com";
const USER_PASSWORD: &str = "1234";

pub struct LoggedInUser {
    pub user: users::Model,
    pub token: String,
}

pub async fn init_user_login(request: &TestServer, ctx: &AppContext) -> LoggedInUser {
    let register_payload = serde_json::json!({
        "name": "loco",
        "email": USER_EMAIL,
        "password": USER_PASSWORD
    });

    //Creating a new user
    request.post("/api/v1/auth/register").json(&register_payload).await;
    let user = users::Model::find_by_email(&ctx.db, USER_EMAIL).await.unwrap();

    if let Some(token) = &user.email_verification_token {
        request.get(&format!("/api/v1/auth/verify/{token}")).await;
    }

    let response = request
        .post("/api/v1/auth/login")
        .json(&serde_json::json!({
            "email": USER_EMAIL,
            "password": USER_PASSWORD
        }))
        .await;

    let api_res: locos::dtos::common::ApiResponse<LoginResponse> = serde_json::from_str(&response.text()).unwrap();
    let login_response = api_res.data.expect("Login response data expected");

    LoggedInUser {
        user: users::Model::find_by_email(&ctx.db, USER_EMAIL).await.unwrap(),
        token: login_response.token,
    }
}

pub async fn init_admin_login(request: &TestServer, ctx: &AppContext) -> LoggedInUser {
    use sea_orm::{ActiveModelTrait, IntoActiveModel};
    let logged = init_user_login(request, ctx).await;
    let mut active = logged.user.into_active_model();
    active.role = sea_orm::ActiveValue::Set(locos::common::consts::auth::ROLE_ADMIN.to_string());
    let admin_user = active.update(&ctx.db).await.unwrap();
    LoggedInUser {
        user: admin_user,
        token: logged.token,
    }
}

pub fn auth_header(token: &str) -> (HeaderName, HeaderValue) {
    let auth_header_value = HeaderValue::from_str(&format!("Bearer {}", token)).unwrap();

    (HeaderName::from_static("authorization"), auth_header_value)
}
