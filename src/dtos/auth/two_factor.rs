use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

// ─────────────────────────────────────────────────────────────────────────────
// Two-Factor 请求参数
// ─────────────────────────────────────────────────────────────────────────────

/// 两步验证操作需要密码确认请求参数 [Request DTO]
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TwoFactorPasswordRequest {
    /// 当前用户密码
    #[schema(value_type = String, format = Password)]
    pub password: SecretString,
}

/// 验证 TOTP 动态码请求参数 [Request DTO]
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct VerifyTotpRequest {
    /// 6 位数字 TOTP 验证码
    #[validate(length(min = 6, max = 8, message = "TOTP 验证码必须为 6-8 位"))]
    pub code: String,

    /// 信任此设备
    pub trust_device: Option<bool>,
}

/// 验证单次备用恢复码请求参数 [Request DTO]
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct VerifyBackupCodeRequest {
    /// 备用恢复码
    #[validate(length(min = 1, message = "备用恢复码不能为空"))]
    pub code: String,

    /// 信任此设备
    pub trust_device: Option<bool>,
}

/// 启用 2FA 成功返回的 TOTP URI 与初始备用码列表 [Response DTO]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TwoFactorEnableResource {
    #[serde(rename = "totpURI")]
    pub totp_uri: String,
    #[serde(rename = "backupCodes")]
    pub backup_codes: Vec<String>,
}

/// 重新生成备用码响应数据对象 [Response DTO]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TwoFactorBackupCodesResource {
    #[serde(rename = "backupCodes")]
    pub backup_codes: Vec<String>,
}

/// 登录重定向至 2FA 验证挑战响应 [Response DTO]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TwoFactorRedirectResource {
    #[serde(rename = "twoFactorRedirect")]
    pub two_factor_redirect: bool,
}

impl Default for TwoFactorRedirectResource {
    fn default() -> Self {
        Self {
            two_factor_redirect: true,
        }
    }
}
