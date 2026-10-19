use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::models::apikey;

/// 创建 API Key 请求参数 [Request DTO]
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateApiKeyRequest {
    pub name: Option<String>,
    #[serde(alias = "expires_in")]
    pub expires_in: Option<i64>,
    pub prefix: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub permissions: Option<serde_json::Value>,
    #[serde(alias = "rate_limit_enabled")]
    pub rate_limit_enabled: Option<bool>,
    #[serde(alias = "rate_limit_time_window")]
    pub rate_limit_time_window: Option<i32>,
    #[serde(alias = "rate_limit_max")]
    pub rate_limit_max: Option<i32>,
    pub remaining: Option<i32>,
    #[serde(alias = "refill_amount")]
    pub refill_amount: Option<i32>,
    #[serde(alias = "refill_interval")]
    pub refill_interval: Option<i32>,
}

/// 校验 API Key 请求参数 [Request DTO]
///
/// `key` 允许为空：handler 会回退读取 `x-api-key` 请求头，空值统一在服务层判定。
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct VerifyApiKeyRequest {
    pub key: String,
    pub permissions: Option<serde_json::Value>,
}

/// 删除 API Key 请求参数 [Request DTO]
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeleteApiKeyRequest {
    #[serde(alias = "key_id", alias = "id")]
    #[validate(length(min = 1, message = "keyId 不能为空"))]
    pub key_id: String,
}

/// 更新 API Key 请求参数 [Request DTO]
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateApiKeyRequest {
    #[serde(alias = "key_id", alias = "id")]
    #[validate(length(min = 1, message = "keyId 不能为空"))]
    pub key_id: String,
    pub name: Option<String>,
    pub enabled: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

/// API Key 详情响应数据对象 [Response DTO]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeyResource {
    pub id: String,
    pub name: Option<String>,
    pub start: Option<String>,
    pub prefix: Option<String>,
    pub user_id: String,
    pub enabled: bool,
    pub rate_limit_enabled: bool,
    pub rate_limit_time_window: Option<i32>,
    pub rate_limit_max: Option<i32>,
    pub request_count: i32,
    pub remaining: Option<i32>,
    pub last_request_at: Option<DateTime<FixedOffset>>,
    pub expires_at: Option<DateTime<FixedOffset>>,
    pub permissions: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

impl From<apikey::Model> for ApiKeyResource {
    fn from(m: apikey::Model) -> Self {
        let permissions = m.permissions.and_then(|s| serde_json::from_str(&s).ok());
        let metadata = m.metadata.and_then(|s| serde_json::from_str(&s).ok());
        Self {
            id: m.id,
            name: m.name,
            start: m.start,
            prefix: m.prefix,
            user_id: m.user_id,
            enabled: m.enabled,
            rate_limit_enabled: m.rate_limit_enabled,
            rate_limit_time_window: m.rate_limit_time_window,
            rate_limit_max: m.rate_limit_max,
            request_count: m.request_count,
            remaining: m.remaining,
            last_request_at: m.last_request_at,
            expires_at: m.expires_at,
            permissions,
            metadata,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

/// 创建 API Key 成功响应数据对象 (包含明文 Key) [Response DTO]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeyCreatedResource {
    pub key: String,
    #[serde(flatten)]
    pub api_key: ApiKeyResource,
}

/// 校验 API Key 响应数据对象 [Response DTO]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct VerifyApiKeyResource {
    pub valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<ApiKeyResource>,
}
