pub use super::_entities::posts::{ActiveModel, Entity, Model};
use sea_orm::entity::prelude::*;
pub type Posts = Entity;

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

use loco_rs::prelude::*;
use sea_orm::{Condition, QueryOrder};

// implement your read-oriented logic here
impl Model {
    /// 根据主键 ID 查找文章
    ///
    /// # Errors
    ///
    /// 当数据库查询出错或未查找到对应记录时返回 `ModelError::EntityNotFound`
    pub async fn find_by_id(db: &DatabaseConnection, id: i64) -> ModelResult<Self> {
        let post = Entity::find_by_id(id).one(db).await?;
        post.ok_or_else(|| ModelError::EntityNotFound)
    }

    /// 创建并持久化一篇新文章
    ///
    /// # Errors
    ///
    /// 当数据库插入失败时返回 `ModelError`
    pub async fn create(
        db: &DatabaseConnection,
        title: Option<String>,
        content: Option<String>,
    ) -> ModelResult<Self> {
        let active = ActiveModel {
            title: ActiveValue::Set(title),
            content: ActiveValue::Set(content),
            ..Default::default()
        };
        active.insert(db).await.map_err(ModelError::from)
    }

    /// 更新当前文章的标题与内容
    ///
    /// # Errors
    ///
    /// 当数据库更新失败时返回 `ModelError`
    pub async fn update_post(
        self,
        db: &DatabaseConnection,
        title: Option<String>,
        content: Option<String>,
    ) -> ModelResult<Self> {
        let mut active: ActiveModel = self.into();
        active.title = ActiveValue::Set(title);
        active.content = ActiveValue::Set(content);
        active.update(db).await.map_err(ModelError::from)
    }

    /// 从数据库中删除当前文章
    ///
    /// # Errors
    ///
    /// 当数据库删除失败时返回 `ModelError`
    pub async fn delete(self, db: &DatabaseConnection) -> ModelResult<()> {
        let active: ActiveModel = self.into();
        active.delete(db).await?;
        Ok(())
    }
}

// implement your custom finders, selectors oriented logic here
impl Entity {
    /// 按创建时间倒序分页获取文章列表
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
            .order_by_desc(super::_entities::posts::Column::CreatedAt)
            .paginate_with_total(db, page, page_size)
            .await
    }

    /// 根据标题或内容关键字模糊搜索文章列表并支持分页
    ///
    /// # Errors
    ///
    /// 当数据库搜索查询失败时返回 `ModelError`
    pub async fn search(
        db: &DatabaseConnection,
        keyword: &str,
        page: u64,
        page_size: u64,
    ) -> ModelResult<(Vec<Model>, u64)> {
        use super::repository::QueryPaginationExt;

        let pattern = format!("%{keyword}%");
        Self::find()
            .filter(
                Condition::any()
                    .add(super::_entities::posts::Column::Title.like(&pattern))
                    .add(super::_entities::posts::Column::Content.like(&pattern)),
            )
            .order_by_desc(super::_entities::posts::Column::CreatedAt)
            .paginate_with_total(db, page, page_size)
            .await
    }

    /// 根据 ID 列表批量删除文章记录
    ///
    /// # Errors
    ///
    /// 当数据库删除操作失败时返回 `ModelError`
    pub async fn delete_by_ids(db: &DatabaseConnection, ids: &[i64]) -> ModelResult<u64> {
        if ids.is_empty() {
            return Ok(0);
        }

        let res = Self::delete_many()
            .filter(super::_entities::posts::Column::Id.is_in(ids.to_vec()))
            .exec(db)
            .await?;

        tracing::info!(
            count = res.rows_affected,
            "Successfully deleted posts by IDs"
        );

        Ok(res.rows_affected)
    }
}
