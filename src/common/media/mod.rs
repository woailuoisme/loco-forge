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

/// 媒体处理统一门面 (Media Facade)
///
/// 提供图像处理、缩放裁剪与二维码生成的快捷入口。
pub struct Media;

impl Media {
    /// 创建图像处理器构建器
    pub fn image(bytes: &[u8]) -> MediaResult<ImageProcessor> {
        ImageProcessor::from_bytes(bytes)
    }

    /// 创建二维码构建器
    pub fn qrcode(content: impl Into<String>) -> QrCodeBuilder {
        QrCodeBuilder::new(content)
    }
}

#[cfg(test)]
mod tests {
    use ::image::{ImageFormat, Rgba, RgbaImage};
    use axum::http::header;
    use rstest::{fixture, rstest};
    use std::io::Cursor;

    use super::*;

    /// 辅助函数：生成一个单色测试 PNG 内存字节数据
    fn create_test_image_bytes(width: u32, height: u32, color: [u8; 4]) -> Vec<u8> {
        let img = RgbaImage::from_pixel(width, height, Rgba(color));
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        img.write_to(&mut cursor, ImageFormat::Png).unwrap();
        buffer
    }

    /// rstest Fixture: 自动生成通用单色测试图片字节数据
    #[fixture]
    fn sample_image_bytes() -> Vec<u8> {
        create_test_image_bytes(200, 100, [255, 0, 0, 255])
    }

    #[test]
    fn test_qr_code_generation_png_and_data_url() {
        let builder = QrCodeBuilder::new("https://loco.rs").size(300).margin(2);

        let png_bytes = builder.to_png_bytes().expect("Should generate PNG bytes");
        assert!(!png_bytes.is_empty());
        assert_eq!(&png_bytes[0..4], &[0x89, 0x50, 0x4E, 0x47]);

        let data_url = builder
            .to_data_url()
            .expect("Should generate Data URL string");
        assert!(data_url.starts_with("data:image/png;base64,"));

        let dynamic_img = builder
            .to_dynamic_image()
            .expect("Should generate DynamicImage");
        assert!(dynamic_img.width() >= 200);
    }

    #[test]
    fn test_qr_code_svg_export() {
        let builder = QrCodeBuilder::new("Hello Loco QR Code")
            .size(200)
            .fg_color([255, 0, 0, 255])
            .bg_color([255, 255, 255, 255]);

        let svg_str = builder.to_svg_string().expect("Should generate SVG string");
        assert!(svg_str.contains("<svg"));
        assert!(svg_str.contains("#FF0000"));
    }

    #[test]
    fn test_qr_code_with_logo_embedding() {
        let logo_png = create_test_image_bytes(20, 20, [0, 0, 255, 255]);
        let logo_cfg = QrLogoConfig::new(logo_png)
            .with_scale_ratio(0.25)
            .with_border_padding(3);

        let builder = QrCodeBuilder::new("QR with Logo Center")
            .size(320)
            .with_logo(logo_cfg);

        let png_bytes = builder
            .to_png_bytes()
            .expect("Should generate QR code with Logo");
        assert!(!png_bytes.is_empty());

        let img = ::image::load_from_memory(&png_bytes).unwrap();
        assert_eq!(img.width(), 320);
    }

    #[rstest]
    #[case(ResizeMode::Fit, (80, 40))]
    #[case(ResizeMode::Exact, (80, 80))]
    #[case(ResizeMode::FillCrop, (80, 80))]
    fn test_image_processor_resizing_modes(
        sample_image_bytes: Vec<u8>,
        #[case] mode: ResizeMode,
        #[case] expected_dims: (u32, u32),
    ) {
        let proc = ImageProcessor::from_bytes(&sample_image_bytes)
            .unwrap()
            .resize(80, 80, mode);
        assert_eq!(proc.dimensions(), expected_dims);
    }

    #[test]
    fn test_image_processor_watermark_and_format() {
        let base_png = create_test_image_bytes(100, 100, [255, 255, 255, 255]);
        let wm_png = create_test_image_bytes(20, 20, [0, 255, 0, 255]);

        let processed_bytes = ImageProcessor::from_bytes(&base_png)
            .unwrap()
            .watermark(&wm_png, WatermarkPosition::BottomRight, 0.8)
            .unwrap()
            .format(OutputFormat::Jpeg { quality: 85 })
            .to_bytes()
            .expect("Should export JPEG with watermark");

        assert!(!processed_bytes.is_empty());
        // JPEG 头部魔数 0xFF 0xD8
        assert_eq!(&processed_bytes[0..2], &[0xFF, 0xD8]);
    }

    #[tokio::test]
    async fn test_async_spawn_media_task() {
        let res = spawn_media_task(|| {
            QrCodeBuilder::new("Async QR Generation Test")
                .size(150)
                .to_png_bytes()
        })
        .await
        .expect("Spawn media task should complete successfully");

        assert!(!res.is_empty());
    }

    #[test]
    fn test_media_response_headers() {
        use axum::response::IntoResponse;

        let png_data = vec![1, 2, 3, 4];
        let media_res = MediaResponse::png(png_data);
        assert_eq!(media_res.content_type, "image/png");

        let response = media_res.into_response();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let headers = response.headers();
        assert_eq!(headers.get(header::CONTENT_TYPE).unwrap(), "image/png");
        assert_eq!(
            headers.get(header::CACHE_CONTROL).unwrap(),
            "public, max-age=86400"
        );

        let svg_response = MediaResponse::svg("<svg></svg>").into_response();
        assert_eq!(
            svg_response.headers().get(header::CONTENT_TYPE).unwrap(),
            "image/svg+xml"
        );

        let webp_response = MediaResponse::webp(vec![1, 2, 3]).into_response();
        assert_eq!(
            webp_response.headers().get(header::CONTENT_TYPE).unwrap(),
            "image/webp"
        );
    }

    #[test]
    fn test_image_presets_and_filters() {
        let input_bytes = create_test_image_bytes(400, 300, [100, 150, 200, 255]);

        // 1. 测试 Presets 预设 (Avatar: 200x200, Cover: 1200x630)
        let avatar_proc = ImageProcessor::from_bytes(&input_bytes)
            .unwrap()
            .apply_preset(ImagePreset::Avatar);
        assert_eq!(avatar_proc.dimensions(), (200, 200));

        let cover_proc = ImageProcessor::from_bytes(&input_bytes)
            .unwrap()
            .apply_preset(ImagePreset::Cover);
        assert_eq!(cover_proc.dimensions(), (1200, 630));

        // 2. 测试 链式滤镜 (grayscale, blur, brighten, rotate90)
        let filtered_proc = ImageProcessor::from_bytes(&input_bytes)
            .unwrap()
            .grayscale()
            .blur(1.0)
            .brighten(10)
            .contrast(1.1)
            .rotate90();

        // 原始 400x300 旋转 90 度后变成 300x400
        assert_eq!(filtered_proc.dimensions(), (300, 400));

        // 3. 测试 WebP 导出
        let webp_bytes = filtered_proc
            .to_webp_optimal(80)
            .expect("Should export WebP optimal bytes");
        assert!(!webp_bytes.is_empty());
        // WebP 魔数 "RIFF" ... "WEBP"
        assert_eq!(&webp_bytes[0..4], b"RIFF");
        assert_eq!(&webp_bytes[8..12], b"WEBP");
    }

    #[test]
    fn test_media_facade_image_and_qrcode() {
        let input_bytes = create_test_image_bytes(200, 200, [50, 100, 150, 255]);
        let img_bytes = Media::image(&input_bytes)
            .expect("Media::image should initialize")
            .resize(100, 100, ResizeMode::FillCrop)
            .to_bytes()
            .expect("Should export processed image bytes");
        assert!(!img_bytes.is_empty());

        let qr_png = Media::qrcode("https://loco.rs")
            .size(256)
            .to_png_bytes()
            .expect("Media::qrcode should generate png");
        assert!(!qr_png.is_empty());
    }
}
