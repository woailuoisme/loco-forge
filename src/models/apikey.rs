pub use super::_entities::apikey::{ActiveModel, Entity, Model};
use sea_orm::entity::prelude::*;
pub type Apikey = Entity;

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(self, _db: &C, insert: bool) -> std::result::Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if !insert && self.updated_at.is_unchanged() {
            let mut this = self;
            this.updated_at = sea_orm::ActiveValue::Set(chrono::Utc::now().into());
            Ok(this)
        } else {
            Ok(self)
        }
    }
}

use chrono::Utc;
use loco_rs::model::ModelResult;
use sea_orm::{ActiveValue, ColumnTrait, QueryFilter, QueryOrder};
use uuid::Uuid;

// ─────────────────────────────────────────────────────────────────────────────
// ① 单实例领域逻辑 (Model)
// ─────────────────────────────────────────────────────────────────────────────
impl Model {
    /// 验证当前 API Key 是否可用 (未禁用、未过期、未耗尽)
    #[must_use]
    pub fn is_valid(&self) -> bool {
        if !self.enabled {
            return false;
        }
        if let Some(exp) = self.expires_at
            && exp < Utc::now()
        {
            return false;
        }
        if let Some(rem) = self.remaining
            && rem <= 0
        {
            return false;
        }
        true
    }

    /// 消费一次请求配额并更新访问指标
    pub async fn consume_request(&self, db: &DatabaseConnection) -> ModelResult<Self> {
        let mut active: ActiveModel = self.clone().into();
        active.request_count = ActiveValue::Set(self.request_count + 1);
        active.last_request_at = ActiveValue::Set(Some(Utc::now().into()));
        if let Some(rem) = self.remaining {
            active.remaining = ActiveValue::Set(Some(rem.saturating_sub(1)));
        }
        active.updated_at = ActiveValue::Set(Utc::now().into());
        let updated = active.update(db).await?;
        Ok(updated)
    }

    /// 更新 API Key 配置
    #[allow(clippy::too_many_arguments)]
    pub async fn update_metadata(
        &self,
        db: &DatabaseConnection,
        name: Option<String>,
        enabled: Option<bool>,
        remaining: Option<i32>,
        refill_interval: Option<i32>,
        refill_amount: Option<i32>,
        permissions: Option<String>,
        metadata: Option<String>,
    ) -> ModelResult<Self> {
        let mut active: ActiveModel = self.clone().into();
        if let Some(n) = name {
            active.name = ActiveValue::Set(Some(n));
        }
        if let Some(e) = enabled {
            active.enabled = ActiveValue::Set(e);
        }
        if let Some(r) = remaining {
            active.remaining = ActiveValue::Set(Some(r));
        }
        if let Some(ri) = refill_interval {
            active.refill_interval = ActiveValue::Set(Some(ri));
        }
        if let Some(ra) = refill_amount {
            active.refill_amount = ActiveValue::Set(Some(ra));
        }
        if let Some(p) = permissions {
            active.permissions = ActiveValue::Set(Some(p));
        }
        if let Some(m) = metadata {
            active.metadata = ActiveValue::Set(Some(m));
        }
        active.updated_at = ActiveValue::Set(Utc::now().into());
        let updated = active.update(db).await?;
        Ok(updated)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ② ActiveModel 行为与钩子
// ─────────────────────────────────────────────────────────────────────────────
impl ActiveModel {}

// ─────────────────────────────────────────────────────────────────────────────
// ③ 集合级查询与操作 (Entity)
// ─────────────────────────────────────────────────────────────────────────────

/// API Key 只存哈希（SHA-256 hex，key 本身高熵无需加盐），明文仅在创建响应回传一次
fn hash_key(raw: &str) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(raw.as_bytes());
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

impl Entity {
    /// 创建新的 API Key
    #[allow(clippy::too_many_arguments)]
    pub async fn create_key(
        db: &DatabaseConnection,
        user_id: &str,
        name: Option<String>,
        prefix: Option<String>,
        expires_in_secs: Option<i64>,
        remaining: Option<i32>,
        refill_interval: Option<i32>,
        refill_amount: Option<i32>,
        rate_limit_enabled: Option<bool>,
        rate_limit_time_window: Option<i32>,
        rate_limit_max: Option<i32>,
        permissions: Option<String>,
        metadata: Option<String>,
    ) -> ModelResult<(Model, String)> {
        let now = Utc::now();
        let expires_at = expires_in_secs.map(|s| now + chrono::Duration::seconds(s));
        let pfx = prefix.unwrap_or_else(|| "pk_".to_string());
        let random_part = Uuid::now_v7().simple().to_string();
        let raw_key = format!("{pfx}{random_part}");
        let start = raw_key.chars().take(8).collect::<String>();

        let active = ActiveModel {
            id: ActiveValue::Set(Uuid::now_v7().to_string()),
            config_id: ActiveValue::Set("default".to_string()),
            name: ActiveValue::Set(name),
            start: ActiveValue::Set(Some(start)),
            prefix: ActiveValue::Set(Some(pfx)),
            key: ActiveValue::Set(hash_key(&raw_key)),
            user_id: ActiveValue::Set(user_id.to_string()),
            reference_id: ActiveValue::Set(None),
            refill_interval: ActiveValue::Set(refill_interval),
            refill_amount: ActiveValue::Set(refill_amount),
            last_refill_at: ActiveValue::Set(None),
            enabled: ActiveValue::Set(true),
            rate_limit_enabled: ActiveValue::Set(rate_limit_enabled.unwrap_or(false)),
            rate_limit_time_window: ActiveValue::Set(rate_limit_time_window),
            rate_limit_max: ActiveValue::Set(rate_limit_max),
            request_count: ActiveValue::Set(0),
            remaining: ActiveValue::Set(remaining),
            last_request_at: ActiveValue::Set(None),
            expires_at: ActiveValue::Set(expires_at.map(Into::into)),
            permissions: ActiveValue::Set(permissions),
            metadata: ActiveValue::Set(metadata),
            created_at: ActiveValue::Set(now.into()),
            updated_at: ActiveValue::Set(now.into()),
        };

        let created = Self::insert(active).exec_with_returning(db).await?;
        Ok((created, raw_key))
    }

    /// 根据 ID 查找 API Key
    pub async fn find_by_id(db: &DatabaseConnection, id: &str) -> ModelResult<Option<Model>> {
        use super::_entities::apikey::Column;
        let key = Self::find().filter(Column::Id.eq(id)).one(db).await?;
        Ok(key)
    }

    /// 根据完整 Key 查找（按哈希等值查询，单条索引查询零 N+1）
    pub async fn find_by_key(db: &DatabaseConnection, key: &str) -> ModelResult<Option<Model>> {
        use super::_entities::apikey::Column;
        let record = Self::find()
            .filter(Column::Key.eq(hash_key(key)))
            .one(db)
            .await?;
        Ok(record)
    }

    /// 根据用户 ID 查找其所有 API Key
    pub async fn find_by_user(db: &DatabaseConnection, user_id: &str) -> ModelResult<Vec<Model>> {
        use super::_entities::apikey::Column;
        let list = Self::find()
            .filter(Column::UserId.eq(user_id))
            .order_by_desc(Column::CreatedAt)
            .all(db)
            .await?;
        Ok(list)
    }

    /// 删除指定 API Key
    pub async fn delete_by_id(db: &DatabaseConnection, id: &str) -> ModelResult<bool> {
        use super::_entities::apikey::Column;
        let res = Self::delete_many()
            .filter(Column::Id.eq(id))
            .exec(db)
            .await?;
        Ok(res.rows_affected > 0)
    }

    /// 根据 ID 与用户 ID 删除 API Key (防越权)
    pub async fn delete_by_id_and_user(
        db: &DatabaseConnection,
        id: &str,
        user_id: &str,
    ) -> ModelResult<bool> {
        use super::_entities::apikey::Column;
        let res = Self::delete_many()
            .filter(Column::Id.eq(id))
            .filter(Column::UserId.eq(user_id))
            .exec(db)
            .await?;
        Ok(res.rows_affected > 0)
    }
}
