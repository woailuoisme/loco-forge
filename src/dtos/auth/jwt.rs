use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// JWT 令牌响应数据对象 [Response DTO]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct JwtTokenResource {
    pub token: String,
}

/// JWKS 响应数据对象 [Response DTO]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct JwksResource {
    pub keys: Vec<serde_json::Value>,
}
