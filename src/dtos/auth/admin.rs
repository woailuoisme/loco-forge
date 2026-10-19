use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

/// 管理员设置用户角色请求参数 [Request DTO]
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminSetRoleRequest {
    #[serde(alias = "user_id")]
    #[validate(length(min = 1, message = "userId 不能为空"))]
    pub user_id: String,
    #[validate(length(min = 1, message = "角色不能为空"))]
    pub role: String,
}

/// 管理员封禁用户请求参数 [Request DTO]
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminBanUserRequest {
    #[serde(alias = "user_id")]
    #[validate(length(min = 1, message = "userId 不能为空"))]
    pub user_id: String,
    #[serde(alias = "ban_reason")]
    pub ban_reason: Option<String>,
    #[serde(alias = "ban_expires_in")]
    pub ban_expires_in: Option<i64>,
}

/// 管理员解封用户请求参数 [Request DTO]
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminUnbanUserRequest {
    #[serde(alias = "user_id")]
    #[validate(length(min = 1, message = "userId 不能为空"))]
    pub user_id: String,
}

/// 管理员模拟用户请求参数 [Request DTO]
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminImpersonateUserRequest {
    #[serde(alias = "user_id")]
    #[validate(length(min = 1, message = "userId 不能为空"))]
    pub user_id: String,
}

/// 管理员用户列表查询参数 [Request DTO]
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminListUsersQuery {
    #[serde(alias = "search_field")]
    pub search_field: Option<String>,
    #[serde(alias = "search_value")]
    pub search_value: Option<String>,
    #[serde(alias = "search_operator")]
    pub search_operator: Option<String>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
    #[serde(alias = "sort_by")]
    pub sort_by: Option<String>,
    #[serde(alias = "sort_direction")]
    pub sort_direction: Option<String>,
}

/// 管理员移除用户请求参数 [Request DTO]
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminRemoveUserRequest {
    #[serde(alias = "user_id")]
    #[validate(length(min = 1, message = "userId 不能为空"))]
    pub user_id: String,
}
