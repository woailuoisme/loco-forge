use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

// ─────────────────────────────────────────────────────────────────────────────
// 请求参数 (Request DTOs)
// ─────────────────────────────────────────────────────────────────────────────

/// 邮箱注册请求参数 [Request DTO]
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
#[validate(schema(function = "validate_sign_up_req"))]
pub struct SignUpEmailRequest {
    /// 注册邮箱
    #[validate(email(message = "邮箱格式不正确"))]
    pub email: String,

    /// 登录密码
    #[schema(value_type = String, format = Password)]
    pub password: SecretString,

    /// 用户姓名
    #[validate(length(min = 1, message = "用户姓名不能为空"))]
    pub name: String,

    /// 头像图片 URL
    pub image: Option<String>,

    /// 可选用户名
    pub username: Option<String>,
}

/// 邮箱密码登录请求参数 [Request DTO]
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SignInEmailRequest {
    /// 登录邮箱
    #[validate(email(message = "邮箱格式不正确"))]
    pub email: String,

    /// 登录密码
    #[schema(value_type = String, format = Password)]
    pub password: SecretString,
}

/// 用户名密码登录请求参数 [Request DTO]
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SignInUsernameRequest {
    /// 用户名
    #[validate(length(min = 1, message = "用户名不能为空"))]
    pub username: String,

    /// 登录密码
    #[schema(value_type = String, format = Password)]
    pub password: SecretString,
}

/// 匿名会话登录请求参数 [Request DTO]
#[derive(Debug, Clone, Default, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SignInAnonymousRequest {
    /// 可选自定义显示昵称
    pub name: Option<String>,
}

/// 吊销特定会话请求参数 [Request DTO]
#[derive(Debug, Clone, Default, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RevokeSessionRequest {
    /// 目标 Session Token
    pub token: Option<String>,
    /// 目标 Session ID
    #[serde(alias = "sessionId")]
    pub id: Option<String>,
}

/// 多会话激活切换请求参数 [Request DTO]
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SetActiveSessionRequest {
    /// 目标 Session Token
    #[validate(length(min = 1, message = "sessionToken 不能为空"))]
    pub session_token: String,
}

fn validate_sign_up_req(req: &SignUpEmailRequest) -> Result<(), validator::ValidationError> {
    if req.password.expose_secret().chars().count() < 6 {
        let mut err = validator::ValidationError::new("length");
        err.message = Some("密码长度不能少于 6 个字符".into());
        return Err(err);
    }
    Ok(())
}
