use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

/// 登录请求参数 [Request DTO]
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
    /// 注册邮箱
    #[validate(email(message = "邮箱格式不正确"))]
    pub email: String,
    /// 密码
    #[schema(value_type = String, format = Password)]
    pub password: SecretString,
}

/// 注册请求参数 [Request DTO]
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RegisterRequest {
    /// 注册邮箱
    #[validate(email(message = "邮箱格式不正确"))]
    pub email: String,
    /// 密码
    #[schema(value_type = String, format = Password)]
    pub password: SecretString,
    /// 用户姓名
    #[validate(length(min = 2, message = "用户姓名至少包含 2 个字符"))]
    pub name: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ForgotPasswordRequest {
    /// 注册邮箱
    #[validate(email(message = "邮箱格式不正确"))]
    pub email: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ResetPasswordRequest {
    /// 重置密码 Token
    #[validate(length(min = 1, message = "重置 Token 不能为空"))]
    pub token: String,
    /// 新密码
    #[schema(value_type = String, format = Password)]
    pub password: SecretString,
}

/// 发送 6 位数字 OTP 验证码请求参数 [Request DTO]
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct SendOtpRequest {
    /// 注册邮箱
    #[validate(email(message = "邮箱格式不正确"))]
    pub email: String,
}

/// 校验 6 位数字 OTP 验证码请求参数 [Request DTO]
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct VerifyOtpRequest {
    /// 注册邮箱
    #[validate(email(message = "邮箱格式不正确"))]
    pub email: String,
    /// 6 位纯数字验证码
    #[validate(length(equal = 6, message = "验证码必须为 6 位数字"))]
    pub code: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ResendVerificationRequest {
    /// 注册邮箱
    #[validate(email(message = "邮箱格式不正确"))]
    pub email: String,
}

/// 登录成功响应数据对象 [Response DTO]
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LoginResource {
    /// JWT 认证 Token
    pub token: String,
    /// 用户公共 PID (UUID)
    pub pid: String,
    /// 用户姓名
    pub name: String,
    /// 邮箱是否已验证
    pub is_verified: bool,
}

impl LoginResource {
    #[must_use]
    pub fn new(user: &crate::models::_entities::users::Model, token: &str) -> Self {
        Self {
            token: token.to_owned(),
            pid: user.pid.to_string(),
            name: user.name.clone(),
            is_verified: user.email_verified_at.is_some(),
        }
    }
}

/// 当前登录用户信息响应数据对象 [Response DTO]
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CurrentUserResource {
    /// 用户公共 PID (UUID)
    pub pid: String,
    /// 用户姓名
    pub name: String,
    /// 用户邮箱
    pub email: String,
}

impl CurrentUserResource {
    #[must_use]
    pub fn new(user: &crate::models::_entities::users::Model) -> Self {
        Self {
            pid: user.pid.to_string(),
            name: user.name.clone(),
            email: user.email.clone(),
        }
    }
}
