pub use super::_entities::sso_provider::{ActiveModel, Entity, Model};
use chrono::Utc;
use loco_rs::model::ModelResult;
use sea_orm::{
    ActiveModelBehavior, ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait,
    DatabaseConnection, DbErr, EntityTrait, QueryFilter,
};
use uuid::Uuid;

pub type SsoProvider = Entity;

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

// ① 单实例读写
impl Model {
    /// 删除单个 SSO Provider
    pub async fn delete_by_id(self, db: &DatabaseConnection) -> ModelResult<()> {
        let active: ActiveModel = self.into();
        active.delete(db).await?;
        Ok(())
    }
}

// ② 字段级钩子与 ActiveModel 行为
impl ActiveModel {}

// ③ 集合级查询与关联逻辑
impl Entity {
    /// 根据 provider_id (如 "google", "github") 查询配置
    pub async fn find_by_provider_id(
        db: &DatabaseConnection,
        provider_id: &str,
    ) -> ModelResult<Option<Model>> {
        use super::_entities::sso_provider::Column;
        let provider = Self::find()
            .filter(Column::ProviderId.eq(provider_id))
            .one(db)
            .await?;
        Ok(provider)
    }

    /// 根据域名查询 SSO Provider
    pub async fn find_by_domain(
        db: &DatabaseConnection,
        domain: &str,
    ) -> ModelResult<Option<Model>> {
        use super::_entities::sso_provider::Column;
        let provider = Self::find()
            .filter(Column::Domain.eq(domain))
            .one(db)
            .await?;
        Ok(provider)
    }

    /// 创建或更新 SSO Provider
    pub async fn upsert_provider(
        db: &DatabaseConnection,
        provider_id: &str,
        issuer: &str,
        domain: &str,
        oidc_config: Option<&str>,
        saml_config: Option<&str>,
    ) -> ModelResult<Model> {
        use super::_entities::sso_provider::Column;
        let existing = Self::find()
            .filter(Column::ProviderId.eq(provider_id))
            .one(db)
            .await?;

        if let Some(model) = existing {
            let mut active: ActiveModel = model.into();
            active.issuer = ActiveValue::Set(issuer.to_string());
            active.domain = ActiveValue::Set(domain.to_string());
            if let Some(cfg) = oidc_config {
                active.oidc_config = ActiveValue::Set(Some(cfg.to_string()));
            }
            if let Some(cfg) = saml_config {
                active.saml_config = ActiveValue::Set(Some(cfg.to_string()));
            }
            active.updated_at = ActiveValue::Set(Utc::now().into());
            let updated = active.update(db).await?;
            Ok(updated)
        } else {
            let active = ActiveModel {
                id: ActiveValue::Set(Uuid::now_v7().to_string()),
                provider_id: ActiveValue::Set(provider_id.to_string()),
                issuer: ActiveValue::Set(issuer.to_string()),
                domain: ActiveValue::Set(domain.to_string()),
                oidc_config: ActiveValue::Set(oidc_config.map(str::to_string)),
                saml_config: ActiveValue::Set(saml_config.map(str::to_string)),
                user_id: ActiveValue::Set(None),
                organization_id: ActiveValue::Set(None),
                created_at: ActiveValue::Set(Utc::now().into()),
                updated_at: ActiveValue::Set(Utc::now().into()),
            };
            let created = Self::insert(active).exec_with_returning(db).await?;
            Ok(created)
        }
    }
}
