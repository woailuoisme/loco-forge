use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

// ─────────────────────────────────────────────────────────────────────────────
// Magic Link 请求参数
// ─────────────────────────────────────────────────────────────────────────────

/// 发送 Magic Link 请求参数 [Request DTO]
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SignInMagicLinkRequest {
    /// 目标邮箱
    #[validate(email(message = "邮箱格式不正确"))]
    pub email: String,

    /// 成功登录后回调跳转地址
    #[serde(rename = "callbackURL")]
    pub callback_url: Option<String>,

    /// 新用户注册成功后回调跳转地址
    #[serde(rename = "newUserCallbackURL")]
    pub new_user_callback_url: Option<String>,

    /// 错误时回调跳转地址
    #[serde(rename = "errorCallbackURL")]
    pub error_callback_url: Option<String>,

    /// 新用户姓名
    pub name: Option<String>,
}

/// 验证 Magic Link 查询参数 [Request Query]
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct VerifyMagicLinkQuery {
    /// 重定向回调 URL
    #[serde(rename = "callbackURL")]
    pub callback_url: Option<String>,
}
