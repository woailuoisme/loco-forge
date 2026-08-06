//! # 离线文件/资源下载后台 Worker (DownloadWorker)
//!
//! 该 Worker 用于在后台异步处理耗时较长的文件下载、数据导出或远程资源拉取任务，
//! 避免阻塞主 HTTP 请求线程。

use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};

/// 离线下载后台 Worker 实例
pub struct DownloadWorker {
    pub ctx: AppContext,
}

/// 离线下载作业参数对象 (DownloadWorkerArgs)
#[derive(Deserialize, Debug, Serialize)]
pub struct DownloadWorkerArgs {
    /// 请求下载的用户唯一标识符 (User GUID)
    pub user_guid: String,
}

#[async_trait]
impl BackgroundWorker<DownloadWorkerArgs> for DownloadWorker {
    fn build(ctx: &AppContext) -> Self {
        Self { ctx: ctx.clone() }
    }
    async fn perform(&self, _args: DownloadWorkerArgs) -> Result<()> {
        // TODO: Some actual work goes here...

        Ok(())
    }
}
