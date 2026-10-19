//! # 组织成员关系值对象 (Membership)
//!
//! 把「查成员 → 判不存在 → 判角色」三步合成一个深模块：
//! 调用方只需 `Membership::load(..).await?.require_owner_or_admin("..")?`，
//! 权限规则改动只发生在本文件，不再散落在各 service 方法里。

use crate::common::consts;
use crate::errors::{AppError, AppResult};
use crate::models::member;

/// 用户与组织的成员关系
#[derive(Debug, Clone)]
pub struct Membership {
    /// 成员记录（含 role）
    pub member: member::Model,
}

impl Membership {
    /// 加载成员关系；不存在即 403，调用方无需再判 `Option`
    pub async fn load(
        db: &impl sea_orm::ConnectionTrait,
        organization_id: &str,
        user_id: &str,
    ) -> AppResult<Self> {
        let member = member::Entity::find_by_org_and_user(db, organization_id, user_id)
            .await?
            .ok_or_else(|| AppError::forbidden("You are not a member of this organization"))?;

        Ok(Self { member })
    }

    /// 是否为组织所有者
    #[must_use]
    pub fn is_owner(&self) -> bool {
        self.member.role == consts::auth::ROLE_OWNER
    }

    /// 是否具备管理权限（owner 或 admin）
    #[must_use]
    pub fn is_owner_or_admin(&self) -> bool {
        self.is_owner() || self.member.role == consts::auth::ROLE_ADMIN
    }

    /// 仅 owner 放行，否则以调用方提供的文案返回 403
    pub fn require_owner(self, message: &str) -> AppResult<Self> {
        if self.is_owner() {
            Ok(self)
        } else {
            Err(AppError::forbidden(message))
        }
    }

    /// owner 或 admin 放行，否则以调用方提供的文案返回 403
    pub fn require_owner_or_admin(self, message: &str) -> AppResult<Self> {
        if self.is_owner_or_admin() {
            Ok(self)
        } else {
            Err(AppError::forbidden(message))
        }
    }
}
