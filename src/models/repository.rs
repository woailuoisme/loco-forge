//! # Sea-ORM 2.0 泛型仓储扩展 Trait (Generic Model Repository)
//!
//! 为所有 Sea-ORM `Select<E>` 查询与数据实体提供统一的分页、批量操作扩展能力。

use async_trait::async_trait;
use loco_rs::model::ModelResult;
use sea_orm::{DatabaseConnection, EntityTrait, FromQueryResult, PaginatorTrait, Select};

/// Sea-ORM 2.0 查询分页扩展 Trait (Query Pagination Extension)
#[async_trait]
pub trait QueryPaginationExt<M> {
    /// 执行分页并返回 `(Vec<M>, total_items: u64)`
    ///
    /// # Errors
    ///
    /// 当数据库分页查询失败时返回 `ModelResult`
    async fn paginate_with_total(
        self,
        db: &DatabaseConnection,
        page: u64,
        page_size: u64,
    ) -> ModelResult<(Vec<M>, u64)>;
}

#[async_trait]
impl<E, M> QueryPaginationExt<M> for Select<E>
where
    E: EntityTrait<Model = M>,
    M: FromQueryResult + Send + Sync + 'static,
{
    async fn paginate_with_total(
        self,
        db: &DatabaseConnection,
        page: u64,
        page_size: u64,
    ) -> ModelResult<(Vec<M>, u64)> {
        let paginator = self.paginate(db, page_size);
        let total = paginator.num_items().await?;
        let items = paginator.fetch_page(page.saturating_sub(1)).await?;
        Ok((items, total))
    }
}
