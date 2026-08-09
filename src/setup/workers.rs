//! # 后台异步 Worker 队列注册模块
//!
//! 负责注册后台处理队列表 (Redis / Apalis 异步 Job)。

use crate::workers::{
    downloader::DownloadWorker, notification::NotificationWorker, scout::ScoutWorker,
};
use loco_rs::{
    Result,
    app::AppContext,
    bgworker::{BackgroundWorker, Queue},
};

/// 连接与注册后台 Worker 队列
///
/// 当开启 `cargo loco start --worker` 时，处理注册在队列中的异步 Job。
pub async fn connect_workers(ctx: &AppContext, queue: &Queue) -> Result<()> {
    // 步骤 1: 构建并注册文件下载异步 Worker 实例
    queue.register(DownloadWorker::build(ctx)).await?;

    // 步骤 2: 构建并注册通知异步 Worker 实例
    queue.register(NotificationWorker::build(ctx)).await?;

    // 步骤 3: 构建并注册 Scout 搜索引擎异步 Worker 实例
    queue.register(ScoutWorker::build(ctx)).await?;

    Ok(())
}
