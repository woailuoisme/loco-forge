//! # 离线文件/资源下载后台 Worker (DownloadWorker)
//!
//! 该 Worker 用于在后台异步处理耗时较长的文件下载、数据导出或远程资源拉取任务，
//! 避免阻塞主 HTTP 请求线程。集成 `tokio_util::sync::CancellationToken` 以支持长任务协同取消。

use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;

/// 离线下载后台 Worker 实例
pub struct DownloadWorker {
    pub ctx: AppContext,
}

/// 离线下载作业参数对象 (DownloadWorkerArgs)
#[derive(Deserialize, Debug, Serialize)]
pub struct DownloadWorkerArgs {
    /// 请求下载的用户唯一标识符 (User GUID)
    pub user_guid: String,
    /// 目标资源下载 URL
    pub url: Option<String>,
}

impl DownloadWorker {
    /// 执行支持取消信号的耗时下载任务
    pub async fn perform_with_cancellation(
        &self,
        args: &DownloadWorkerArgs,
        cancel_token: CancellationToken,
    ) -> Result<()> {
        tokio::select! {
            _ = cancel_token.cancelled() => {
                tracing::warn!(target: "worker", user_guid = %args.user_guid, "DownloadWorker task was cancelled via CancellationToken");
                Err(loco_rs::Error::string("Task cancelled by token"))
            }
            res = self.do_download(args) => {
                res
            }
        }
    }

    async fn do_download(&self, _args: &DownloadWorkerArgs) -> Result<()> {
        // 模拟耗时异步 IO 下载过程...
        Ok(())
    }
}

#[async_trait]
impl BackgroundWorker<DownloadWorkerArgs> for DownloadWorker {
    fn build(ctx: &AppContext) -> Self {
        Self { ctx: ctx.clone() }
    }
    async fn perform(&self, args: DownloadWorkerArgs) -> Result<()> {
        let cancel_token = CancellationToken::new();
        self.perform_with_cancellation(&args, cancel_token).await
    }
}
