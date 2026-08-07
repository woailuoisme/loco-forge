//! # Scout 搜索引擎通用 Trait 定义 (Searchable & Engine)

use super::builder::{ScoutQuery, SearchResult};
use async_trait::async_trait;
use loco_rs::prelude::*;
use serde_json::Value;

/// 可全文检索的 ORM 模型或结构体接口 (Searchable Trait)
pub trait Searchable: Send + Sync {
    /// 获取当前模型对应的 Meilisearch 索引名称 (如 "users", "posts")
    fn searchable_as(&self) -> &str;

    /// 获取模型的主键唯一标识字符串 (如 "101", "uuid-v4-string")
    fn get_searchable_key(&self) -> String;

    /// 将模型转换为待索引的 JSON 文档对象
    fn to_searchable_value(&self) -> Result<Value>;
}

/// 全文搜索引擎驱动接口 (Engine Trait)
#[async_trait]
pub trait Engine: Send + Sync {
    /// 将一组 JSON 文档更新或新增至指定索引
    async fn update(&self, index_name: &str, docs: &[Value]) -> Result<Value>;

    /// 从指定索引中批量删除指定主键 ID 的文档
    async fn delete(&self, index_name: &str, ids: &[&str]) -> Result<Value>;

    /// 清空指定索引下的所有文档 (Flush)
    async fn flush(&self, index_name: &str) -> Result<Value>;

    /// 在指定索引中执行全文检索并返回原始命中的文档与元数据
    async fn search(&self, index_name: &str, query: &ScoutQuery) -> Result<SearchResult>;
}
