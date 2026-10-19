use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::models::organization;

use super::*;

/// 创建组织请求参数 [Request DTO]
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrganizationRequest {
    #[validate(length(min = 1, message = "组织名称不能为空"))]
    pub name: String,
    #[validate(length(min = 1, message = "组织标识 slug 不能为空"))]
    pub slug: String,
    pub logo: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// 更新组织数据包
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateOrganizationDataRequest {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub logo: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// 更新组织请求参数 [Request DTO]
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateOrganizationRequest {
    #[serde(alias = "organization_id")]
    #[validate(length(min = 1, message = "organizationId 不能为空"))]
    pub organization_id: String,
    pub name: Option<String>,
    pub slug: Option<String>,
    pub logo: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub data: Option<UpdateOrganizationDataRequest>,
}

/// 获取组织详情查询参数 [Request DTO]
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetFullOrganizationQuery {
    #[serde(alias = "organization_id")]
    pub organization_id: Option<String>,
    #[serde(alias = "organization_slug")]
    pub organization_slug: Option<String>,
}

/// 设置当前激活组织请求参数 [Request DTO]
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SetActiveOrganizationRequest {
    #[serde(alias = "organization_id")]
    pub organization_id: Option<String>,
}

/// 组织详情响应数据对象 [Response DTO]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationResource {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub logo: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<FixedOffset>,
}

impl From<organization::Model> for OrganizationResource {
    fn from(m: organization::Model) -> Self {
        let metadata = m.metadata.and_then(|s| serde_json::from_str(&s).ok());
        Self {
            id: m.id,
            name: m.name,
            slug: m.slug,
            logo: m.logo,
            metadata,
            created_at: m.created_at,
        }
    }
}

/// 完整组织架构详情响应数据对象 [Response DTO]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FullOrganizationResource {
    #[serde(flatten)]
    pub organization: OrganizationResource,
    pub members: Vec<MemberResource>,
    pub invitations: Vec<InvitationResource>,
}
