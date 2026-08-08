use loco_rs::prelude::*;

/// 在 Tokio 阻塞线程池中异步非阻塞执行 CPU 密集型的 Excel 导出任务
pub async fn spawn_excel_export<F>(f: F) -> Result<Vec<u8>, Error>
where
    F: FnOnce() -> Result<Vec<u8>, Error> + Send + 'static,
{
    spawn_excel_import(f).await
}

/// 在 Tokio 阻塞线程池中异步非阻塞执行 CPU 密集型的 Excel 导入与解析任务
pub async fn spawn_excel_import<F, T>(f: F) -> Result<T, Error>
where
    F: FnOnce() -> Result<T, Error> + Send + 'static,
    T: Send + 'static,
{
    tokio::task::spawn_blocking(f).await.map_err(|e| {
        tracing::error!("Excel 异步 Task 执行失败: {:?}", e);
        Error::InternalServerError
    })?
}

