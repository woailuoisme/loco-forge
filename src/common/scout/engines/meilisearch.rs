//! # Meilisearch 全文搜索引擎驱动 (Meilisearch Engine)

use crate::common::scout::{
    builder::{ScoutQuery, SearchResult},
    config::MeilisearchConfig,
    traits::Engine,
};
use async_trait::async_trait;
use loco_rs::prelude::*;
use serde_json::{Value, json};
use std::sync::LazyLock;
use std::time::Duration;

// 共享的全局 HTTP 连接池客户端 (复用 Keep-Alive，大幅降低网络开销)
static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("Failed to initialize Meilisearch HTTP client")
});

/// Meilisearch 全文搜索引擎驱动结构体 (MeilisearchEngine)
pub struct MeilisearchEngine {
    config: MeilisearchConfig,
}

impl MeilisearchEngine {
    /// 创建 MeilisearchEngine 实例
    pub fn new(config: MeilisearchConfig) -> Self {
        Self { config }
    }

    /// 封装附带 API Key 的 HTTP Request Builder
    fn request_builder(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        let url = format!(
            "{}/{}",
            self.config.host.trim_end_matches('/'),
            path.trim_start_matches('/')
        );
        let mut req = HTTP_CLIENT.request(method, &url);
        if let Some(key) = &self.config.api_key {
            req = req.header("Authorization", format!("Bearer {key}"));
            req = req.header("X-Meili-Api-Key", key);
        }
        req
    }
}

#[async_trait]
impl Engine for MeilisearchEngine {
    async fn update(&self, index_name: &str, docs: &[Value]) -> Result<Value> {
        let prefixed_index = self.config.prefixed_index(index_name);
        let path = format!("indexes/{prefixed_index}/documents");

        let res = self
            .request_builder(reqwest::Method::POST, &path)
            .json(docs)
            .send()
            .await
            .map_err(|e| Error::string(&format!("Meilisearch update request failed: {e}")))?;

        if !res.status().is_success() {
            let status = res.status();
            let err_text = res.text().await.unwrap_or_default();
            tracing::error!("Meilisearch 文档更新失败 HTTP {}: {}", status, err_text);
            return Err(Error::string(&format!(
                "Meilisearch update error HTTP {status}: {err_text}"
            )));
        }

        let json_val = res
            .json::<Value>()
            .await
            .map_err(|e| Error::string(&format!("Parse Meilisearch update response failed: {e}")))?;

        tracing::info!(
            "Meilisearch 文档成功索引至 [{}] ({} 条记录)",
            prefixed_index,
            docs.len()
        );
        Ok(json_val)
    }

    async fn delete(&self, index_name: &str, ids: &[&str]) -> Result<Value> {
        let prefixed_index = self.config.prefixed_index(index_name);
        let path = format!("indexes/{prefixed_index}/documents/delete-batch");

        let res = self
            .request_builder(reqwest::Method::POST, &path)
            .json(ids)
            .send()
            .await
            .map_err(|e| Error::string(&format!("Meilisearch delete request failed: {e}")))?;

        if !res.status().is_success() {
            let status = res.status();
            let err_text = res.text().await.unwrap_or_default();
            tracing::error!("Meilisearch 文档删除失败 HTTP {}: {}", status, err_text);
            return Err(Error::string(&format!(
                "Meilisearch delete error HTTP {status}: {err_text}"
            )));
        }

        let json_val = res
            .json::<Value>()
            .await
            .map_err(|e| Error::string(&format!("Parse Meilisearch delete response failed: {e}")))?;

        tracing::info!(
            "Meilisearch 文档成功从 [{}] 批量删除 ({} 条记录)",
            prefixed_index,
            ids.len()
        );
        Ok(json_val)
    }

    async fn flush(&self, index_name: &str) -> Result<Value> {
        let prefixed_index = self.config.prefixed_index(index_name);
        let path = format!("indexes/{prefixed_index}/documents");

        let res = self
            .request_builder(reqwest::Method::DELETE, &path)
            .send()
            .await
            .map_err(|e| Error::string(&format!("Meilisearch flush request failed: {e}")))?;

        let json_val = res
            .json::<Value>()
            .await
            .map_err(|e| Error::string(&format!("Parse Meilisearch flush response failed: {e}")))?;

        tracing::info!("Meilisearch 索引 [{}] 已清空 (Flush)", prefixed_index);
        Ok(json_val)
    }

    async fn search(&self, index_name: &str, query: &ScoutQuery) -> Result<SearchResult> {
        let prefixed_index = self.config.prefixed_index(index_name);
        let path = format!("indexes/{prefixed_index}/search");

        let mut body = json!({
            "q": query.q,
        });
        if let Some(f) = &query.filter {
            body["filter"] = json!(f);
        }
        if !query.sort.is_empty() {
            body["sort"] = json!(query.sort);
        }
        if let Some(limit) = query.limit {
            body["limit"] = json!(limit);
        }
        if let Some(offset) = query.offset {
            body["offset"] = json!(offset);
        }

        let res = self
            .request_builder(reqwest::Method::POST, &path)
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::string(&format!("Meilisearch search request failed: {e}")))?;

        if !res.status().is_success() {
            let status = res.status();
            let err_text = res.text().await.unwrap_or_default();
            tracing::error!("Meilisearch 检索失败 HTTP {}: {}", status, err_text);
            return Err(Error::string(&format!(
                "Meilisearch search error HTTP {status}: {err_text}"
            )));
        }

        let result = res
            .json::<SearchResult>()
            .await
            .map_err(|e| Error::string(&format!("Parse Meilisearch search response failed: {e}")))?;

        Ok(result)
    }
}
