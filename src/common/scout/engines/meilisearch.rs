//! # Meilisearch 全文搜索引擎驱动 (Meilisearch Engine)

use crate::common::scout::{
    builder::{ScoutQuery, SearchResult},
    config::MeilisearchConfig,
    traits::Engine,
};
use async_trait::async_trait;
use loco_rs::prelude::*;
use meilisearch_sdk::client::Client;
use serde_json::Value;

use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};

static CLIENT_CACHE: LazyLock<RwLock<HashMap<String, Client>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// Meilisearch 全文搜索引擎驱动结构体 (MeilisearchEngine)
pub struct MeilisearchEngine {
    client: Client,
    config: MeilisearchConfig,
}

impl MeilisearchEngine {
    /// 创建 MeilisearchEngine 实例，自动复用全局 HTTP 客户端连接池
    pub fn new(config: MeilisearchConfig) -> Result<Self> {
        let cache_key = format!("{}:{:?}", config.host, config.api_key);

        if let Ok(read_guard) = CLIENT_CACHE.read()
            && let Some(cached_client) = read_guard.get(&cache_key)
        {
            return Ok(Self {
                client: cached_client.clone(),
                config,
            });
        }

        let client = Client::new(&config.host, config.api_key.as_deref())
            .map_err(|e| Error::string(&format!("Meilisearch client init failed: {e}")))?;

        if let Ok(mut write_guard) = CLIENT_CACHE.write() {
            write_guard.insert(cache_key, client.clone());
        }

        Ok(Self { client, config })
    }

    /// 获取底层官方 SDK Client 句柄
    pub fn client(&self) -> &Client {
        &self.client
    }
}

/// 构造 Meilisearch 异步任务响应 JSON（task_uid + index_uid）
fn task_response(task_uid: u32, index_uid: Option<&str>) -> Value {
    serde_json::json!({
        "task_uid": task_uid,
        "index_uid": index_uid,
    })
}

#[async_trait]
impl Engine for MeilisearchEngine {
    async fn update(&self, index_name: &str, docs: &[Value]) -> Result<Value> {
        let prefixed_index = self.config.prefixed_index(index_name);
        let index = self.client.index(&prefixed_index);

        let task_info = index
            .add_documents(docs, None)
            .await
            .map_err(|e| Error::string(&format!("Meilisearch update failed: {e}")))?;

        tracing::info!(
            task_uid = task_info.task_uid,
            index = %prefixed_index,
            doc_count = docs.len(),
            "Meilisearch 文档已提交索引任务"
        );

        Ok(task_response(
            task_info.task_uid,
            task_info.index_uid.as_deref(),
        ))
    }

    async fn delete(&self, index_name: &str, ids: &[&str]) -> Result<Value> {
        let prefixed_index = self.config.prefixed_index(index_name);
        let index = self.client.index(&prefixed_index);

        let task_info = index
            .delete_documents(ids)
            .await
            .map_err(|e| Error::string(&format!("Meilisearch delete failed: {e}")))?;

        tracing::info!(
            task_uid = task_info.task_uid,
            index = %prefixed_index,
            id_count = ids.len(),
            "Meilisearch 文档批量删除请求已提交"
        );

        Ok(task_response(
            task_info.task_uid,
            task_info.index_uid.as_deref(),
        ))
    }

    async fn flush(&self, index_name: &str) -> Result<Value> {
        let prefixed_index = self.config.prefixed_index(index_name);
        let index = self.client.index(&prefixed_index);

        let task_info = index
            .delete_all_documents()
            .await
            .map_err(|e| Error::string(&format!("Meilisearch flush failed: {e}")))?;

        tracing::info!(
            task_uid = task_info.task_uid,
            index = %prefixed_index,
            "Meilisearch 索引清空请求已提交"
        );

        Ok(task_response(
            task_info.task_uid,
            task_info.index_uid.as_deref(),
        ))
    }

    async fn search(&self, index_name: &str, query: &ScoutQuery) -> Result<SearchResult> {
        let prefixed_index = self.config.prefixed_index(index_name);
        let index = self.client.index(&prefixed_index);

        let sort_refs: Vec<&str> = query.sort.iter().map(String::as_str).collect();

        let mut search_builder = index.search();
        search_builder.with_query(&query.q);

        if let Some(filter) = &query.filter {
            search_builder.with_filter(filter);
        }

        if !sort_refs.is_empty() {
            search_builder.with_sort(&sort_refs);
        }

        if let Some(limit) = query.limit {
            search_builder.with_limit(limit);
        }

        if let Some(offset) = query.offset {
            search_builder.with_offset(offset);
        }

        let search_results = search_builder
            .execute::<Value>()
            .await
            .map_err(|e| Error::string(&format!("Meilisearch search failed: {e}")))?;

        let hits: Vec<Value> = search_results.hits.into_iter().map(|h| h.result).collect();
        let estimated_total_hits = search_results.estimated_total_hits.unwrap_or(hits.len());

        Ok(SearchResult {
            hits,
            estimated_total_hits,
            processing_time_ms: search_results.processing_time_ms,
            limit: query.limit.unwrap_or(20),
            offset: query.offset.unwrap_or(0),
        })
    }
}
