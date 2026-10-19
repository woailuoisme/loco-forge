use chrono::Utc;
use loco_rs::prelude::*;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseConnection,
    EntityTrait, IntoActiveModel, QueryFilter,
};
use uuid::Uuid;

pub use super::_entities::account::{ActiveModel, Column, Entity, Model};
pub type Account = Entity;

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
    /// 校验密码哈希是否匹配
    #[must_use]
    pub fn verify_password(&self, raw_password: &str) -> bool {
        let Some(hash) = &self.password else {
            return false;
        };
        loco_rs::hash::verify_password(raw_password, hash)
    }

    /// 更新密码凭据
    pub async fn update_password(
        &self,
        db: &DatabaseConnection,
        new_raw_password: &str,
    ) -> ModelResult<Self> {
        let hashed = loco_rs::hash::hash_password(new_raw_password)
            .map_err(|e| ModelError::Any(e.into()))?;
        let mut active = self.clone().into_active_model();
        active.password = Set(Some(hashed));
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
    /// 查找特定 Provider 下指定 account_id 的账号凭据
    pub async fn find_by_provider_and_account(
        db: &DatabaseConnection,
        provider_id: &str,
        account_id: &str,
    ) -> ModelResult<Option<Model>> {
        let account = Self::find()
            .filter(Column::ProviderId.eq(provider_id))
            .filter(Column::AccountId.eq(account_id))
            .one(db)
            .await?;
        Ok(account)
    }

    /// 查找用户的密码凭据（provider_id = "credential"）
    pub async fn find_credential_by_user(
        db: &DatabaseConnection,
        user_id: &str,
    ) -> ModelResult<Option<Model>> {
        let account = Self::find()
            .filter(Column::UserId.eq(user_id))
            .filter(Column::ProviderId.eq("credential"))
            .one(db)
            .await?;
        Ok(account)
    }

    /// 为用户创建密码凭据 (provider_id = "credential")
    pub async fn create_credential<C>(
        db: &C,
        user_id: &str,
        account_id: &str,
        raw_password: &str,
    ) -> ModelResult<Model>
    where
        C: ConnectionTrait,
    {
        let hashed =
            loco_rs::hash::hash_password(raw_password).map_err(|e| ModelError::Any(e.into()))?;
        let now = Utc::now().into();
        let active = ActiveModel {
            id: Set(Uuid::now_v7().to_string()),
            user_id: Set(user_id.to_string()),
            account_id: Set(account_id.to_string()),
            provider_id: Set("credential".to_string()),
            password: Set(Some(hashed)),
            access_token: Set(None),
            refresh_token: Set(None),
            id_token: Set(None),
            access_token_expires_at: Set(None),
            refresh_token_expires_at: Set(None),
            scope: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        };
        let account = active.insert(db).await?;
        Ok(account)
    }

    /// 为用户创建第三方 OAuth / OIDC 账号绑定记录
    pub async fn create_oauth_account(
        db: &DatabaseConnection,
        user_id: &str,
        provider_id: &str,
        account_id: &str,
        access_token: Option<&str>,
        id_token: Option<&str>,
    ) -> ModelResult<Model> {
        let now = Utc::now().into();
        let active = ActiveModel {
            id: Set(Uuid::now_v7().to_string()),
            user_id: Set(user_id.to_string()),
            account_id: Set(account_id.to_string()),
            provider_id: Set(provider_id.to_string()),
            password: Set(None),
            access_token: Set(access_token.map(str::to_string)),
            refresh_token: Set(None),
            id_token: Set(id_token.map(str::to_string)),
            access_token_expires_at: Set(None),
            refresh_token_expires_at: Set(None),
            scope: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        };
        let account = active.insert(db).await?;
        Ok(account)
    }

    /// 删除指定用户的所有关联账号凭据
    pub async fn delete_by_user(db: &impl ConnectionTrait, user_id: &str) -> ModelResult<u64> {
        let res = Self::delete_many()
            .filter(Column::UserId.eq(user_id))
            .exec(db)
            .await?;
        Ok(res.rows_affected)
    }
}
