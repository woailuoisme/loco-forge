//! 后台管理员认证服务 (Admin Auth Service)
//!
//! 在认证域 [`crate::services::SessionService`] 之上增加一道**管理员闸门**：
//! 只有「存在 + 活跃 + 具备 admin 角色」的账号才能进入后台登录流程。
//!
//! **闸门位置的意义**：`/v1/sessions/email` 允许任何普通用户成功登录，
//! 拿到 token 后调用 `/v1/admin/*` 才被 `AdminUser` 提取器以 403 拒绝 ——
//! 攻击者仍可无限次用合法凭据试探后台接口。本服务把拒绝提前到登录第一步。

use loco_rs::prelude::*;

use crate::{
    dtos::{
        admin::{AdminLoginRequest, AdminMeResource},
        auth::{SessionResource, SignInResponse},
    },
    errors::{AppError, AppResult},
    models::{session, user},
    services::SessionService,
};

/// 统一的登录失败文案
///
/// 账号不存在、非管理员、账号非活跃、密码错误四种情形**必须返回同一文案**，
/// 否则会构成账号枚举通道 —— 攻击者可用任意密码试探，
/// 凡返回差异响应者即确认是有效的管理员邮箱。
const INVALID_CREDENTIALS: &str = "Invalid email or password";

/// 后台管理员认证服务
pub struct AdminAuthService;

impl AdminAuthService {
    /// 管理员登录。
    ///
    /// 流程：管理员闸门校验 → 复用认证域登录实现 (密码校验 / 2FA 分支 /
    /// 会话创建 / Cookie 签发)。
    ///
    /// 返回 `(登录响应, 可选的 2FA Challenge Token)`，由 handler 决定 Cookie。
    pub async fn login(
        db: &DatabaseConnection,
        params: &AdminLoginRequest,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> AppResult<(SignInResponse, Option<String>)> {
        let email = params.email.trim().to_lowercase();

        // ── 管理员闸门 ────────────────────────────────────────────────────────
        let Some(user) = user::Entity::find_by_email(db, &email).await? else {
            tracing::warn!(email = %email, ip = ?ip, "后台登录被拒：账号不存在");
            return Err(AppError::unauthorized(INVALID_CREDENTIALS));
        };

        if !user.is_admin() {
            tracing::warn!(
                user_id = %user.id,
                email = %email,
                role = %user.role,
                ip = ?ip,
                "后台登录被拒：账号不具备管理员角色"
            );
            return Err(AppError::unauthorized(INVALID_CREDENTIALS));
        }

        // is_active() 内部含 is_banned() (识别临时封禁是否已到期) 与 is_deleted()
        if !user.is_active() {
            tracing::warn!(
                user_id = %user.id,
                email = %email,
                status = %user.status,
                banned = user.banned,
                ip = ?ip,
                "后台登录被拒：账号状态非活跃"
            );
            return Err(AppError::unauthorized(INVALID_CREDENTIALS));
        }

        // ── 复用认证域实现 ────────────────────────────────────────────────────
        let result = SessionService::sign_in_email(db, params, ip.clone(), user_agent).await?;

        tracing::info!(
            admin_id = %user.id,
            email = %email,
            ip = ?ip,
            "后台管理员登录成功"
        );
        Ok(result)
    }

    /// 管理员登出 (吊销当前会话)。
    pub async fn logout(db: &DatabaseConnection, admin_id: &str, token: &str) -> AppResult<()> {
        SessionService::sign_out(db, token).await?;
        tracing::info!(admin_id = %admin_id, "后台管理员登出成功");
        Ok(())
    }

    /// 组装当前管理员信息 (含完整运维投影与会话详情)。
    #[must_use]
    pub fn me(admin: &user::Model, session: &session::Model) -> AdminMeResource {
        AdminMeResource {
            user: admin.clone().into(),
            session: SessionResource::from(session.clone()),
        }
    }
}
