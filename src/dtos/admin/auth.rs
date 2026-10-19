//! 后台管理员 auth DTO (Admin Auth DTOs)
//!
//! 登录响应直接复用认证域的 [`SignInResponse`](crate::dtos::auth::SignInResponse)
//! (含 2FA 分支)，避免重复定义契约。

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::dtos::{
    admin::AdminUserResource,
    auth::{SessionResource, SignInEmailRequest},
};

/// 后台管理员登录请求
///
/// 与 `/v1/sessions/email` 共用字段契约 (邮箱 + 密码)，
/// 差异在于本端点会额外校验管理员身份与账号活跃状态。
pub type AdminLoginRequest = SignInEmailRequest;

/// 后台管理员信息 (`GET /v1/admin/auth/me` 响应)
///
/// 注：未派生 `ts_rs::TS` —— 嵌套的 [`SessionResource`] 未实现该 trait。
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminMeResource {
    /// 管理员的完整用户投影 (含封禁 / 注销等运维字段)
    pub user: AdminUserResource,
    /// 当前活跃会话详情
    pub session: SessionResource,
}
