pub use super::_entities::invitation::{ActiveModel, Entity, Model};
use sea_orm::entity::prelude::*;
pub type Invitation = Entity;

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
    /// 检查邀请是否已过期
    #[must_use]
    pub fn is_expired(&self) -> bool {
        self.expires_at < Utc::now()
    }

    /// 更新邀请状态
    pub async fn set_status(&self, db: &DatabaseConnection, status: &str) -> ModelResult<Self> {
        let mut active: ActiveModel = self.clone().into();
        active.status = ActiveValue::Set(status.to_string());
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
    /// 创建邀请
    pub async fn create_invitation(
        db: &DatabaseConnection,
        organization_id: &str,
        email: &str,
        role: &str,
        inviter_id: &str,
        expires_in_secs: i64,
    ) -> ModelResult<Model> {
        let now = Utc::now();
        let expires_at = now + chrono::Duration::seconds(expires_in_secs);
        let active = ActiveModel {
            id: ActiveValue::Set(Uuid::now_v7().to_string()),
            organization_id: ActiveValue::Set(organization_id.to_string()),
            email: ActiveValue::Set(email.trim().to_lowercase()),
            role: ActiveValue::Set(role.to_string()),
            status: ActiveValue::Set("pending".to_string()),
            expires_at: ActiveValue::Set(expires_at.into()),
            inviter_id: ActiveValue::Set(inviter_id.to_string()),
            created_at: ActiveValue::Set(now.into()),
            updated_at: ActiveValue::Set(now.into()),
        };
        let created = Self::insert(active).exec_with_returning(db).await?;
        Ok(created)
    }

    /// 根据 ID 查找邀请
    pub async fn find_by_id(db: &DatabaseConnection, id: &str) -> ModelResult<Option<Model>> {
        use super::_entities::invitation::Column;
        let invitation = Self::find().filter(Column::Id.eq(id)).one(db).await?;
        Ok(invitation)
    }

    /// 查询组织内待处理的邀请
    pub async fn find_pending_by_org(
        db: &DatabaseConnection,
        organization_id: &str,
    ) -> ModelResult<Vec<Model>> {
        use super::_entities::invitation::Column;
        let now = Utc::now();
        let list = Self::find()
            .filter(Column::OrganizationId.eq(organization_id))
            .filter(Column::Status.eq("pending"))
            .filter(Column::ExpiresAt.gt(now))
            .order_by_desc(Column::CreatedAt)
            .all(db)
            .await?;
        Ok(list)
    }

    /// 删除指定邀请
    pub async fn delete_by_id(db: &DatabaseConnection, id: &str) -> ModelResult<bool> {
        use super::_entities::invitation::Column;
        let res = Self::delete_many()
            .filter(Column::Id.eq(id))
            .exec(db)
            .await?;
        Ok(res.rows_affected > 0)
    }
}
