use crate::common::media::error::{MediaError, MediaResult};

/// 在 Tokio 阻塞线程池 (`tokio::task::spawn_blocking`) 中异步非阻塞执行 CPU 密集型的媒体/图片处理闭包。
/// 能够有效规避图像缩放、二维码生成或水印合成等 CPU 计算任务阻塞 Tokio worker 线程的问题。
pub async fn spawn_media_task<F, T>(f: F) -> MediaResult<T>
where
    F: FnOnce() -> MediaResult<T> + Send + 'static,
    T: Send + 'static,
{
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| MediaError::AsyncTask(e.to_string()))?
}
