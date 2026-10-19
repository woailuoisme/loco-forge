use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use super::*;

/// 发起设备授权码请求参数 [Request DTO]
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeviceCodeRequest {
    pub client_id: Option<String>,
    pub scope: Option<String>,
}

/// 查询设备授权码状态请求参数 [Request DTO]
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeviceCheckRequest {
    #[serde(alias = "user_code")]
    #[validate(length(min = 1, message = "用户码不能为空"))]
    pub user_code: String,
}

/// 批准设备授权请求参数 [Request DTO]
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeviceApproveRequest {
    #[serde(alias = "user_code")]
    #[validate(length(min = 1, message = "用户码不能为空"))]
    pub user_code: String,
}

/// 拒绝设备授权请求参数 [Request DTO]
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeviceDenyRequest {
    #[serde(alias = "user_code")]
    #[validate(length(min = 1, message = "用户码不能为空"))]
    pub user_code: String,
}

/// 轮询获取设备访问令牌请求参数 [Request DTO]
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeviceTokenRequest {
    #[serde(alias = "grant_type")]
    pub grant_type: String,
    #[serde(alias = "device_code")]
    pub device_code: String,
    #[serde(alias = "client_id")]
    pub client_id: Option<String>,
}

/// 设备授权码响应数据对象 [Response DTO]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeviceCodeResource {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub verification_uri_complete: Option<String>,
    pub expires_in: i64,
    pub interval: i64,
}

/// 设备码检查响应数据对象 [Response DTO]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeviceCheckResource {
    pub user_code: String,
    pub status: String,
    pub client_id: Option<String>,
    pub scope: Option<String>,
}

/// 设备授权令牌响应数据对象 [Response DTO]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeviceTokenResource {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: UserResource,
    pub session: SessionResource,
}
