//! # Scout 搜索引擎模块单元与集成测试

use loco_rs::testing::prelude::*;
use locos::app::App;
use locos::common::scout::{MeilisearchConfig, ScoutBuilder, SearchResult, Searchable};
use serde_json::json;

/// 模拟模型对象 (Article)
#[derive(serde::Serialize)]
struct Article {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub category: String,
}

impl Searchable for Article {
    fn searchable_as(&self) -> &str {
        "articles"
    }

    fn get_searchable_key(&self) -> String {
        self.id.to_string()
    }

    fn to_searchable_value(&self) -> loco_rs::Result<serde_json::Value> {
        Ok(json!({
            "id": self.id,
            "title": self.title,
            "content": self.content,
            "category": self.category,
        }))
    }
}

#[test]
fn test_searchable_trait_methods() {
    let article = Article {
        id: 10086,
        title: "Rust 高性能全栈开发".to_string(),
        content: "Loco 框架集成 Meilisearch 搜索引擎".to_string(),
        category: "tech".to_string(),
    };

    assert_eq!(article.searchable_as(), "articles");
    assert_eq!(article.get_searchable_key(), "10086");

    let doc = article.to_searchable_value().unwrap();
    assert_eq!(doc["id"], 10086);
    assert_eq!(doc["title"], "Rust 高性能全栈开发");
}

#[test]
fn test_search_result_extract_ids() {
    let res = SearchResult {
        hits: vec![
            json!({ "id": 101, "title": "Article 1" }),
            json!({ "id": "102", "title": "Article 2" }),
            json!({ "id": 103, "title": "Article 3" }),
        ],
        estimated_total_hits: 3,
        processing_time_ms: 2,
        limit: 20,
        offset: 0,
    };

    let ids = res.extract_ids();
    assert_eq!(ids, vec![101, 102, 103]);
}

#[test]
fn test_meilisearch_config_prefix() {
    let cfg = MeilisearchConfig {
        host: "http://localhost:7700".to_string(),
        api_key: Some("secret_key".to_string()),
        index_prefix: Some("dev_".to_string()),
        timeout_sec: 10,
    };

    assert_eq!(cfg.prefixed_index("users"), "dev_users");
    assert_eq!(cfg.prefixed_index("posts"), "dev_posts");
}

#[tokio::test]
#[serial_test::serial]
async fn test_scout_builder_query_chaining() {
    request::<App, _, _>(|_request, ctx| async move {
        let builder = ScoutBuilder::new(&ctx, "articles")
            .query("Rust")
            .filter("category = 'tech'")
            .sort(&["created_at:desc"])
            .paginate(1, 15);

        // 默认背景测试环境中无 Live Meilisearch 服务，预期 get_raw 返回连接错误
        let res = builder.get_raw().await;
        assert!(res.is_err());
    })
    .await;
}
