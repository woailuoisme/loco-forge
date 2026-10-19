pub use super::_entities::organization::{ActiveModel, Entity, Model};
use sea_orm::entity::prelude::*;
pub type Organization = Entity;

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
    /// 更新组织基本信息
    pub async fn update_info(
        &self,
        db: &DatabaseConnection,
        name: Option<String>,
        slug: Option<String>,
        logo: Option<String>,
        metadata: Option<String>,
    ) -> ModelResult<Self> {
        let mut active: ActiveModel = self.clone().into();
        if let Some(n) = name {
            active.name = ActiveValue::Set(n);
        }
        if let Some(s) = slug {
            active.slug = ActiveValue::Set(s);
        }
        if let Some(l) = logo {
            active.logo = ActiveValue::Set(Some(l));
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
impl Entity {
    /// 创建新组织
    pub async fn create_organization<C>(
        db: &C,
        name: &str,
        slug: &str,
        logo: Option<String>,
        metadata: Option<String>,
    ) -> ModelResult<Model>
    where
        C: ConnectionTrait,
    {
        let now = Utc::now().into();
        let active = ActiveModel {
            id: ActiveValue::Set(Uuid::now_v7().to_string()),
            name: ActiveValue::Set(name.to_string()),
            slug: ActiveValue::Set(slug.to_string()),
            logo: ActiveValue::Set(logo),
            metadata: ActiveValue::Set(metadata),
            created_at: ActiveValue::Set(now),
            updated_at: ActiveValue::Set(now),
        };
        let created = Self::insert(active).exec_with_returning(db).await?;
        Ok(created)
    }

    /// 根据 ID 查找组织
    pub async fn find_by_id(db: &DatabaseConnection, id: &str) -> ModelResult<Option<Model>> {
        use super::_entities::organization::Column;
        let org = Self::find().filter(Column::Id.eq(id)).one(db).await?;
        Ok(org)
    }

    /// 根据 Slug 查找组织
    pub async fn find_by_slug(db: &DatabaseConnection, slug: &str) -> ModelResult<Option<Model>> {
        use super::_entities::organization::Column;
        let org = Self::find().filter(Column::Slug.eq(slug)).one(db).await?;
        Ok(org)
    }

    /// 查找指定用户加入的所有组织列表
    pub async fn find_user_organizations(
        db: &DatabaseConnection,
        user_id: &str,
    ) -> ModelResult<Vec<Model>> {
        use super::_entities::organization::Column;
        let orgs = Self::find()
            .inner_join(super::_entities::member::Entity)
            .filter(super::_entities::member::Column::UserId.eq(user_id))
            .order_by_desc(Column::CreatedAt)
            .all(db)
            .await?;
        Ok(orgs)
    }

    /// 删除指定组织
    pub async fn delete_organization(db: &DatabaseConnection, id: &str) -> ModelResult<bool> {
        use super::_entities::organization::Column;
        let res = Self::delete_many()
            .filter(Column::Id.eq(id))
            .exec(db)
            .await?;
        Ok(res.rows_affected > 0)
    }
}
