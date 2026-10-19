pub use super::_entities::verification::{ActiveModel, Column, Entity, Model};
use chrono::{Duration, Utc};
use sea_orm::{QueryFilter, Set, entity::prelude::*};
use uuid::Uuid;

pub type Verification = Entity;

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(self, _db: &C, insert: bool) -> std::result::Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if !insert && self.updated_at.is_unchanged() {
            let mut this = self;
            this.updated_at = sea_orm::ActiveValue::Set(Utc::now().into());
            Ok(this)
        } else {
            Ok(self)
        }
    }
}

// ① 单实例读写
impl Model {
    /// 检查该凭据是否已过期
    #[must_use]
    pub fn is_expired(&self) -> bool {
        self.expires_at < Utc::now()
    }

    /// 消费并删除当前验证凭据
    pub async fn consume<C>(&self, db: &C) -> Result<(), DbErr>
    where
        C: ConnectionTrait,
    {
        Entity::delete_by_id(self.id.clone()).exec(db).await?;
        Ok(())
    }
}

// ② 钩子行为与生命周期
impl ActiveModel {}

// ③ 集合查询与管理
impl Entity {
    /// 根据唯一 identifier 查找有效且未过期的凭据
    pub async fn find_valid_by_identifier<C>(
        db: &C,
        identifier: &str,
    ) -> Result<Option<Model>, DbErr>
    where
        C: ConnectionTrait,
    {
        let now = Utc::now();
        Self::find()
            .filter(Column::Identifier.eq(identifier))
            .filter(Column::ExpiresAt.gt(now))
            .one(db)
            .await
    }

    /// 创建新的验证凭据，若已存在相同 identifier 则先删除覆盖
    pub async fn create_record<C>(
        db: &C,
        identifier: &str,
        value: &str,
        expires_in_seconds: i64,
    ) -> Result<Model, DbErr>
    where
        C: ConnectionTrait,
    {
        // 幂等清理同一 identifier 的历史旧凭据（清理失败则中止，避免旧凭据残留有效）
        Self::delete_many()
            .filter(Column::Identifier.eq(identifier))
            .exec(db)
            .await?;

        let now = Utc::now();
        let expires_at = now + Duration::seconds(expires_in_seconds);

        let active = ActiveModel {
            id: Set(Uuid::new_v4().to_string()),
            identifier: Set(identifier.to_string()),
            value: Set(value.to_string()),
            expires_at: Set(expires_at.into()),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
        };

        active.insert(db).await
    }

    /// 查找有效且未过期的验证记录
    pub async fn find_valid<C>(
        db: &C,
        identifier: &str,
        value: &str,
    ) -> Result<Option<Model>, DbErr>
    where
        C: ConnectionTrait,
    {
        let now = Utc::now();
        Self::find()
            .filter(Column::Identifier.eq(identifier))
            .filter(Column::Value.eq(value))
            .filter(Column::ExpiresAt.gt(now))
            .one(db)
            .await
    }

    /// 校验有效性（不消耗记录）
    pub async fn peek_valid<C>(db: &C, identifier: &str, value: &str) -> Result<bool, DbErr>
    where
        C: ConnectionTrait,
    {
        Ok(Self::find_valid(db, identifier, value).await?.is_some())
    }

    /// 原子校验并消费（单条 DELETE）有效记录，防止重放攻击
    pub async fn consume<C>(db: &C, identifier: &str, value: &str) -> Result<bool, DbErr>
    where
        C: ConnectionTrait,
    {
        let res = Self::delete_many()
            .filter(Column::Identifier.eq(identifier))
            .filter(Column::Value.eq(value))
            .filter(Column::ExpiresAt.gt(Utc::now()))
            .exec(db)
            .await?;
        Ok(res.rows_affected > 0)
    }

    /// 根据 identifier 前缀与 value 查找有效记录（不消费，如 Magic Link 预检）
    pub async fn find_valid_by_prefix_and_value<C>(
        db: &C,
        prefix: &str,
        value: &str,
    ) -> Result<Option<Model>, DbErr>
    where
        C: ConnectionTrait,
    {
        Self::find()
            .filter(Column::Identifier.starts_with(prefix))
            .filter(Column::Value.eq(value))
            .filter(Column::ExpiresAt.gt(Utc::now()))
            .one(db)
            .await
    }

    /// 根据 identifier 前缀原子消费记录（如 Magic Link 验证）
    pub async fn consume_by_prefix_and_value<C>(
        db: &C,
        prefix: &str,
        value: &str,
    ) -> Result<bool, DbErr>
    where
        C: ConnectionTrait,
    {
        let res = Self::delete_many()
            .filter(Column::Identifier.starts_with(prefix))
            .filter(Column::Value.eq(value))
            .filter(Column::ExpiresAt.gt(Utc::now()))
            .exec(db)
            .await?;
        Ok(res.rows_affected > 0)
    }

    /// OTP 暴力破解防护：记录一次校验失败，达到上限即锁定
    pub async fn record_otp_failure<C>(db: &C, identifier: &str) -> Result<(), DbErr>
    where
        C: ConnectionTrait,
    {
        use crate::common::consts::auth::{OTP_LOCKOUT_SECS, OTP_MAX_ATTEMPTS};

        let fail_key = format!("otp-fail:{identifier}");
        let count = Self::find_valid_by_identifier(db, &fail_key)
            .await?
            .and_then(|m| m.value.parse::<i32>().ok())
            .unwrap_or(0)
            + 1;
        Self::create_record(db, &fail_key, &count.to_string(), OTP_LOCKOUT_SECS).await?;
        if count >= OTP_MAX_ATTEMPTS {
            tracing::warn!(identifier = %identifier, "OTP attempt limit reached, locked");
        }
        Ok(())
    }

    /// OTP 是否处于锁定状态（失败次数达上限且未过锁定窗口）
    pub async fn otp_is_locked<C>(db: &C, identifier: &str) -> Result<bool, DbErr>
    where
        C: ConnectionTrait,
    {
        use crate::common::consts::auth::OTP_MAX_ATTEMPTS;

        let fail_key = format!("otp-fail:{identifier}");
        Ok(Self::find_valid_by_identifier(db, &fail_key)
            .await?
            .and_then(|m| m.value.parse::<i32>().ok())
            .is_some_and(|c| c >= OTP_MAX_ATTEMPTS))
    }

    /// 清理所有过期失效的凭据
    pub async fn clean_expired<C>(db: &C) -> Result<u64, DbErr>
    where
        C: ConnectionTrait,
    {
        let now = Utc::now();
        let res = Self::delete_many()
            .filter(Column::ExpiresAt.lt(now))
            .exec(db)
            .await?;
        Ok(res.rows_affected)
    }
}
