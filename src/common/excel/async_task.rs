use bytes::Bytes;
use loco_rs::prelude::*;

/// 在 Tokio 阻塞线程池中异步非阻塞执行 CPU 密集型的 Excel 导出任务 (返回零拷贝 `bytes::Bytes`)
pub async fn spawn_excel_export<F>(f: F) -> Result<Bytes, Error>
where
    F: FnOnce() -> Result<Bytes, Error> + Send + 'static,
{
    spawn_excel_blocking(f).await
}

/// 在 Tokio 阻塞线程池中异步非阻塞执行 CPU 密集型的 Excel 导入与解析任务
pub async fn spawn_excel_blocking<F, T>(f: F) -> Result<T, Error>
where
    F: FnOnce() -> Result<T, Error> + Send + 'static,
    T: Send + 'static,
{
    tokio::task::spawn_blocking(f).await.map_err(|e| {
        tracing::error!("Excel 异步 Task 执行失败: {:?}", e);
        Error::InternalServerError
    })?
}
