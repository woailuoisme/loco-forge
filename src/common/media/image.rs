use image::{DynamicImage, GenericImageView, ImageFormat, codecs::jpeg::JpegEncoder, imageops};
use std::io::Cursor;

use crate::common::consts;
use crate::common::media::error::{MediaError, MediaResult};

/// 图像缩放与裁剪模式 (Resize Mode)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeMode {
    /// 保持宽高比缩放，适应目标边界框 (尺寸不会超出指定的 width/height)
    Fit,
    /// 保持宽高比放大/缩小，填满目标边界框，并居中裁剪多余部分 (正好等于指定的 width/height)
    FillCrop,
    /// 强制无损/变形拉伸到指定的目标尺寸 (忽略原始宽高比)
    Exact,
    /// 极速生成缩略图 (优化 Downsampling 算法性能)
    Thumbnail,
}

/// 常见的图像变体预设 (Laravel Media Library Conversions / Presets)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImagePreset {
    /// 用户头像：200x200 填充居中裁剪
    Avatar,
    /// 缩略图：150x150 比例适应
    Thumbnail,
    /// 中等高清图：600x600 比例适应
    Medium,
    /// 高清大图：1200x1200 比例适应
    Large,
    /// 社交媒体 OG 封面展示卡片：1200x630 (1.91:1 最佳黄金比例)
    Cover,
}

/// 水印九宫格位置定义 (Watermark Position)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatermarkPosition {
    Center,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    TopCenter,
    BottomCenter,
    LeftCenter,
    RightCenter,
}

/// 导出目标图像格式定义 (Output Format)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    #[default]
    Png,
    Jpeg {
        quality: u8,
    },
    Webp {
        quality: u8,
    },
    Bmp,
    Ico,
}

/// 图像处理器构建器 (ImageProcessor)
#[derive(Debug, Clone)]
pub struct ImageProcessor {
    image: DynamicImage,
    format: OutputFormat,
}

impl ImageProcessor {
    /// 从内存二进制字节数据中加载并解析图片 (自动校正 EXIF 旋转角，包含 8000px 维度防炸安全保护)
    pub fn from_bytes(bytes: &[u8]) -> MediaResult<Self> {
        if bytes.is_empty() {
            return Err(MediaError::InvalidInput("Image input byte array is empty".to_string()));
        }
        let image = image::load_from_memory(bytes)?;
        let (w, h) = image.dimensions();
        if w > consts::media::MAX_DIMENSION || h > consts::media::MAX_DIMENSION {
            return Err(MediaError::InvalidInput(format!(
                "Image dimensions ({w}x{h}) exceed maximum allowed safety limit of {}x{}px",
                consts::media::MAX_DIMENSION,
                consts::media::MAX_DIMENSION
            )));
        }
        Ok(Self {
            image,
            format: OutputFormat::Png,
        })
    }

    /// 从已有的 `image::DynamicImage` 实例构造
    pub fn from_image(image: DynamicImage) -> Self {
        Self {
            image,
            format: OutputFormat::Png,
        }
    }

    /// 获取当前图像的像素尺寸 (width, height)
    pub fn dimensions(&self) -> (u32, u32) {
        self.image.dimensions()
    }

    /// 设置图像导出目标格式 (如 PNG, JPEG, WebP 含质量参数)
    pub fn format(mut self, format: OutputFormat) -> Self {
        self.format = format;
        self
    }

    /// 应用 Laravel Media Library 风格的变体预设 (Presets)
    pub fn apply_preset(self, preset: ImagePreset) -> Self {
        match preset {
            ImagePreset::Avatar => self.resize(200, 200, ResizeMode::FillCrop),
            ImagePreset::Thumbnail => self.resize(150, 150, ResizeMode::Fit),
            ImagePreset::Medium => self.resize(600, 600, ResizeMode::Fit),
            ImagePreset::Large => self.resize(1200, 1200, ResizeMode::Fit),
            ImagePreset::Cover => self.resize(1200, 630, ResizeMode::FillCrop),
        }
    }

    /// 执行图像尺寸调整/裁剪操作
    pub fn resize(mut self, target_width: u32, target_height: u32, mode: ResizeMode) -> Self {
        if target_width == 0 || target_height == 0 {
            return self;
        }

        self.image = match mode {
            ResizeMode::Fit => self
                .image
                .resize(target_width, target_height, imageops::FilterType::Lanczos3),
            ResizeMode::Exact => self
                .image
                .resize_exact(target_width, target_height, imageops::FilterType::Lanczos3),
            ResizeMode::Thumbnail => self.image.thumbnail(target_width, target_height),
            ResizeMode::FillCrop => {
                let (orig_w, orig_h) = self.image.dimensions();
                if orig_w == 0 || orig_h == 0 {
                    return self;
                }

                let ratio_w = target_width as f32 / orig_w as f32;
                let ratio_h = target_height as f32 / orig_h as f32;
                let scale_ratio = ratio_w.max(ratio_h);

                let scaled_w = (orig_w as f32 * scale_ratio).round() as u32;
                let scaled_h = (orig_h as f32 * scale_ratio).round() as u32;

                let scaled_img = self
                    .image
                    .resize_exact(scaled_w, scaled_h, imageops::FilterType::Lanczos3);

                let crop_x = (scaled_w.saturating_sub(target_width)) / 2;
                let crop_y = (scaled_h.saturating_sub(target_height)) / 2;

                scaled_img.crop_imm(crop_x, crop_y, target_width, target_height)
            }
        };

        self
    }

    // ==================== 链式滤镜与变换操作 (Filters & Transforms) ====================

    /// 灰度滤镜 (Grayscale)
    pub fn grayscale(mut self) -> Self {
        self.image = self.image.grayscale();
        self
    }

    /// 高斯模糊滤镜 (Gaussian Blur)
    pub fn blur(mut self, sigma: f32) -> Self {
        self.image = self.image.blur(sigma.max(0.0));
        self
    }

    /// 亮度调节 (Brighten)
    pub fn brighten(mut self, value: i32) -> Self {
        self.image = self.image.brighten(value);
        self
    }

    /// 对比度调节 (Contrast)
    pub fn contrast(mut self, value: f32) -> Self {
        self.image = self.image.adjust_contrast(value);
        self
    }

    /// 顺时针旋转 90 度
    pub fn rotate90(mut self) -> Self {
        self.image = self.image.rotate90();
        self
    }

    /// 顺时针旋转 180 度
    pub fn rotate180(mut self) -> Self {
        self.image = self.image.rotate180();
        self
    }

    /// 顺时针旋转 270 度
    pub fn rotate270(mut self) -> Self {
        self.image = self.image.rotate270();
        self
    }

    /// 水平镜像翻转 (Flip Horizontal)
    pub fn fliph(mut self) -> Self {
        self.image = self.image.fliph();
        self
    }

    /// 垂直镜像翻转 (Flip Vertical)
    pub fn flipv(mut self) -> Self {
        self.image = self.image.flipv();
        self
    }

    // ==================== 水印与导出操作 (Watermark & Export) ====================

    /// 叠加图片水印 (支持九宫格位置控制与透明度 alpha 调配)
    pub fn watermark(mut self, watermark_bytes: &[u8], position: WatermarkPosition, opacity: f32) -> MediaResult<Self> {
        if watermark_bytes.is_empty() {
            return Ok(self);
        }

        let watermark_dynamic = image::load_from_memory(watermark_bytes)?;
        let (base_w, base_h) = self.image.dimensions();
        let (wm_w, wm_h) = watermark_dynamic.dimensions();

        if wm_w == 0 || wm_h == 0 || base_w == 0 || base_h == 0 {
            return Ok(self);
        }

        let max_wm_w = (base_w as f32 * consts::media::WATERMARK_MAX_RATIO) as u32;
        let max_wm_h = (base_h as f32 * consts::media::WATERMARK_MAX_RATIO) as u32;

        let final_wm_dynamic = if wm_w > max_wm_w || wm_h > max_wm_h {
            watermark_dynamic.resize(max_wm_w.max(1), max_wm_h.max(1), imageops::FilterType::Lanczos3)
        } else {
            watermark_dynamic
        };

        let (cur_wm_w, cur_wm_h) = final_wm_dynamic.dimensions();

        let mut wm_rgba = final_wm_dynamic.to_rgba8();
        let alpha = opacity.clamp(0.0, 1.0);
        if alpha < 1.0 {
            for pixel in wm_rgba.pixels_mut() {
                pixel[3] = ((pixel[3] as f32) * alpha) as u8;
            }
        }

        let padding = consts::media::WATERMARK_PADDING;
        let center_x = (base_w.saturating_sub(cur_wm_w)) / 2;
        let center_y = (base_h.saturating_sub(cur_wm_h)) / 2;
        let right_x = base_w.saturating_sub(cur_wm_w).saturating_sub(padding);
        let bottom_y = base_h.saturating_sub(cur_wm_h).saturating_sub(padding);

        let (x, y) = match position {
            WatermarkPosition::Center => (center_x, center_y),
            WatermarkPosition::TopLeft => (padding, padding),
            WatermarkPosition::TopRight => (right_x, padding),
            WatermarkPosition::BottomLeft => (padding, bottom_y),
            WatermarkPosition::BottomRight => (right_x, bottom_y),
            WatermarkPosition::TopCenter => (center_x, padding),
            WatermarkPosition::BottomCenter => (center_x, bottom_y),
            WatermarkPosition::LeftCenter => (padding, center_y),
            WatermarkPosition::RightCenter => (right_x, center_y),
        };

        let mut base_rgba = self.image.to_rgba8();
        imageops::overlay(&mut base_rgba, &wm_rgba, x as i64, y as i64);
        self.image = DynamicImage::ImageRgba8(base_rgba);

        Ok(self)
    }

    /// 快捷导出为现代化 Web 推荐的高效 WebP 字节流
    pub fn to_webp_optimal(self, quality: u8) -> MediaResult<Vec<u8>> {
        self.format(OutputFormat::Webp { quality }).to_bytes()
    }

    /// 转换并导出为字节 Byte 数组
    pub fn to_bytes(&self) -> MediaResult<Vec<u8>> {
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);

        match self.format {
            OutputFormat::Png => {
                self.image.write_to(&mut cursor, ImageFormat::Png)?;
            }
            OutputFormat::Jpeg { quality } => {
                let rgb_image = self.image.to_rgb8();
                let mut encoder = JpegEncoder::new_with_quality(&mut cursor, quality.clamp(1, 100));
                encoder.encode(
                    rgb_image.as_raw(),
                    rgb_image.width(),
                    rgb_image.height(),
                    image::ExtendedColorType::Rgb8,
                )?;
            }
            OutputFormat::Webp { quality } => {
                let encoder = image::codecs::webp::WebPEncoder::new_lossless(&mut cursor);
                if quality >= consts::media::WEBP_LOSSLESS_QUALITY {
                    self.image.write_with_encoder(encoder)?;
                } else {
                    self.image.write_to(&mut cursor, ImageFormat::WebP)?;
                }
            }
            OutputFormat::Bmp => {
                self.image.write_to(&mut cursor, ImageFormat::Bmp)?;
            }
            OutputFormat::Ico => {
                self.image.write_to(&mut cursor, ImageFormat::Ico)?;
            }
        }

        Ok(buffer)
    }

    /// 获取底层 `DynamicImage` 内部对象
    pub fn to_dynamic_image(self) -> DynamicImage {
        self.image
    }
}
