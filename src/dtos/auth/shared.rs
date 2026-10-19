use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

use crate::models::{session, user};

use super::*;

// ─────────────────────────────────────────────────────────────────────────────
// 响应数据对象 (Response Resource DTOs)
// ─────────────────────────────────────────────────────────────────────────────

/// 用户信息响应数据对象 [Response DTO]
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserResource {
    pub id: String,
    pub name: String,
    pub email: String,
    pub email_verified: bool,
    pub image: Option<String>,
    pub role: String,
    pub banned: bool,
    pub username: Option<String>,
    pub display_username: Option<String>,
    pub phone_number: Option<String>,
    pub phone_number_verified: bool,
    pub is_anonymous: bool,
    pub last_login_method: Option<String>,
    pub two_factor_enabled: bool,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

impl From<user::Model> for UserResource {
    fn from(m: user::Model) -> Self {
        Self {
            id: m.id,
            name: m.name,
            email: m.email,
            email_verified: m.email_verified,
            image: m.image,
            role: m.role,
            banned: m.banned,
            username: m.username,
            display_username: m.display_username,
            phone_number: m.phone_number,
            phone_number_verified: m.phone_number_verified,
            is_anonymous: m.is_anonymous,
            last_login_method: m.last_login_method,
            two_factor_enabled: m.two_factor_enabled,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

/// 持久化会话响应数据对象 [Response DTO]
///
/// 安全约束：不回传会话 token（仅登录响应顶层回传一次当前 token），
/// 防止 list-sessions 泄漏用户全部会话凭据。
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SessionResource {
    pub id: String,
    pub user_id: String,
    pub expires_at: DateTime<FixedOffset>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub active_organization_id: Option<String>,
    pub impersonated_by: Option<String>,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

impl From<session::Model> for SessionResource {
    fn from(s: session::Model) -> Self {
        Self {
            id: s.id,
            user_id: s.user_id,
            expires_at: s.expires_at,
            ip_address: s.ip_address,
            user_agent: s.user_agent,
            active_organization_id: s.active_organization_id,
            impersonated_by: s.impersonated_by,
            created_at: s.created_at,
            updated_at: s.updated_at,
        }
    }
}

/// 登录 / 注册成功响应数据对象 (Better-Auth Client 顶级标准结构) [Response DTO]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthResource {
    /// 鉴权会话 Token
    pub token: String,
    /// 认证用户信息
    pub user: UserResource,
    /// 会话详情
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session: Option<SessionResource>,
}

/// 兼容别名（测试与客户端沿用）
pub type AuthSessionResource = AuthResource;

/// 获取当前会话响应数据对象 (Better-Auth get-session 标准结构) [Response DTO]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SessionContextResource {
    /// 当前活跃会话详情
    pub session: SessionResource,
    /// 当前认证用户信息
    pub user: UserResource,
}

/// 通用操作成功状态响应数据对象 [Response DTO]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StatusResource {
    pub status: bool,
}

impl StatusResource {
    #[must_use]
    pub fn ok() -> Self {
        Self { status: true }
    }
}

/// 登录统一响应对象 [Response DTO]
///
/// **根结构唯一**：普通登录返回 `{token, user, session?}`；
/// 需二次验证时仅返回 `{twoFactorRedirect: true}`，调用方以该字段判别分支。
/// 字段按需省略 (`skip_serializing_if`)，因此两种情况共享同一 JSON 根结构，
/// 不再使用 `#[serde(untagged)]` 枚举 (会因条件不同产生多变根结构)。
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SignInResponse {
    /// 鉴权会话 Token (2FA 挑战场景下不出现)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    /// 认证用户信息 (2FA 挑战场景下不出现)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<UserResource>,
    /// 会话详情 (2FA 挑战场景下不出现)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session: Option<SessionResource>,
    /// 是否需要重定向至 2FA 验证 (仅 2FA 场景出现且为 `true`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub two_factor_redirect: Option<bool>,
}

impl From<AuthResource> for SignInResponse {
    fn from(auth: AuthResource) -> Self {
        Self {
            token: Some(auth.token),
            user: Some(auth.user),
            session: auth.session,
            two_factor_redirect: None,
        }
    }
}

impl From<TwoFactorRedirectResource> for SignInResponse {
    fn from(redirect: TwoFactorRedirectResource) -> Self {
        Self {
            token: None,
            user: None,
            session: None,
            two_factor_redirect: Some(redirect.two_factor_redirect),
        }
    }
}
