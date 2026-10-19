use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::models::passkey;

/// 生成 Passkey 认证选项请求参数 [Request DTO]
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PasskeyGenerateAuthOptionsRequest {
    #[validate(length(min = 1, message = "邮箱不能为空"))]
    pub email: String,
}

/// 验证 Passkey 认证并登录请求参数 [Request DTO]
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PasskeyVerifyAuthRequest {
    pub response: serde_json::Value,
    pub credential_id: Option<String>,
}

/// 生成 Passkey 注册选项请求参数 [Request DTO]
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PasskeyGenerateRegOptionsRequest {
    pub name: Option<String>,
}

/// 验证 Passkey 注册请求参数 [Request DTO]
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PasskeyVerifyRegRequest {
    pub response: serde_json::Value,
    pub name: Option<String>,
}

/// 删除 Passkey 请求参数 [Request DTO]
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeletePasskeyRequest {
    #[validate(length(min = 1, message = "Passkey ID 不能为空"))]
    pub id: String,
}

/// 更新 Passkey 名称请求参数 [Request DTO]
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePasskeyRequest {
    #[validate(length(min = 1, message = "Passkey ID 不能为空"))]
    pub id: String,
    #[validate(length(min = 1, message = "Passkey 名称不能为空"))]
    pub name: String,
}

/// Passkey 详情响应数据对象 [Response DTO]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PasskeyResource {
    pub id: String,
    pub name: Option<String>,
    pub public_key: String,
    pub credential_id: String,
    pub counter: i64,
    pub device_type: String,
    pub backed_up: bool,
    pub transports: Option<String>,
    pub created_at: DateTime<FixedOffset>,
}

impl From<passkey::Model> for PasskeyResource {
    fn from(m: passkey::Model) -> Self {
        Self {
            id: m.id,
            name: m.name,
            public_key: m.public_key,
            credential_id: m.credential_id,
            counter: m.counter,
            device_type: m.device_type,
            backed_up: m.backed_up,
            transports: m.transports,
            created_at: m.created_at,
        }
    }
}
