use thiserror::Error;

/// 媒体处理模块的错误类型 (Media Error)
#[derive(Debug, Error)]
pub enum MediaError {
    /// 图像编解码或处理错误
    #[error("Image processing error: {0}")]
    Image(#[from] image::ImageError),

    /// 二维码生成错误
    #[error("QR code generation error: {0}")]
    Qr(#[from] qrcode::types::QrError),

    /// I/O 文件读取与写入错误
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// 不合法的输入参数 (如尺寸为0、格式不支持等)
    #[error("Invalid input or parameters: {0}")]
    InvalidInput(String),

    /// 异步线程池任务执行失败
    #[error("Async join task failed: {0}")]
    AsyncTask(String),
}

pub type MediaResult<T> = Result<T, MediaError>;

impl From<MediaError> for loco_rs::Error {
    fn from(err: MediaError) -> Self {
        loco_rs::Error::string(&err.to_string())
    }
}
