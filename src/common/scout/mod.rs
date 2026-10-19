//! # 仿 Laravel Scout 全文搜索引擎模块
//!
//! 提供强类型 `Searchable` 模版接口、Meilisearch 驱动实现、支持 Fluent API 查询构造器 (`ScoutBuilder`)，
//! 以及基于 Loco Background Worker 的异步队列延迟索引更新。
//!
//! ## 示例 (Example)
//! ```rust,ignore
//! use locos::common::scout::{Searchable, ScoutBuilder};
//! use serde_json::json;
//!
//! pub struct Article {
//!     pub id: i64,
//!     pub title: String,
//! }
//!
//! impl Searchable for Article {
//!     fn searchable_as(&self) -> String {
//!         "articles".to_string()
//!     }
//!     fn get_searchable_key(&self) -> String {
//!         self.id.to_string()
//!     }
//!     fn to_searchable_array(&self) -> serde_json::Value {
//!         json!({ "id": self.id, "title": self.title })
//!     }
//! }
//!
//! // 1. 同步更新索引
//! article.searchable(&ctx).await?;
//!
//! // 2. Fluent 全文检索与 ORM 还原
//! let articles = ScoutBuilder::new(&ctx, "articles")
//!     .query("Rust")
//!     .filter("status = 'published'")
//!     .get_models::<ArticleEntity>(&ctx, &db)
//!     .await?;
//! ```

pub mod builder;
pub mod config;
pub mod engines;
pub mod traits;

pub use builder::{ScoutBuilder, ScoutQuery, SearchResult};
pub use config::MeilisearchConfig;
pub use engines::MeilisearchEngine;
pub use traits::{Engine, Searchable};

use loco_rs::prelude::*;

/// 为实现了 `Searchable` 的结构体或模型提供静态/实例级别的实用扩展方法
pub struct Scout;

impl Scout {
    /// 从 AppContext 解析 Meilisearch 配置并创建引擎（配置缺失时返回统一错误）
    fn engine_from_ctx(ctx: &AppContext) -> Result<MeilisearchEngine> {
        let config = MeilisearchConfig::from_ctx(ctx)
            .ok_or_else(|| Error::string("Meilisearch config is not set"))?;
        MeilisearchEngine::new(config)
    }

    /// 同步/异步更新单个模型记录至 Meilisearch 索引
    pub async fn searchable<T: Searchable + ?Sized>(ctx: &AppContext, model: &T) -> Result<()> {
        let engine = Self::engine_from_ctx(ctx)?;
        let docs = vec![model.to_searchable_value()?];
        engine.update(model.searchable_as(), &docs).await?;
        Ok(())
    }

    /// 批量更新模型记录数组至 Meilisearch 索引
    pub async fn searchable_batch<T: Searchable>(ctx: &AppContext, models: &[T]) -> Result<()> {
        if models.is_empty() {
            return Ok(());
        }

        let engine = Self::engine_from_ctx(ctx)?;
        let index_name = models[0].searchable_as();
        let mut docs = Vec::with_capacity(models.len());
        for m in models {
            docs.push(m.to_searchable_value()?);
        }

        engine.update(index_name, &docs).await?;
        Ok(())
    }

    /// 从 Meilisearch 索引中删除单个模型记录
    pub async fn unsearchable<T: Searchable + ?Sized>(ctx: &AppContext, model: &T) -> Result<()> {
        let engine = Self::engine_from_ctx(ctx)?;
        let key = model.get_searchable_key();
        engine.delete(model.searchable_as(), &[&key]).await?;
        Ok(())
    }

    /// 将索引更新入列推送至 Loco Background Worker 进行后台延迟分发
    pub async fn searchable_queued<T: Searchable + ?Sized>(
        ctx: &AppContext,
        model: &T,
    ) -> Result<()> {
        let job_args = crate::workers::scout::ScoutJobArgs {
            action: crate::workers::scout::ScoutAction::Update,
            index_name: model.searchable_as().to_string(),
            key: model.get_searchable_key(),
            document: Some(model.to_searchable_value()?),
        };
        crate::workers::scout::ScoutWorker::perform_later(ctx, job_args).await?;
        Ok(())
    }

    /// 将索引删除入列推送至 Loco Background Worker
    pub async fn unsearchable_queued<T: Searchable + ?Sized>(
        ctx: &AppContext,
        model: &T,
    ) -> Result<()> {
        let job_args = crate::workers::scout::ScoutJobArgs {
            action: crate::workers::scout::ScoutAction::Delete,
            index_name: model.searchable_as().to_string(),
            key: model.get_searchable_key(),
            document: None,
        };
        crate::workers::scout::ScoutWorker::perform_later(ctx, job_args).await?;
        Ok(())
    }
}
