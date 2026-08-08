use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, utoipa::ToSchema)]
pub struct ForgotPasswordRequest {
    /// 注册邮箱
    pub email: String,
}

#[derive(Debug, Deserialize, Serialize, utoipa::ToSchema)]
pub struct ResetPasswordRequest {
    /// 重置密码 Token
    pub token: String,
    /// 新密码
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize, utoipa::ToSchema)]
pub struct MagicLinkRequest {
    /// 注册邮箱
    pub email: String,
}

#[derive(Debug, Deserialize, Serialize, utoipa::ToSchema)]
pub struct ResendVerificationRequest {
    /// 注册邮箱
    pub email: String,
}
