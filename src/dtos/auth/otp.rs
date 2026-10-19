use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use super::*;

// ─────────────────────────────────────────────────────────────────────────────
// Email OTP 请求参数
// ─────────────────────────────────────────────────────────────────────────────

/// 发送邮箱 OTP 请求参数 [Request DTO]
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SendEmailOtpRequest {
    /// 目标邮箱
    #[validate(email(message = "邮箱格式不正确"))]
    pub email: String,

    /// OTP 业务类型: "sign-in" | "email-verification" | "forget-password"
    #[serde(rename = "type")]
    #[validate(length(min = 1, message = "type 不能为空"))]
    pub otp_type: String,
}

/// 检查邮箱 OTP 验证码有效性请求参数 [Request DTO]
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CheckEmailOtpRequest {
    /// 目标邮箱
    #[validate(email(message = "邮箱格式不正确"))]
    pub email: String,

    /// OTP 业务类型
    #[serde(rename = "type")]
    #[validate(length(min = 1, message = "type 不能为空"))]
    pub otp_type: String,

    /// OTP 验证码
    #[validate(length(min = 1, message = "otp 不能为空"))]
    pub otp: String,
}

/// 邮箱 OTP 登录请求参数 [Request DTO]
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SignInEmailOtpRequest {
    /// 目标邮箱
    #[validate(email(message = "邮箱格式不正确"))]
    pub email: String,

    /// OTP 验证码
    #[validate(length(min = 1, message = "otp 不能为空"))]
    pub otp: String,

    /// 用户姓名 (新用户自动注册时使用)
    pub name: Option<String>,

    /// 头像图片 URL
    pub image: Option<String>,
}

/// 邮箱验证 OTP 请求参数 [Request DTO]
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct VerifyEmailOtpRequest {
    /// 目标邮箱
    #[validate(email(message = "邮箱格式不正确"))]
    pub email: String,

    /// OTP 验证码
    #[validate(length(min = 1, message = "otp 不能为空"))]
    pub otp: String,
}

/// 邮箱 OTP 重置密码请求参数 [Request DTO]
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ResetPasswordEmailOtpRequest {
    /// 目标邮箱
    #[validate(email(message = "邮箱格式不正确"))]
    pub email: String,

    /// OTP 验证码
    #[validate(length(min = 1, message = "otp 不能为空"))]
    pub otp: String,

    /// 新密码
    #[schema(value_type = String, format = Password)]
    pub password: SecretString,
}

// ─────────────────────────────────────────────────────────────────────────────
// Phone Number 请求参数
// ─────────────────────────────────────────────────────────────────────────────

/// 发送手机号短信验证码请求参数 [Request DTO]
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SendPhoneOtpRequest {
    /// 手机号码
    #[validate(length(min = 3, message = "手机号码不能为空"))]
    pub phone_number: String,
}

/// 校验手机验证码请求参数 [Request DTO]
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct VerifyPhoneNumberRequest {
    /// 手机号码
    #[validate(length(min = 3, message = "手机号码不能为空"))]
    pub phone_number: String,

    /// 短信验证码
    #[validate(length(min = 1, message = "验证码不能为空"))]
    pub code: String,

    /// 是否禁用自动创建会话 (默认 false，即验证通过自动登录)
    pub disable_session: Option<bool>,

    /// 是否作为修改手机号使用 (默认 false)
    pub update_phone_number: Option<bool>,
}

/// 手机号密码登录请求参数 [Request DTO]
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SignInPhoneNumberRequest {
    /// 手机号码
    #[validate(length(min = 3, message = "手机号码不能为空"))]
    pub phone_number: String,

    /// 登录密码
    #[schema(value_type = String, format = Password)]
    pub password: SecretString,

    /// 记住我
    pub remember_me: Option<bool>,
}

/// 邮箱 OTP 验证成功响应数据对象 [Response DTO]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct VerifyEmailOtpResource {
    pub status: bool,
    pub user: UserResource,
}
