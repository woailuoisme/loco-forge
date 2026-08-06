//! # 媒体与二维码处理模块 (Media & QR Code Module)
//!
//! 提供高性能、强类型、异步安全的图像处理、缩放裁剪、水印叠加与二维码生成功能。
//!
//! ## 快速使用示例 (Examples)
//!
//! ### 1. 生成带 Logo 的二维码
//! ```rust,ignore
//! use locos::common::media::{QrCodeBuilder, QrLogoConfig};
//!
//! let logo_bytes = std::fs::read("logo.png")?;
//! let logo_cfg = QrLogoConfig::new(logo_bytes).with_scale_ratio(0.2);
//!
//! let png_bytes = QrCodeBuilder::new("https://loco.rs")
//!     .size(300)
//!     .with_logo(logo_cfg)
//!     .to_png_bytes()?;
//! ```
//!
//! ### 2. 图像居中裁剪与水印叠加
//! ```rust,ignore
//! use locos::common::media::{ImageProcessor, ResizeMode, WatermarkPosition, OutputFormat};
//!
//! let watermark_bytes = std::fs::read("watermark.png")?;
//! let processed_bytes = ImageProcessor::from_bytes(&input_bytes)?
//!     .resize(800, 600, ResizeMode::FillCrop)
//!     .watermark(&watermark_bytes, WatermarkPosition::BottomRight, 0.7)?
//!     .format(OutputFormat::Jpeg { quality: 85 })
//!     .to_bytes()?;
//! ```

pub mod async_task;
pub mod error;
pub mod image;
pub mod qrcode;
pub mod response;

pub use async_task::spawn_media_task;
pub use error::{MediaError, MediaResult};
pub use image::{ImagePreset, ImageProcessor, OutputFormat, ResizeMode, WatermarkPosition};
pub use qrcode::{QrCodeBuilder, QrLogoConfig};
pub use response::MediaResponse;
