//! # 全文检索 Fluent 查询构造器 (ScoutBuilder & ScoutQuery & SearchResult)

use super::{config::MeilisearchConfig, engines::meilisearch::MeilisearchEngine, traits::Engine};
use loco_rs::prelude::*;
use sea_orm::{ColumnTrait, EntityTrait};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Meilisearch 检索请求参数包 (ScoutQuery)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScoutQuery {
    /// 全文搜索关键字
    pub q: String,
    /// 过滤条件表达式 (例如: `status = 'published' AND age > 18`)
    pub filter: Option<String>,
    /// 排序条件数组 (例如: `["created_at:desc"]`)
    pub sort: Vec<String>,
    /// 返回结果限制数量
    pub limit: Option<usize>,
    /// 分页偏移量
    pub offset: Option<usize>,
}

/// Meilisearch 搜索命中响应结果 (SearchResult)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchResult {
    /// 命中的文档 JSON 列表
    pub hits: Vec<Value>,
    /// 预计总命中数
    #[serde(default)]
    pub estimated_total_hits: usize,
    /// 处理耗时 (毫秒)
    #[serde(default)]
    pub processing_time_ms: usize,
    /// 限制数量
    #[serde(default)]
    pub limit: usize,
    /// 分页偏移
    #[serde(default)]
    pub offset: usize,
}

impl SearchResult {
    /// 从命中的文档列表中提取主键 ID 数组 (自动转换并提取 `id` 字段)
    pub fn extract_ids(&self) -> Vec<i64> {
        self.hits
            .iter()
            .filter_map(|hit| {
                let id_val = hit.get("id")?;
                id_val.as_i64().or_else(|| id_val.as_str()?.parse().ok())
            })
            .collect()
    }
}

/// 仿 Laravel `Scout` Fluent 检索构造器 (ScoutBuilder)
pub struct ScoutBuilder<'a> {
    ctx: &'a AppContext,
    index_name: String,
    query: ScoutQuery,
}

impl<'a> ScoutBuilder<'a> {
    /// 创建一个新的检索构造器
    pub fn new(ctx: &'a AppContext, index_name: impl Into<String>) -> Self {
        Self {
            ctx,
            index_name: index_name.into(),
            query: ScoutQuery::default(),
        }
    }

    /// 设置全文搜索关键词 (Query Text)
    pub fn query(mut self, q: impl Into<String>) -> Self {
        self.query.q = q.into();
        self
    }

    /// 设置检索过滤表达式 (Filter)
    pub fn filter(mut self, filter: impl Into<String>) -> Self {
        self.query.filter = Some(filter.into());
        self
    }

    /// 设置排序数组 (Sort)
    pub fn sort(mut self, sort: &[&str]) -> Self {
        self.query.sort = sort.iter().map(|s| s.to_string()).collect();
        self
    }

    /// 设置检索限制数量 (Limit)
    pub fn limit(mut self, limit: usize) -> Self {
        self.query.limit = Some(limit);
        self
    }

    /// 设置分页偏移量 (Offset)
    pub fn offset(mut self, offset: usize) -> Self {
        self.query.offset = Some(offset);
        self
    }

    /// 快捷设置第 N 页与每页条数 (1-indexed page)
    pub fn paginate(mut self, page: usize, per_page: usize) -> Self {
        let p = page.max(1);
        self.query.limit = Some(per_page);
        self.query.offset = Some((p - 1) * per_page);
        self
    }

    /// 执行检索并获取 Meilisearch 的原始响应 JSON 数据包
    pub async fn get_raw(&self) -> Result<SearchResult> {
        let config =
            MeilisearchConfig::from_ctx(self.ctx).ok_or_else(|| Error::string("Meilisearch config is not set"))?;

        let engine = MeilisearchEngine::new(config)?;
        engine.search(&self.index_name, &self.query).await
    }

    /// 执行检索并仅提取主键 ID 列表
    pub async fn get_ids(&self) -> Result<Vec<i64>> {
        let raw = self.get_raw().await?;
        Ok(raw.extract_ids())
    }

    /// 执行检索，并反查 SeaORM 数据库还原实体列表（保持 Meilisearch 相关度 Hit 排序）
    pub async fn get_models<E>(&self, db: &DatabaseConnection) -> Result<Vec<E::Model>>
    where
        E: EntityTrait,
        E::Model: Send + Sync + Serialize,
        E::Column: ColumnTrait,
    {
        let ids = self.get_ids().await?;
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        use sea_orm::{Iterable, PrimaryKeyToColumn, QueryFilter};
        let mut query = E::find();

        if let Some(primary_key) = E::PrimaryKey::iter().next() {
            let col = primary_key.into_column();
            query = query.filter(col.is_in(ids.clone()));
        }

        let models = query
            .all(db)
            .await
            .map_err(|e| Error::string(&format!("Database query failed: {e}")))?;

        // 建立 ID -> Model 映射表，保持 Meilisearch 命中得分与排序关系
        let mut model_map: std::collections::HashMap<i64, E::Model> = models
            .into_iter()
            .filter_map(|m| {
                let val = serde_json::to_value(&m).ok()?;
                let id = val.get("id")?.as_i64()?;
                Some((id, m))
            })
            .collect();

        let ordered_models = ids.into_iter().filter_map(|id| model_map.remove(&id)).collect();

        Ok(ordered_models)
    }
}
