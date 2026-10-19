use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

/// 发起 OAuth2 授权登录请求参数 [Request DTO]
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SignInOAuth2Request {
    #[validate(length(min = 1, message = "Provider ID 不能为空"))]
    pub provider_id: String,
    pub callback_url: Option<String>,
    pub error_callback_url: Option<String>,
    pub redirect: Option<bool>,
}

/// OAuth2 回调请求参数 [Request DTO]
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OAuth2CallbackRequest {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

/// One Tap 回调登录请求参数 [Request DTO]
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OneTapCallbackRequest {
    #[validate(length(min = 1, message = "凭证 ID Token 不能为空"))]
    pub credential: String,
    pub client_id: Option<String>,
}

/// OAuth2 授权重定向响应数据对象 [Response DTO]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OAuth2UrlResource {
    pub url: String,
    pub redirect: bool,
}
