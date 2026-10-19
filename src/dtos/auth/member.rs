use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::models::{invitation, member};

use super::*;

/// 邀请成员加入组织请求参数 [Request DTO]
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct InviteMemberRequest {
    #[serde(alias = "organization_id")]
    #[validate(length(min = 1, message = "organizationId 不能为空"))]
    pub organization_id: String,
    #[validate(email(message = "邮箱格式不正确"))]
    pub email: String,
    pub role: String,
    pub resend: Option<bool>,
}

/// 接受组织邀请请求参数 [Request DTO]
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AcceptInvitationRequest {
    #[serde(alias = "invitation_id")]
    #[validate(length(min = 1, message = "invitationId 不能为空"))]
    pub invitation_id: String,
}

/// 拒绝组织邀请请求参数 [Request DTO]
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RejectInvitationRequest {
    #[serde(alias = "invitation_id")]
    #[validate(length(min = 1, message = "invitationId 不能为空"))]
    pub invitation_id: String,
}

/// 撤回组织邀请请求参数 [Request DTO]
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CancelInvitationRequest {
    #[serde(alias = "invitation_id")]
    #[validate(length(min = 1, message = "invitationId 不能为空"))]
    pub invitation_id: String,
}

/// 移除组织成员请求参数 [Request DTO]
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RemoveMemberRequest {
    #[serde(alias = "organization_id")]
    #[validate(length(min = 1, message = "organizationId 不能为空"))]
    pub organization_id: String,
    #[serde(alias = "member_id_or_email", alias = "memberId", alias = "userId")]
    #[validate(length(min = 1, message = "memberIdOrEmail 不能为空"))]
    pub member_id_or_email: String,
}

/// 更新成员角色请求参数 [Request DTO]
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMemberRoleRequest {
    #[serde(alias = "organization_id")]
    #[validate(length(min = 1, message = "organizationId 不能为空"))]
    pub organization_id: String,
    #[serde(alias = "member_id", alias = "userId")]
    #[validate(length(min = 1, message = "memberId 不能为空"))]
    pub member_id: String,
    #[validate(length(min = 1, message = "角色不能为空"))]
    pub role: String,
}

/// 退出组织请求参数 [Request DTO]
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LeaveOrganizationRequest {
    #[serde(alias = "organization_id")]
    #[validate(length(min = 1, message = "organizationId 不能为空"))]
    pub organization_id: String,
}

/// 组织成员响应数据对象 [Response DTO]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MemberResource {
    pub id: String,
    pub organization_id: String,
    pub user_id: String,
    pub role: String,
    pub created_at: DateTime<FixedOffset>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<UserResource>,
}

impl From<member::Model> for MemberResource {
    fn from(m: member::Model) -> Self {
        Self {
            id: m.id,
            organization_id: m.organization_id,
            user_id: m.user_id,
            role: m.role,
            created_at: m.created_at,
            user: None,
        }
    }
}

/// 当前活跃成员响应数据对象 [Response DTO]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ActiveMemberResource {
    pub id: String,
    pub organization_id: String,
    pub user_id: String,
    pub role: String,
    pub created_at: DateTime<FixedOffset>,
}

impl From<member::Model> for ActiveMemberResource {
    fn from(m: member::Model) -> Self {
        Self {
            id: m.id,
            organization_id: m.organization_id,
            user_id: m.user_id,
            role: m.role,
            created_at: m.created_at,
        }
    }
}

/// 组织邀请响应数据对象 [Response DTO]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct InvitationResource {
    pub id: String,
    pub organization_id: String,
    pub email: String,
    pub role: String,
    pub status: String,
    pub expires_at: DateTime<FixedOffset>,
    pub inviter_id: String,
}

impl From<invitation::Model> for InvitationResource {
    fn from(m: invitation::Model) -> Self {
        Self {
            id: m.id,
            organization_id: m.organization_id,
            email: m.email,
            role: m.role,
            status: m.status,
            expires_at: m.expires_at,
            inviter_id: m.inviter_id,
        }
    }
}
