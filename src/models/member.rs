pub use super::_entities::member::{ActiveModel, Entity, Model};
use sea_orm::entity::prelude::*;
pub type Member = Entity;

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
    /// 更新组织成员角色
    pub async fn update_role(&self, db: &DatabaseConnection, new_role: &str) -> ModelResult<Self> {
        let mut active: ActiveModel = self.clone().into();
        active.role = ActiveValue::Set(new_role.to_string());
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
    /// 创建组织成员
    pub async fn create_member<C>(
        db: &C,
        organization_id: &str,
        user_id: &str,
        role: &str,
    ) -> ModelResult<Model>
    where
        C: sea_orm::ConnectionTrait,
    {
        let now = Utc::now().into();
        let active = ActiveModel {
            id: ActiveValue::Set(Uuid::now_v7().to_string()),
            organization_id: ActiveValue::Set(organization_id.to_string()),
            user_id: ActiveValue::Set(user_id.to_string()),
            role: ActiveValue::Set(role.to_string()),
            created_at: ActiveValue::Set(now),
            updated_at: ActiveValue::Set(now),
        };
        let created = Self::insert(active).exec_with_returning(db).await?;
        Ok(created)
    }

    /// 根据 ID 查找成员
    pub async fn find_by_id(db: &DatabaseConnection, id: &str) -> ModelResult<Option<Model>> {
        use super::_entities::member::Column;
        let member = Self::find().filter(Column::Id.eq(id)).one(db).await?;
        Ok(member)
    }

    /// 根据组织 ID 与用户 ID 查找成员
    pub async fn find_by_org_and_user(
        db: &impl sea_orm::ConnectionTrait,
        organization_id: &str,
        user_id: &str,
    ) -> ModelResult<Option<Model>> {
        use super::_entities::member::Column;
        let member = Self::find()
            .filter(Column::OrganizationId.eq(organization_id))
            .filter(Column::UserId.eq(user_id))
            .one(db)
            .await?;
        Ok(member)
    }

    /// 支持按用户 ID、成员 ID 或邮箱查找组织成员
    pub async fn find_by_org_and_user_or_email(
        db: &DatabaseConnection,
        organization_id: &str,
        user_id_or_email: &str,
    ) -> ModelResult<Option<Model>> {
        if user_id_or_email.contains('@') {
            if let Some(u) = super::user::Entity::find_by_email(db, user_id_or_email).await? {
                return Self::find_by_org_and_user(db, organization_id, &u.id).await;
            }
            return Ok(None);
        }
        if let Some(m) = Self::find_by_id(db, user_id_or_email).await?
            && m.organization_id == organization_id
        {
            return Ok(Some(m));
        }
        Self::find_by_org_and_user(db, organization_id, user_id_or_email).await
    }

    /// 获取组织下所有成员及关联用户
    pub async fn find_org_members_with_user<C>(
        db: &C,
        organization_id: &str,
    ) -> ModelResult<Vec<(Model, Option<super::_entities::user::Model>)>>
    where
        C: sea_orm::ConnectionTrait,
    {
        use super::_entities::member::Column;
        let list = Self::find()
            .filter(Column::OrganizationId.eq(organization_id))
            .find_also_related(super::_entities::user::Entity)
            .order_by_asc(Column::CreatedAt)
            .all(db)
            .await?;
        Ok(list)
    }

    /// 删除指定成员
    pub async fn delete_member<C>(db: &C, id: &str) -> ModelResult<bool>
    where
        C: sea_orm::ConnectionTrait,
    {
        use super::_entities::member::Column;
        let res = Self::delete_many()
            .filter(Column::Id.eq(id))
            .exec(db)
            .await?;
        Ok(res.rows_affected > 0)
    }

    /// 根据组织 ID 与用户 ID 移除成员
    pub async fn delete_by_org_and_user(
        db: &DatabaseConnection,
        organization_id: &str,
        user_id: &str,
    ) -> ModelResult<bool> {
        use super::_entities::member::Column;
        let res = Self::delete_many()
            .filter(Column::OrganizationId.eq(organization_id))
            .filter(Column::UserId.eq(user_id))
            .exec(db)
            .await?;
        Ok(res.rows_affected > 0)
    }
}
