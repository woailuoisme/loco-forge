use chrono::Utc;
use loco_rs::prelude::*;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseConnection,
    EntityTrait, IntoActiveModel, QueryFilter,
};
use uuid::Uuid;

use crate::common::consts;

pub use super::_entities::user::{ActiveModel, Column, Entity, Model};
pub type User = Entity;

/// 管理端用户列表查询参数 (由 Service 层从 [`crate::dtos::admin::AdminListUsersQuery`] 归一化而来)
#[derive(Debug, Clone, Default)]
pub struct AdminUserQueryParams<'a> {
    /// 每页数量 (已 clamp)
    pub page_size: u64,
    /// SQL offset 偏移量
    pub offset: u64,
    /// 模糊搜索关键词 (匹配 email / name / username)
    pub search: Option<&'a str>,
    /// 角色精确过滤
    pub role: Option<&'a str>,
    /// 状态精确过滤 (`active` | `inactive` | `banned`)
    pub status: Option<&'a str>,
    /// 封禁状态过滤
    pub banned: Option<bool>,
    /// 排序字段 (`createdAt` | `updatedAt` | `email` | `name` | `lastLoginAt`)
    pub sort_by: Option<&'a str>,
    /// 是否包含已软删用户
    pub include_deleted: bool,
}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(self, _db: &C, insert: bool) -> std::result::Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        let mut this = self;
        let now = Utc::now().into();
        if insert {
            if this.id.is_not_set() {
                this.id = Set(Uuid::now_v7().to_string());
            }
            if this.created_at.is_not_set() {
                this.created_at = Set(now);
            }
            this.updated_at = Set(now);
        } else if this.updated_at.is_unchanged() {
            this.updated_at = Set(now);
        }
        Ok(this)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ① 单实例读写方法 (Model)
// ─────────────────────────────────────────────────────────────────────────────
impl Model {
    /// 检查用户是否具备管理员角色
    #[must_use]
    pub fn is_admin(&self) -> bool {
        self.role == consts::auth::ROLE_ADMIN
    }

    /// 管理员闸门：非管理员直接 403，调用方无需重复写判定
    pub fn require_admin(&self) -> crate::errors::AppResult<()> {
        if self.is_admin() {
            Ok(())
        } else {
            Err(crate::errors::AppError::forbidden("Admin access required"))
        }
    }

    /// 检查用户是否已被封禁
    #[must_use]
    pub fn is_banned(&self) -> bool {
        if !self.banned {
            return false;
        }
        if let Some(ban_expires) = self.ban_expires {
            let now: chrono::DateTime<chrono::FixedOffset> = Utc::now().into();
            ban_expires > now
        } else {
            true
        }
    }

    /// 检查用户是否已被软删除 (注销)
    #[must_use]
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    /// 检查用户状态是否正常活跃 (未被封禁、未被软删除且状态为 active)
    #[must_use]
    pub fn is_active(&self) -> bool {
        !self.is_banned() && !self.is_deleted() && self.status == "active"
    }

    /// 更新用户最后登录方式与登录时间
    pub async fn update_last_login(
        &self,
        db: &DatabaseConnection,
        method: &str,
    ) -> ModelResult<Self> {
        let mut active = self.clone().into_active_model();
        active.last_login_method = Set(Some(method.to_string()));
        active.last_login_at = Set(Some(Utc::now().into()));
        let updated = active.update(db).await?;
        Ok(updated)
    }

    /// 匿名用户无缝就地转正
    pub async fn upgrade_anonymous(
        &self,
        db: &DatabaseConnection,
        email: &str,
        name: &str,
        username: Option<&str>,
    ) -> ModelResult<Self> {
        let mut active = self.clone().into_active_model();
        active.email = Set(email.to_lowercase());
        active.name = Set(name.to_string());
        active.is_anonymous = Set(false);
        if let Some(uname) = username {
            active.username = Set(Some(uname.to_lowercase()));
            active.display_username = Set(Some(uname.to_string()));
        }
        let updated = active.update(db).await?;
        Ok(updated)
    }

    /// 封禁或解封用户
    pub async fn set_banned(
        &self,
        db: &DatabaseConnection,
        banned: bool,
        reason: Option<&str>,
        expires: Option<chrono::DateTime<chrono::FixedOffset>>,
    ) -> ModelResult<Self> {
        let mut active = self.clone().into_active_model();
        active.banned = Set(banned);
        active.ban_reason = Set(reason.map(ToString::to_string));
        active.ban_expires = Set(expires);
        let updated = active.update(db).await?;
        Ok(updated)
    }

    /// 设置用户的两步验证开启状态
    pub async fn set_two_factor_enabled<C>(&self, db: &C, enabled: bool) -> ModelResult<Self>
    where
        C: ConnectionTrait,
    {
        let mut active = self.clone().into_active_model();
        active.two_factor_enabled = Set(enabled);
        let updated = active.update(db).await?;
        Ok(updated)
    }

    /// 设置用户邮箱验证状态
    pub async fn set_email_verified(
        &self,
        db: &DatabaseConnection,
        verified: bool,
    ) -> ModelResult<Self> {
        let mut active = self.clone().into_active_model();
        active.email_verified = Set(verified);
        let updated = active.update(db).await?;
        Ok(updated)
    }

    /// 设置并更新用户手机号与验证状态
    pub async fn set_phone_number(
        &self,
        db: &DatabaseConnection,
        phone_number: &str,
        verified: bool,
    ) -> ModelResult<Self> {
        let mut active = self.clone().into_active_model();
        active.phone_number = Set(Some(phone_number.to_string()));
        active.phone_number_verified = Set(verified);
        let updated = active.update(db).await?;
        Ok(updated)
    }

    /// 设置并更新用户最近登录方式
    pub async fn set_last_login_method(
        &self,
        db: &DatabaseConnection,
        method: &str,
    ) -> ModelResult<Self> {
        let mut active = self.clone().into_active_model();
        active.last_login_method = Set(Some(method.to_string()));
        active.last_login_at = Set(Some(Utc::now().into()));
        let updated = active.update(db).await?;
        Ok(updated)
    }

    /// 设置并更新用户全局角色
    pub async fn set_role(&self, db: &DatabaseConnection, role: &str) -> ModelResult<Self> {
        let mut active = self.clone().into_active_model();
        active.role = Set(role.to_string());
        active.updated_at = Set(Utc::now().into());
        let updated = active.update(db).await?;
        Ok(updated)
    }

    /// 设置并更新用户账号状态 (`active` | `inactive` | `banned`)
    pub async fn set_status(&self, db: &DatabaseConnection, status: &str) -> ModelResult<Self> {
        let mut active = self.clone().into_active_model();
        active.status = Set(status.to_string());
        let updated = active.update(db).await?;
        Ok(updated)
    }

    /// 软删除 (注销) 用户：仅写入 `deleted_at`，保留全部数据供审计。
    ///
    /// 注意：`email` / `username` / `phone_number` 的 partial 唯一索引均带
    /// `WHERE deleted_at IS NULL`，因此软删后这些标识**立即释放**，可被新用户注册。
    pub async fn soft_delete(&self, db: &impl ConnectionTrait) -> ModelResult<Self> {
        let mut active = self.clone().into_active_model();
        active.deleted_at = Set(Some(Utc::now().into()));
        let updated = active.update(db).await?;
        Ok(updated)
    }

    /// 按管理端请求更新用户可编辑资料字段。
    ///
    /// 各参数遵循 `api.md` §2.7 的 PATCH 三态语义：
    /// - `None` 表示字段缺省，保持原值不变；
    /// - `Some(None)` 表示显式置空；
    /// - `Some(Some(v))` 表示覆盖为新值。
    pub async fn update_profile(
        &self,
        db: &DatabaseConnection,
        name: Option<String>,
        role: Option<String>,
        status: Option<String>,
        username: Option<Option<String>>,
        image: Option<Option<String>>,
        email_verified: Option<bool>,
    ) -> ModelResult<Self> {
        let mut active = self.clone().into_active_model();
        if let Some(v) = name {
            active.name = Set(v);
        }
        if let Some(v) = role {
            active.role = Set(v);
        }
        if let Some(v) = status {
            active.status = Set(v);
        }
        if let Some(v) = username {
            active.username = Set(v.as_ref().map(|u| u.to_lowercase()));
            active.display_username = Set(v);
        }
        if let Some(v) = image {
            active.image = Set(v);
        }
        if let Some(v) = email_verified {
            active.email_verified = Set(v);
        }
        let updated = active.update(db).await?;
        Ok(updated)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ② ActiveModel 钩子与字段行为
// ─────────────────────────────────────────────────────────────────────────────
impl ActiveModel {}

// ─────────────────────────────────────────────────────────────────────────────
// ③ 集合级查询与创建 (Entity)
// ─────────────────────────────────────────────────────────────────────────────
impl Entity {
    /// 根据主键 ID 查找用户 (排除已软删用户)
    pub async fn find_by_id(db: &DatabaseConnection, id: &str) -> ModelResult<Option<Model>> {
        let user = Self::find()
            .filter(Column::Id.eq(id))
            .filter(Column::DeletedAt.is_null())
            .one(db)
            .await?;
        Ok(user)
    }

    /// 根据主键 ID 查找用户 (**包含**已软删用户，仅供管理端审计使用)
    pub async fn find_by_id_with_deleted(
        db: &DatabaseConnection,
        id: &str,
    ) -> ModelResult<Option<Model>> {
        let user = Self::find().filter(Column::Id.eq(id)).one(db).await?;
        Ok(user)
    }

    /// 根据邮箱查找用户（邮箱大小写无关，排除已软删用户）
    pub async fn find_by_email(db: &DatabaseConnection, email: &str) -> ModelResult<Option<Model>> {
        let user = Self::find()
            .filter(Column::Email.eq(email.to_lowercase()))
            .filter(Column::DeletedAt.is_null())
            .one(db)
            .await?;
        Ok(user)
    }

    /// 根据用户名查找用户（用户名大小写无关，排除已软删用户）
    pub async fn find_by_username(
        db: &DatabaseConnection,
        username: &str,
    ) -> ModelResult<Option<Model>> {
        let user = Self::find()
            .filter(Column::Username.eq(username.to_lowercase()))
            .filter(Column::DeletedAt.is_null())
            .one(db)
            .await?;
        Ok(user)
    }

    /// 根据手机号码查找用户 (排除已软删用户)
    pub async fn find_by_phone_number(
        db: &DatabaseConnection,
        phone_number: &str,
    ) -> ModelResult<Option<Model>> {
        let user = Self::find()
            .filter(Column::PhoneNumber.eq(phone_number))
            .filter(Column::DeletedAt.is_null())
            .one(db)
            .await?;
        Ok(user)
    }

    /// 创建普通邮箱注册用户
    pub async fn create_user<C>(
        db: &C,
        email: &str,
        name: &str,
        role: Option<&str>,
        image: Option<String>,
        username: Option<String>,
    ) -> ModelResult<Model>
    where
        C: ConnectionTrait,
    {
        let now = Utc::now().into();
        let active = ActiveModel {
            id: Set(Uuid::now_v7().to_string()),
            name: Set(name.to_string()),
            email: Set(email.to_lowercase()),
            email_verified: Set(false),
            image: Set(image),
            role: Set(role.unwrap_or("user").to_string()),
            status: Set("active".to_string()),
            banned: Set(false),
            two_factor_enabled: Set(false),
            username: Set(username.as_ref().map(|u| u.to_lowercase())),
            display_username: Set(username),
            phone_number: Set(None),
            phone_number_verified: Set(false),
            is_anonymous: Set(false),
            last_login_method: Set(Some("email".to_string())),
            ban_reason: Set(None),
            ban_expires: Set(None),
            last_login_at: Set(Some(now)),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
        };
        let user = active.insert(db).await?;
        Ok(user)
    }

    /// 创建通过手机号注册的用户
    pub async fn create_phone_user(
        db: &DatabaseConnection,
        phone_number: &str,
    ) -> ModelResult<Model> {
        let now = Utc::now().into();
        let anon_id = Uuid::now_v7().to_string();
        let suffix = if phone_number.len() >= 4 {
            &phone_number[phone_number.len() - 4..]
        } else {
            phone_number
        };
        let display_name = format!("User {suffix}");
        let placeholder_email = format!("phone_{anon_id}@phone.local");

        let active = ActiveModel {
            id: Set(anon_id),
            name: Set(display_name),
            email: Set(placeholder_email),
            email_verified: Set(false),
            image: Set(None),
            role: Set("user".to_string()),
            status: Set("active".to_string()),
            banned: Set(false),
            two_factor_enabled: Set(false),
            username: Set(None),
            display_username: Set(None),
            phone_number: Set(Some(phone_number.to_string())),
            phone_number_verified: Set(true),
            is_anonymous: Set(false),
            last_login_method: Set(Some("phone-number".to_string())),
            ban_reason: Set(None),
            ban_expires: Set(None),
            last_login_at: Set(Some(now)),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
        };
        let user = active.insert(db).await?;
        Ok(user)
    }

    /// 创建匿名会话用户
    pub async fn create_anonymous(
        db: &DatabaseConnection,
        name: Option<&str>,
    ) -> ModelResult<Model> {
        let now = Utc::now().into();
        let anon_id = Uuid::now_v7().to_string();
        let short_id = &anon_id[..8];
        let user_name = name.map_or_else(|| format!("Anonymous {short_id}"), ToString::to_string);
        let placeholder_email = format!("anon_{anon_id}@anonymous.local");

        let active = ActiveModel {
            id: Set(anon_id),
            name: Set(user_name),
            email: Set(placeholder_email),
            email_verified: Set(false),
            image: Set(None),
            role: Set("user".to_string()),
            status: Set("active".to_string()),
            banned: Set(false),
            two_factor_enabled: Set(false),
            username: Set(None),
            display_username: Set(None),
            phone_number: Set(None),
            phone_number_verified: Set(false),
            is_anonymous: Set(true),
            last_login_method: Set(Some("anonymous".to_string())),
            ban_reason: Set(None),
            ban_expires: Set(None),
            last_login_at: Set(Some(now)),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
        };
        let user = active.insert(db).await?;
        Ok(user)
    }

    /// 分页查询所有用户 (排除已软删用户)
    pub async fn find_all_paginated(
        db: &DatabaseConnection,
        limit: u64,
        offset: u64,
        search_value: Option<&str>,
    ) -> ModelResult<(Vec<Model>, u64)> {
        use sea_orm::{Condition, PaginatorTrait, QueryOrder, QuerySelect};
        let mut query = Self::find().filter(Column::DeletedAt.is_null());
        if let Some(val) = search_value {
            // contains 自带 % 包裹，避免 %%val%% 双重通配与通配符注入
            query = query.filter(
                Condition::any()
                    .add(Column::Email.contains(val))
                    .add(Column::Name.contains(val)),
            );
        }
        let total = query.clone().count(db).await?;
        let users = query
            .order_by_desc(Column::CreatedAt)
            .limit(limit)
            .offset(offset)
            .all(db)
            .await?;
        Ok((users, total))
    }

    /// 管理端用户列表分页查询，支持过滤 (角色 / 状态 / 封禁) 与排序。
    ///
    /// - `include_deleted = false` 时隐藏已软删用户 (默认)；
    /// - `sort_by` 支持 `createdAt` | `updatedAt` | `email` | `name` | `lastLoginAt`；
    /// - 排序始终追加 `id DESC` 以保证分页遍历的确定性 (避免 Page Drift)。
    pub async fn find_all_admin_paginated(
        db: &DatabaseConnection,
        params: &AdminUserQueryParams<'_>,
    ) -> ModelResult<(Vec<Model>, u64)> {
        use sea_orm::{Condition, PaginatorTrait, QueryOrder, QuerySelect};

        let mut query = Self::find();

        if !params.include_deleted {
            query = query.filter(Column::DeletedAt.is_null());
        }
        if let Some(role) = params.role {
            query = query.filter(Column::Role.eq(role));
        }
        if let Some(status) = params.status {
            query = query.filter(Column::Status.eq(status));
        }
        if let Some(banned) = params.banned {
            query = query.filter(Column::Banned.eq(banned));
        }
        if let Some(val) = params.search {
            // contains 自带 % 包裹，避免 %%val%% 双重通配与通配符注入
            query = query.filter(
                Condition::any()
                    .add(Column::Email.contains(val))
                    .add(Column::Name.contains(val))
                    .add(Column::Username.contains(val)),
            );
        }

        let total = query.clone().count(db).await?;

        let (column, desc) = match params.sort_by.unwrap_or("createdAt") {
            "updatedAt" => (Column::UpdatedAt, true),
            "email" => (Column::Email, false),
            "name" => (Column::Name, false),
            "lastLoginAt" => (Column::LastLoginAt, true),
            _ => (Column::CreatedAt, true),
        };
        let query = if desc {
            query.order_by_desc(column)
        } else {
            query.order_by_asc(column)
        };

        let users = query
            .order_by_desc(Column::Id)
            .limit(params.page_size)
            .offset(params.offset)
            .all(db)
            .await?;
        Ok((users, total))
    }
}
