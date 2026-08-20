//! # 用户凭证与临时 Token 状态机 (User Credentials & Token State Machine)
//!
//! 统一管理 OTP 6位验证码、邮箱验证 Token 与密码重置 Token 的
//! 签发、过期时间核验、单次核销与数据库过期批量清理。

use chrono::{Duration, offset::Local};
use loco_rs::model::{ModelError, ModelResult};
use sea_orm::{
    ActiveValue, DatabaseConnection, EntityTrait, QueryFilter, entity::prelude::*, sea_query::Expr,
};
use uuid::Uuid;

use crate::{
    common::{
        consts::auth::{OTP_EXPIRATION_MIN, RESET_TOKEN_EXPIRATION_MIN},
        utils::CodeUtil,
    },
    models::_entities::users::{self, ActiveModel, Model},
};

/// 临时 Token 类别枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    /// 邮箱激活验证 Token
    EmailVerification,
    /// 密码重置 Token (默认 30 分钟)
    PasswordReset,
    /// 6 位纯数字 OTP 登录验证码 (默认 10 分钟)
    Otp,
}

/// 用户凭证与 Token 生命周期状态机
pub struct UserCredentials;

impl UserCredentials {
    /// 统一按类别签发 Token
    pub fn issue_token(kind: TokenKind, active: &mut ActiveModel) -> String {
        match kind {
            TokenKind::Otp => Self::issue_otp(active),
            TokenKind::EmailVerification => Self::issue_email_verification(active),
            TokenKind::PasswordReset => Self::issue_password_reset(active),
        }
    }

    /// 签发 6 位数字 OTP 登录验证码与过期时间
    pub fn issue_otp(active: &mut ActiveModel) -> String {
        let code = CodeUtil::generate_otp();
        let expired = Local::now() + Duration::minutes(OTP_EXPIRATION_MIN);

        active.magic_link_token = ActiveValue::set(Some(code.clone()));
        active.magic_link_expiration = ActiveValue::set(Some(expired.into()));
        code
    }

    /// 签发邮箱激活验证 Token
    pub fn issue_email_verification(active: &mut ActiveModel) -> String {
        let token = Uuid::new_v4().to_string();
        let now = Local::now();

        active.email_verification_token = ActiveValue::set(Some(token.clone()));
        active.email_verification_sent_at = ActiveValue::set(Some(now.into()));
        token
    }

    /// 签发密码重置 Token
    pub fn issue_password_reset(active: &mut ActiveModel) -> String {
        let token = Uuid::new_v4().to_string();
        let now = Local::now();

        active.reset_token = ActiveValue::set(Some(token.clone()));
        active.reset_sent_at = ActiveValue::set(Some(now.into()));
        token
    }

    /// 清除 OTP 验证码与过期时间
    pub fn clear_otp(active: &mut ActiveModel) {
        active.magic_link_token = ActiveValue::set(None);
        active.magic_link_expiration = ActiveValue::set(None);
    }

    /// 清除密码重置 Token 与发送时间
    pub fn clear_password_reset(active: &mut ActiveModel) {
        active.reset_token = ActiveValue::set(None);
        active.reset_sent_at = ActiveValue::set(None);
    }

    /// 清除邮箱验证 Token 并标记验证通过
    pub fn mark_email_verified(active: &mut ActiveModel) {
        let now = Local::now();
        active.email_verified_at = ActiveValue::set(Some(now.into()));
        active.email_verification_token = ActiveValue::set(None);
        active.email_verification_sent_at = ActiveValue::set(None);
    }

    /// 根据邮箱验证 Token 查找用户对象
    ///
    /// # Errors
    ///
    /// 当未找到对应用户记录或数据库查询错误时返回 `ModelError::EntityNotFound`
    pub async fn verify_email_token(db: &DatabaseConnection, token: &str) -> ModelResult<Model> {
        let user = users::Entity::find()
            .filter(users::Column::EmailVerificationToken.eq(token))
            .one(db)
            .await?;
        user.ok_or_else(|| ModelError::EntityNotFound)
    }

    /// 验证 OTP 验证码有效性并返回对应 Model
    ///
    /// # Errors
    ///
    /// 当验证码不存在或已过期时返回 `ModelError`
    pub async fn verify_otp(
        db: &DatabaseConnection,
        email: &str,
        code: &str,
    ) -> ModelResult<Model> {
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

    /// 验证重置密码 Token 有效性并返回对应 Model
    ///
    /// # Errors
    ///
    /// 当 Token 不存在或已过期时返回 `ModelError`
    pub async fn verify_reset_token(db: &DatabaseConnection, token: &str) -> ModelResult<Model> {
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
        let res1 = users::Entity::update_many()
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
        let res2 = users::Entity::update_many()
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
}
