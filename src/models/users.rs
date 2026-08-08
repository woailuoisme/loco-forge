use async_trait::async_trait;
use chrono::{Duration, offset::Local};
use loco_rs::{auth::jwt, hash, prelude::*};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Map;
use uuid::Uuid;

use secrecy::ExposeSecret;

pub use super::_entities::users::{self, ActiveModel, Entity, Model};
pub use crate::dtos::auth::{LoginRequest, RegisterRequest};

/// 魔术链接 Token 随机字符串长度
pub const MAGIC_LINK_LENGTH: i8 = 32;
/// 魔术链接 Token 过期时间 (分钟)
pub const MAGIC_LINK_EXPIRATION_MIN: i8 = 5;
/// 重置密码 Token 过期时间 (分钟)
pub const RESET_TOKEN_EXPIRATION_MIN: i64 = 30;

/// 用户角色枚举定义
#[derive(
    Debug, Clone, PartialEq, Eq, Default, EnumIter, DeriveActiveEnum, Serialize, Deserialize, utoipa::ToSchema,
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
    pub async fn find_by_verification_token(db: &DatabaseConnection, token: &str) -> ModelResult<Self> {
        let user = users::Entity::find()
            .filter(users::Column::EmailVerificationToken.eq(token))
            .one(db)
            .await?;
        user.ok_or_else(|| ModelError::EntityNotFound)
    }

    /// 根据魔术链接 Token 查找用户对象并验证其有效性与过期时间
    ///
    /// # Errors
    ///
    /// 当未找到对应用户记录、Token 已过期或发生数据库查询错误时返回 `ModelError`
    pub async fn find_by_magic_token(db: &DatabaseConnection, token: &str) -> ModelResult<Self> {
        let user = users::Entity::find()
            .filter(users::Column::MagicLinkToken.eq(token))
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
                    "Magic token expired for user"
                );
                Err(ModelError::msg("magic token expired"))
            }
        } else {
            tracing::warn!(user_pid = %user.pid, "Magic link expiration timestamp missing");
            Err(ModelError::msg("expiration token not exists"))
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
    pub async fn create_with_password(db: &DatabaseConnection, params: &RegisterRequest) -> ModelResult<Self> {
        let txn = db.begin().await?;

        if users::Entity::find()
            .filter(users::Column::Email.eq(&params.email))
            .one(&txn)
            .await?
            .is_some()
        {
            return Err(ModelError::EntityAlreadyExists {});
        }

        let password_hash =
            hash::hash_password(params.password.expose_secret()).map_err(|e| ModelError::Any(e.into()))?;
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
        let _ = jsonwebtoken::crypto::rust_crypto::DEFAULT_PROVIDER.install_default();
        jwt::JWT::new(secret)
            .generate_token(expiration, self.pid.to_string(), Map::new())
            .map_err(ModelError::from)
    }

    /// 清理数据库中已过期的重置密码 Token 与魔术链接 Token（单条 SQL 批量优化）
    ///
    /// # Errors
    ///
    /// 当数据库查询或更新失败时返回 `ModelError`
    pub async fn clean_expired_tokens(db: &DatabaseConnection) -> ModelResult<u64> {
        use sea_orm::sea_query::Expr;

        let now = Local::now();
        let reset_threshold = now - Duration::minutes(RESET_TOKEN_EXPIRATION_MIN);

        let mut cleaned_count = 0u64;

        // 批量置空过期的 reset_token
        let res1 = users::Entity::update_many()
            .col_expr(users::Column::ResetToken, Expr::value(Option::<String>::None))
            .col_expr(
                users::Column::ResetSentAt,
                Expr::value(Option::<chrono::DateTime<chrono::FixedOffset>>::None),
            )
            .filter(users::Column::ResetSentAt.lt(reset_threshold))
            .exec(db)
            .await?;
        cleaned_count += res1.rows_affected;

        // 批量置空过期的 magic_link_token
        let res2 = users::Entity::update_many()
            .col_expr(users::Column::MagicLinkToken, Expr::value(Option::<String>::None))
            .col_expr(
                users::Column::MagicLinkExpiration,
                Expr::value(Option::<chrono::DateTime<chrono::FixedOffset>>::None),
            )
            .filter(users::Column::MagicLinkExpiration.lt(now))
            .exec(db)
            .await?;
        cleaned_count += res2.rows_affected;

        tracing::info!(
            cleaned_count = cleaned_count,
            "Successfully cleaned expired tokens in batch"
        );

        Ok(cleaned_count)
    }
}

impl ActiveModel {
    /// 设置邮箱验证发送状态，生成唯一验证 Token 与时间戳并更新到数据库
    ///
    /// # Errors
    ///
    /// 当数据库更新失败时返回 `ModelError`
    pub async fn set_email_verification_sent(mut self, db: &DatabaseConnection) -> ModelResult<Model> {
        self.email_verification_sent_at = ActiveValue::set(Some(Local::now().into()));
        self.email_verification_token = ActiveValue::Set(Some(Uuid::new_v4().to_string()));
        self.update(db).await.map_err(ModelError::from)
    }

    /// 设置忘记密码重置状态，生成唯一重置 Token 与时间戳并更新到数据库
    ///
    /// # Errors
    ///
    /// 当数据库更新失败时返回 `ModelError`
    pub async fn set_forgot_password_sent(mut self, db: &DatabaseConnection) -> ModelResult<Model> {
        self.reset_sent_at = ActiveValue::set(Some(Local::now().into()));
        self.reset_token = ActiveValue::Set(Some(Uuid::new_v4().to_string()));
        self.update(db).await.map_err(ModelError::from)
    }

    /// 记录用户通过邮箱验证的时间戳并置空验证 Token（防重放）
    ///
    /// # Errors
    ///
    /// 当数据库更新失败时返回 `ModelError`
    pub async fn verified(mut self, db: &DatabaseConnection) -> ModelResult<Model> {
        self.email_verified_at = ActiveValue::set(Some(Local::now().into()));
        // 使验证 Token 失效，避免二次防重放
        self.email_verification_token = ActiveValue::Set(None);
        self.update(db).await.map_err(ModelError::from)
    }

    /// 使用新密码对当前用户进行重置，哈希化密码并清除重置 Token
    ///
    /// # Errors
    ///
    /// 当密码哈希计算失败或数据库更新失败时返回 `ModelError`
    pub async fn reset_password(mut self, db: &DatabaseConnection, password: &str) -> ModelResult<Model> {
        self.password = ActiveValue::set(hash::hash_password(password).map_err(|e| ModelError::Any(e.into()))?);
        self.reset_token = ActiveValue::Set(None);
        self.reset_sent_at = ActiveValue::Set(None);
        self.update(db).await.map_err(ModelError::from)
    }

    /// 为无密码登录生成随机魔术链接 Token 与过期时间戳并保存到数据库
    ///
    /// # Errors
    ///
    /// 当数据库更新失败时返回 `ModelError`
    pub async fn create_magic_link(mut self, db: &DatabaseConnection) -> ModelResult<Model> {
        let random_str = hash::random_string(MAGIC_LINK_LENGTH as usize);
        let expired = Local::now() + Duration::minutes(MAGIC_LINK_EXPIRATION_MIN.into());

        self.magic_link_token = ActiveValue::set(Some(random_str));
        self.magic_link_expiration = ActiveValue::set(Some(expired.into()));
        self.update(db).await.map_err(ModelError::from)
    }

    /// 登录成功后清除魔术链接 Token 与过期时间戳
    ///
    /// # Errors
    ///
    /// 当数据库更新失败时返回 `ModelError`
    pub async fn clear_magic_link(mut self, db: &DatabaseConnection) -> ModelResult<Model> {
        self.magic_link_token = ActiveValue::set(None);
        self.magic_link_expiration = ActiveValue::set(None);
        self.update(db).await.map_err(ModelError::from)
    }
}
