//! # 用户模型与数据仓储 (Users Model & Repository)
//!
//! 遵循 Sea-ORM 3-part 模型结构：
//! 1. `impl Model`: 单实例读写与业务逻辑
//! 2. `impl ActiveModel`: 生命周期钩子与状态更新
//! 3. `impl Entity`: 集合查询、批量操作与分页

use async_trait::async_trait;
use chrono::{Duration, offset::Local};
use loco_rs::{auth::jwt, hash, prelude::*};
use sea_orm::{QueryOrder, entity::prelude::*, sea_query::Expr};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use secrecy::ExposeSecret;

pub use super::_entities::users::{self, ActiveModel, Entity, Model};
use crate::common::{
    consts::auth::{OTP_EXPIRATION_MIN, RESET_TOKEN_EXPIRATION_MIN},
    utils::CodeUtil,
};
pub use crate::dtos::auth::RegisterRequest;

/// 用户角色枚举定义
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Default,
    EnumIter,
    DeriveActiveEnum,
    Serialize,
    Deserialize,
    utoipa::ToSchema,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum UserRole {
    #[sea_orm(string_value = "user")]
    #[default]
    User,
    #[sea_orm(string_value = "admin")]
    Admin,
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::User => write!(f, "user"),
            Self::Admin => write!(f, "admin"),
        }
    }
}

/// 用户模型字段校验器
#[derive(Debug, Validate, Deserialize)]
pub struct Validator {
    #[validate(length(min = 2, message = "Name must be at least 2 characters long."))]
    pub name: String,
    #[validate(email(message = "invalid email"))]
    pub email: String,
}

impl Validatable for ActiveModel {
    fn validator(&self) -> Box<dyn Validate> {
        Box::new(Validator {
            name: self.name.as_ref().to_owned(),
            email: self.email.as_ref().to_owned(),
        })
    }
}

#[async_trait::async_trait]
impl ActiveModelBehavior for super::_entities::users::ActiveModel {
    async fn before_save<C>(self, _db: &C, insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if insert {
            self.validate()?;
            let mut this = self;
            this.pid = ActiveValue::Set(Uuid::new_v4());
            this.api_key = ActiveValue::Set(format!("lo-{}", Uuid::new_v4()));
            if !this.role.is_set() {
                this.role = ActiveValue::Set("user".to_string());
            }
            Ok(this)
        } else {
            Ok(self)
        }
    }
}

#[async_trait]
impl Authenticable for Model {
    async fn find_by_api_key(db: &DatabaseConnection, api_key: &str) -> ModelResult<Self> {
        Self::find_by_api_key(db, api_key).await
    }

    async fn find_by_claims_key(db: &DatabaseConnection, claims_key: &str) -> ModelResult<Self> {
        Self::find_by_pid(db, claims_key).await
    }
}

// =========================================================================
// ① 单实例读写 — find_by_xxx / 纯计算 / 业务方法
// =========================================================================

impl Model {
    /// 判断是否为管理员角色
    #[must_use]
    pub fn is_admin(&self) -> bool {
        self.role == "admin"
    }

    /// 判断是否包含指定角色
    #[must_use]
    pub fn has_role(&self, role: &str) -> bool {
        self.role == role
    }

    /// 根据电子邮箱地址查找用户对象
    ///
    /// # Errors
    ///
    /// 当未找到对应用户记录或发生数据库查询错误时返回 `ModelError::EntityNotFound`
    pub async fn find_by_email(db: &DatabaseConnection, email: &str) -> ModelResult<Self> {
        let user = users::Entity::find()
            .filter(users::Column::Email.eq(email))
            .one(db)
            .await?;
        user.ok_or_else(|| ModelError::EntityNotFound)
    }

    /// 根据邮箱验证 Token 查找用户对象
    ///
    /// # Errors
    ///
    /// 当未找到对应用户记录或发生数据库查询错误时返回 `ModelError::EntityNotFound`
    pub async fn find_by_verification_token(
        db: &DatabaseConnection,
        token: &str,
    ) -> ModelResult<Self> {
        let user = users::Entity::find()
            .filter(users::Column::EmailVerificationToken.eq(token))
            .one(db)
            .await?;
        user.ok_or_else(|| ModelError::EntityNotFound)
    }

    /// 根据 OTP 6 位验证码与邮箱查找用户对象并验证其有效性与过期时间
    ///
    /// # Errors
    ///
    /// 当未找到对应用户记录、验证码已过期或发生数据库查询错误时返回 `ModelError`
    pub async fn find_by_otp(
        db: &DatabaseConnection,
        email: &str,
        code: &str,
    ) -> ModelResult<Self> {
        let user = users::Entity::find()
            .filter(users::Column::Email.eq(email))
            .filter(users::Column::MagicLinkToken.eq(code))
            .one(db)
            .await?;

        let user = user.ok_or_else(|| ModelError::EntityNotFound)?;
        if let Some(expired_at) = user.magic_link_expiration {
            if expired_at >= Local::now() {
                Ok(user)
            } else {
                tracing::warn!(
                    user_pid = %user.pid,
                    token_expiration = %expired_at,
                    "OTP verification code expired for user"
                );
                Err(ModelError::msg("otp code expired"))
            }
        } else {
            tracing::warn!(user_pid = %user.pid, "OTP code expiration timestamp missing");
            Err(ModelError::msg("otp expiration token not exists"))
        }
    }

    /// 根据密码重置 Token 查找用户对象并验证其有效性与过期时间
    ///
    /// # Errors
    ///
    /// 当未找到对应用户记录、Token 已过期或发生数据库查询错误时返回 `ModelError`
    pub async fn find_by_reset_token(db: &DatabaseConnection, token: &str) -> ModelResult<Self> {
        let user = users::Entity::find()
            .filter(users::Column::ResetToken.eq(token))
            .one(db)
            .await?;

        let user = user.ok_or_else(|| ModelError::EntityNotFound)?;
        if let Some(sent_at) = user.reset_sent_at {
            if sent_at + Duration::minutes(RESET_TOKEN_EXPIRATION_MIN) >= Local::now() {
                Ok(user)
            } else {
                tracing::warn!(user_pid = %user.pid, "Reset token expired for user");
                Err(ModelError::msg("reset token expired"))
            }
        } else {
            tracing::warn!(user_pid = %user.pid, "Reset token sent timestamp missing");
            Err(ModelError::msg("reset token sent time not exists"))
        }
    }

    /// 根据 UUID 字符串 (PID) 查找用户对象
    ///
    /// # Errors
    ///
    /// 当 PID 无法解析为 UUID、未找到对应用户记录或发生数据库查询错误时返回 `ModelError`
    pub async fn find_by_pid(db: &DatabaseConnection, pid: &str) -> ModelResult<Self> {
        let parse_uuid = Uuid::parse_str(pid).map_err(|e| ModelError::Any(e.into()))?;
        let user = users::Entity::find()
            .filter(users::Column::Pid.eq(parse_uuid))
            .one(db)
            .await?;
        user.ok_or_else(|| ModelError::EntityNotFound)
    }

    /// 根据 API Key 查找用户对象
    ///
    /// # Errors
    ///
    /// 当未找到对应用户记录或发生数据库查询错误时返回 `ModelError::EntityNotFound`
    pub async fn find_by_api_key(db: &DatabaseConnection, api_key: &str) -> ModelResult<Self> {
        let user = users::Entity::find()
            .filter(users::Column::ApiKey.eq(api_key))
            .one(db)
            .await?;
        user.ok_or_else(|| ModelError::EntityNotFound)
    }

    /// 校验明文密码是否与哈希密码匹配
    #[must_use]
    pub fn verify_password(&self, password: &str) -> bool {
        hash::verify_password(password, &self.password)
    }

    /// 异步创建包含密码哈希的新用户记录并持久化到数据库
    ///
    /// # Errors
    ///
    /// 当邮箱已被占用或数据库写入失败时返回 `ModelError`
    pub async fn create_with_password(
        db: &DatabaseConnection,
        params: &RegisterRequest,
    ) -> ModelResult<Self> {
        let txn = db.begin().await?;

        if users::Entity::find()
            .filter(users::Column::Email.eq(&params.email))
            .one(&txn)
            .await?
            .is_some()
        {
            return Err(ModelError::EntityAlreadyExists {});
        }

        let password_hash = hash::hash_password(params.password.expose_secret())
            .map_err(|e| ModelError::Any(e.into()))?;
        let user = users::ActiveModel {
            email: ActiveValue::set(params.email.clone()),
            password: ActiveValue::set(password_hash),
            name: ActiveValue::set(params.name.clone()),
            ..Default::default()
        }
        .insert(&txn)
        .await?;

        txn.commit().await?;

        Ok(user)
    }

    /// 为当前用户生成指定有效期的 JWT 认证 Token
    ///
    /// # Errors
    ///
    /// 当生成 JWT Claims 错误或签署签名失败时返回 `ModelError`
    pub fn generate_jwt(&self, secret: &str, expiration: u64) -> ModelResult<String> {
        jwt::JWT::new(secret)
            .generate_token(expiration, self.pid.to_string(), serde_json::Map::new())
            .map_err(ModelError::from)
    }

    /// 清理数据库中已过期的重置密码 Token 与 OTP 验证码（转发至 Entity 集合方法）
    ///
    /// # Errors
    ///
    /// 当数据库查询或更新失败时返回 `ModelError`
    pub async fn clean_expired_tokens(db: &DatabaseConnection) -> ModelResult<u64> {
        Entity::clean_expired_tokens(db).await
    }
}

// =========================================================================
// ② ActiveModel 钩子与字段级状态流转方法
// =========================================================================

impl ActiveModel {
    /// 设置邮箱验证发送状态，生成唯一验证 Token 与时间戳并更新到数据库
    ///
    /// # Errors
    ///
    /// 当数据库更新失败时返回 `ModelError`
    pub async fn set_email_verification_sent(
        mut self,
        db: &DatabaseConnection,
    ) -> ModelResult<Model> {
        let token = Uuid::new_v4().to_string();
        let now = Local::now();
        self.email_verification_token = ActiveValue::set(Some(token));
        self.email_verification_sent_at = ActiveValue::set(Some(now.into()));
        self.update(db).await.map_err(ModelError::from)
    }

    /// 设置忘记密码重置状态，生成唯一重置 Token 与时间戳并更新到数据库
    ///
    /// # Errors
    ///
    /// 当数据库更新失败时返回 `ModelError`
    pub async fn set_forgot_password_sent(mut self, db: &DatabaseConnection) -> ModelResult<Model> {
        let token = Uuid::new_v4().to_string();
        let now = Local::now();
        self.reset_token = ActiveValue::set(Some(token));
        self.reset_sent_at = ActiveValue::set(Some(now.into()));
        self.update(db).await.map_err(ModelError::from)
    }

    /// 记录用户通过邮箱验证的时间戳并置空验证 Token（防重放）
    ///
    /// # Errors
    ///
    /// 当数据库更新失败时返回 `ModelError`
    pub async fn verified(mut self, db: &DatabaseConnection) -> ModelResult<Model> {
        let now = Local::now();
        self.email_verified_at = ActiveValue::set(Some(now.into()));
        self.email_verification_token = ActiveValue::set(None);
        self.email_verification_sent_at = ActiveValue::set(None);
        self.update(db).await.map_err(ModelError::from)
    }

    /// 使用新密码对当前用户进行重置，哈希化密码并清除重置 Token
    ///
    /// # Errors
    ///
    /// 当密码哈希计算失败或数据库更新失败时返回 `ModelError`
    pub async fn reset_password(
        mut self,
        db: &DatabaseConnection,
        password: &str,
    ) -> ModelResult<Model> {
        self.password =
            ActiveValue::set(hash::hash_password(password).map_err(|e| ModelError::Any(e.into()))?);
        self.reset_token = ActiveValue::set(None);
        self.reset_sent_at = ActiveValue::set(None);
        self.update(db).await.map_err(ModelError::from)
    }

    /// 为 OTP 验证码登录生成 6 位纯数字验证码与过期时间戳并保存到数据库
    ///
    /// # Errors
    ///
    /// 当数据库更新失败时返回 `ModelError`
    pub async fn create_otp(mut self, db: &DatabaseConnection) -> ModelResult<(Model, String)> {
        let code = CodeUtil::generate_otp();
        let expired = Local::now() + Duration::minutes(OTP_EXPIRATION_MIN);
        self.magic_link_token = ActiveValue::set(Some(code.clone()));
        self.magic_link_expiration = ActiveValue::set(Some(expired.into()));
        let model = self.update(db).await.map_err(ModelError::from)?;
        Ok((model, code))
    }

    /// 登录成功后清除 OTP 验证码与过期时间戳
    ///
    /// # Errors
    ///
    /// 当数据库更新失败时返回 `ModelError`
    pub async fn clear_otp(mut self, db: &DatabaseConnection) -> ModelResult<Model> {
        self.magic_link_token = ActiveValue::set(None);
        self.magic_link_expiration = ActiveValue::set(None);
        self.update(db).await.map_err(ModelError::from)
    }
}

// =========================================================================
// ③ Entity 集合查询 — 分页、批量删除与过期清理
// =========================================================================

impl Entity {
    /// 按创建时间倒序分页获取用户列表
    ///
    /// # Errors
    ///
    /// 当数据库分页查询失败时返回 `ModelError`
    pub async fn find_all_paginated(
        db: &DatabaseConnection,
        page: u64,
        page_size: u64,
    ) -> ModelResult<(Vec<Model>, u64)> {
        use super::repository::QueryPaginationExt;

        Self::find()
            .order_by_desc(users::Column::CreatedAt)
            .paginate_with_total(db, page, page_size)
            .await
    }

    /// 清理数据库中已过期的重置密码 Token 与 OTP 验证码（单条 SQL 批量优化）
    ///
    /// # Errors
    ///
    /// 当数据库查询或更新失败时返回 `ModelError`
    pub async fn clean_expired_tokens(db: &DatabaseConnection) -> ModelResult<u64> {
        let now = Local::now();
        let reset_threshold = now - Duration::minutes(RESET_TOKEN_EXPIRATION_MIN);

        let mut cleaned_count = 0u64;

        // 批量置空过期的 reset_token
        let res1 = Self::update_many()
            .col_expr(
                users::Column::ResetToken,
                Expr::value(Option::<String>::None),
            )
            .col_expr(
                users::Column::ResetSentAt,
                Expr::value(Option::<chrono::DateTime<chrono::FixedOffset>>::None),
            )
            .filter(users::Column::ResetSentAt.lt(reset_threshold))
            .exec(db)
            .await?;
        cleaned_count += res1.rows_affected;

        // 批量置空过期的 OTP (magic_link_token)
        let res2 = Self::update_many()
            .col_expr(
                users::Column::MagicLinkToken,
                Expr::value(Option::<String>::None),
            )
            .col_expr(
                users::Column::MagicLinkExpiration,
                Expr::value(Option::<chrono::DateTime<chrono::FixedOffset>>::None),
            )
            .filter(users::Column::MagicLinkExpiration.lt(now))
            .exec(db)
            .await?;
        cleaned_count += res2.rows_affected;

        Ok(cleaned_count)
    }

    /// 根据 ID 列表批量删除用户记录（单条 SQL，零 N+1，空切片防护）
    ///
    /// # Errors
    ///
    /// 当数据库删除操作失败时返回 `ModelError`
    pub async fn delete_by_ids(db: &DatabaseConnection, ids: &[i64]) -> ModelResult<u64> {
        if ids.is_empty() {
            return Ok(0);
        }

        let res = Self::delete_many()
            .filter(users::Column::Id.is_in(ids.to_vec()))
            .exec(db)
            .await?;

        tracing::info!(
            count = res.rows_affected,
            "Successfully deleted users by IDs"
        );

        Ok(res.rows_affected)
    }
}
