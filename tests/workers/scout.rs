//! ScoutWorker 单元测试：验证后台全文检索索引任务的 perform 行为。
//!
//! ⚠️ 该 Worker 在 `perform` 中会通过 `MeilisearchConfig::from_ctx` 解析配置，
//! 默认（未配置 Meilisearch）会回退到 `localhost:7700`，随后与真实搜索引擎建立连接。
//! 由于 CI / 本地默认环境无 Meilisearch 服务，以下用例标记为 `#[ignore]`，
//! 仅在显式提供可用 Meilisearch 实例（设置 `MEILISEARCH_HOST` / YAML 配置）时运行：
//! `cargo nextest run --test integration -- --ignored`

use loco_rs::bgworker::BackgroundWorker;
use loco_rs::testing::prelude::*;
use locos::{
    app::App,
    workers::scout::{ScoutAction, ScoutJobArgs, ScoutWorker},
};
use serde_json::json;
use serial_test::serial;

/// Update 操作应在已连接 Meilisearch 时成功提交索引任务
#[tokio::test]
#[serial]
#[ignore = "requires a running Meilisearch instance"]
async fn test_scout_worker_update() {
    let boot = boot_test::<App>().await.unwrap();
    let worker = ScoutWorker::build(&boot.app_context);

    let args = ScoutJobArgs {
        action: ScoutAction::Update,
        index_name: "articles".to_string(),
        key: "1".to_string(),
        document: Some(json!({"id": 1, "title": "Loco"})),
    };

    let res = worker.perform(args).await;
    assert!(
        res.is_ok(),
        "ScoutWorker update should succeed with Meilisearch"
    );
}
