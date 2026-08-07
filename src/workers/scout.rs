//! # 全文检索索引更新后台 Worker (ScoutWorker)
//!
//! 该 Worker 负责从后台队列中异步接收全文检索索引更新任务，
//! 支持增量更新文档、删除文档和刷新索引，将数据同步到 Meilisearch 等搜索引擎中。

use crate::common::scout::{config::MeilisearchConfig, engines::meilisearch::MeilisearchEngine, traits::Engine};
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 异步索引操作类型 (ScoutAction)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScoutAction {
    /// 新增或更新文档索引
    Update,
    /// 删除指定主键文档
    Delete,
    /// 清空指定索引
    Flush,
}

/// 全文检索后台 Worker 实例 (ScoutWorker)
pub struct ScoutWorker {
    pub ctx: AppContext,
}

/// 队列作业参数对象 (ScoutJobArgs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoutJobArgs {
    /// 索引操作指令
    pub action: ScoutAction,
    /// 目标索引名
    pub index_name: String,
    /// 文档主键 ID
    pub key: String,
    /// 文档 JSON Payload
    pub document: Option<Value>,
}

#[async_trait]
impl BackgroundWorker<ScoutJobArgs> for ScoutWorker {
    fn build(ctx: &AppContext) -> Self {
        Self { ctx: ctx.clone() }
    }

    async fn perform(&self, args: ScoutJobArgs) -> Result<()> {
        let config = match MeilisearchConfig::from_ctx(&self.ctx) {
            Some(cfg) => cfg,
            None => {
                tracing::warn!("Scout Worker 跳过: 未配置 Meilisearch (缺少 MEILISEARCH_HOST 或 YAML 配置)");
                return Ok(());
            }
        };

        let engine = MeilisearchEngine::new(config)?;

        match args.action {
            ScoutAction::Update => {
                if let Some(doc) = args.document {
                    engine.update(&args.index_name, &[doc]).await?;
                }
            }
            ScoutAction::Delete => {
                if !args.key.is_empty() {
                    engine.delete(&args.index_name, &[&args.key]).await?;
                }
            }
            ScoutAction::Flush => {
                engine.flush(&args.index_name).await?;
            }
        }

        Ok(())
    }
}
