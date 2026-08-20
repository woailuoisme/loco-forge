use async_trait::async_trait;
use loco_rs::{auth::jwt, hash, prelude::*};
use sea_orm::{QueryOrder, entity::prelude::*};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use secrecy::ExposeSecret;

pub use super::_entities::users::{self, ActiveModel, Entity, Model};
pub use crate::dtos::auth::{LoginRequest, RegisterRequest};

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
        super::credentials::UserCredentials::verify_email_token(db, token).await
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
        super::credentials::UserCredentials::verify_otp(db, email, code).await
    }

    /// 根据密码重置 Token 查找用户对象并验证其有效性与过期时间
    ///
    /// # Errors
    ///
    /// 当未找到对应用户记录、Token 已过期或发生数据库查询错误时返回 `ModelError`
    pub async fn find_by_reset_token(db: &DatabaseConnection, token: &str) -> ModelResult<Self> {
        super::credentials::UserCredentials::verify_reset_token(db, token).await
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
        super::credentials::UserCredentials::issue_email_verification(&mut self);
        self.update(db).await.map_err(ModelError::from)
    }

    /// 设置忘记密码重置状态，生成唯一重置 Token 与时间戳并更新到数据库
    ///
    /// # Errors
    ///
    /// 当数据库更新失败时返回 `ModelError`
    pub async fn set_forgot_password_sent(mut self, db: &DatabaseConnection) -> ModelResult<Model> {
        super::credentials::UserCredentials::issue_password_reset(&mut self);
        self.update(db).await.map_err(ModelError::from)
    }

    /// 记录用户通过邮箱验证的时间戳并置空验证 Token（防重放）
    ///
    /// # Errors
    ///
    /// 当数据库更新失败时返回 `ModelError`
    pub async fn verified(mut self, db: &DatabaseConnection) -> ModelResult<Model> {
        super::credentials::UserCredentials::mark_email_verified(&mut self);
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
        super::credentials::UserCredentials::clear_password_reset(&mut self);
        self.update(db).await.map_err(ModelError::from)
    }

    /// 为 OTP 验证码登录生成 6 位纯数字验证码与过期时间戳并保存到数据库
    ///
    /// # Errors
    ///
    /// 当数据库更新失败时返回 `ModelError`
    pub async fn create_otp(mut self, db: &DatabaseConnection) -> ModelResult<(Model, String)> {
        let code = super::credentials::UserCredentials::issue_otp(&mut self);
        let model = self.update(db).await.map_err(ModelError::from)?;
        Ok((model, code))
    }

    /// 登录成功后清除 OTP 验证码与过期时间戳
    ///
    /// # Errors
    ///
    /// 当数据库更新失败时返回 `ModelError`
    pub async fn clear_otp(mut self, db: &DatabaseConnection) -> ModelResult<Model> {
        super::credentials::UserCredentials::clear_otp(&mut self);
        self.update(db).await.map_err(ModelError::from)
    }
}

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
        super::credentials::UserCredentials::clean_expired_tokens(db).await
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
