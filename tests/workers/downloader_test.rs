//! DownloadWorker 单元测试：验证后台离线下载任务的 perform 行为。

use loco_rs::bgworker::BackgroundWorker;
use loco_rs::testing::prelude::*;
use locos::{
    app::App,
    workers::downloader::{DownloadWorker, DownloadWorkerArgs},
};
use serial_test::serial;

/// 无取消信号时，DownloadWorker::perform 应成功返回 Ok(())
#[tokio::test]
#[serial]
async fn test_download_worker_perform_ok() {
    let boot = boot_test::<App>().await.unwrap();
    let worker = DownloadWorker::build(&boot.app_context);

    let args = DownloadWorkerArgs {
        user_guid: "test-user-guid".to_string(),
        url: Some("https://example.com/file.zip".to_string()),
    };

    let res = worker.perform(args).await;
    assert!(res.is_ok(), "DownloadWorker perform should succeed");
}

/// url 为 None 时，下载任务仍应成功返回 Ok(())
#[tokio::test]
#[serial]
async fn test_download_worker_without_url_ok() {
    let boot = boot_test::<App>().await.unwrap();
    let worker = DownloadWorker::build(&boot.app_context);

    let args = DownloadWorkerArgs {
        user_guid: "test-user-guid".to_string(),
        url: None,
    };

    let res = worker.perform(args).await;
    assert!(res.is_ok(), "DownloadWorker perform should succeed without url");
}
