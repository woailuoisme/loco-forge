use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use bytes::Bytes;
use image::{DynamicImage, GenericImageView, ImageFormat, Rgba, RgbaImage, imageops};
use qrcode::{EcLevel, QrCode, render::svg};
use std::io::Cursor;

use crate::common::consts;
use crate::common::media::error::{MediaError, MediaResult};

/// 将 RGBA 颜色转换为 CSS 十六进制字符串（如 `#FF8800`）
fn rgba_to_hex(rgba: Rgba<u8>) -> String {
    format!("#{:02X}{:02X}{:02X}", rgba[0], rgba[1], rgba[2])
}

/// 二维码 Logo 配置结构体
#[derive(Debug, Clone)]
pub struct QrLogoConfig {
    /// Logo 图片内存字节数据 (采用 `bytes::Bytes` 实现零拷贝传递)
    pub logo_bytes: Bytes,
    /// Logo 占二维码总宽度的比例 (0.1 ~ 0.35, 默认 0.20 即 20%)
    pub scale_ratio: f32,
    /// Logo 外围白色对比遮罩边框大小 (像素)
    pub border_padding: u32,
}

impl QrLogoConfig {
    /// 构造新的 QrLogoConfig (零拷贝接收 `impl Into<Bytes>`)
    pub fn new(logo_bytes: impl Into<Bytes>) -> Self {
        Self {
            logo_bytes: logo_bytes.into(),
            scale_ratio: consts::qrcode::LOGO_DEFAULT_SCALE,
            border_padding: consts::qrcode::DEFAULT_BORDER_PADDING,
        }
    }

    /// 设置 Logo 缩放比例
    #[must_use]
    pub fn with_scale_ratio(mut self, ratio: f32) -> Self {
        self.scale_ratio = ratio.clamp(
            consts::qrcode::LOGO_MIN_SCALE,
            consts::qrcode::LOGO_MAX_SCALE,
        );
        self
    }

    /// 设置 Logo 对比边框大小
    #[must_use]
    pub fn with_border_padding(mut self, padding: u32) -> Self {
        self.border_padding = padding;
        self
    }
}

/// 二维码生成构建器 (QrCodeBuilder)
#[derive(Debug, Clone)]
pub struct QrCodeBuilder {
    content: String,
    size: u32,
    margin: u32,
    ec_level: EcLevel,
    fg_color: Rgba<u8>,
    bg_color: Rgba<u8>,
    logo: Option<QrLogoConfig>,
}

impl QrCodeBuilder {
    /// 创建新的二维码构建器
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            size: consts::qrcode::DEFAULT_SIZE,
            margin: consts::qrcode::DEFAULT_MARGIN,
            ec_level: EcLevel::M,
            fg_color: Rgba([0, 0, 0, 255]),
            bg_color: Rgba([255, 255, 255, 255]),
            logo: None,
        }
    }

    /// 设置图像导出像素目标尺寸 (默认 256px)
    #[must_use]
    pub fn size(mut self, size: u32) -> Self {
        self.size = size.max(consts::qrcode::MIN_SIZE);
        self
    }

    /// 设置二维码白边/留白边距格数 (默认 4)
    #[must_use]
    pub fn margin(mut self, margin: u32) -> Self {
        self.margin = margin;
        self
    }

    /// 设置纠错等级 (EcLevel::L, M, Q, H)
    #[must_use]
    pub fn ec_level(mut self, level: EcLevel) -> Self {
        self.ec_level = level;
        self
    }

    /// 设置前景色 (RGBA)
    #[must_use]
    pub fn fg_color(mut self, color: [u8; 4]) -> Self {
        self.fg_color = Rgba(color);
        self
    }

    /// 设置背景色 (RGBA)
    #[must_use]
    pub fn bg_color(mut self, color: [u8; 4]) -> Self {
        self.bg_color = Rgba(color);
        self
    }

    /// 嵌入中心 Logo (自动提升纠错等级至 H - 30% 容错，并叠加背景防护衬垫)
    #[must_use]
    pub fn with_logo(mut self, logo: QrLogoConfig) -> Self {
        self.logo = Some(logo);
        // 有 Logo 覆盖时自动提升容错率到最高级别 (EcLevel::H)，确保二维码可被顺畅扫描
        self.ec_level = EcLevel::H;
        self
    }

    /// 生成 `image::DynamicImage` 内存位图对象
    pub fn to_dynamic_image(&self) -> MediaResult<DynamicImage> {
        if self.content.is_empty() {
            return Err(MediaError::InvalidInput(
                "QR code content cannot be empty".to_string(),
            ));
        }

        // 1. 编码生成原始 QrCode 矩阵
        let code = QrCode::with_error_correction_level(self.content.as_bytes(), self.ec_level)?;
        let code_width = code.width() as u32;
        let total_modules = code_width + 2 * self.margin;

        // 2. 计算每个模块的像素宽度
        let module_px = (self.size / total_modules).max(1);
        let actual_size = total_modules * module_px;

        // 3. 构建 RGBA 图像缓冲区
        let mut img = RgbaImage::from_pixel(actual_size, actual_size, self.bg_color);
        let colors = code.to_colors();

        for y in 0..code_width {
            for x in 0..code_width {
                if colors[(y * code_width + x) as usize] == qrcode::Color::Dark {
                    let px_x = (x + self.margin) * module_px;
                    let px_y = (y + self.margin) * module_px;
                    for dy in 0..module_px {
                        for dx in 0..module_px {
                            img.put_pixel(px_x + dx, px_y + dy, self.fg_color);
                        }
                    }
                }
            }
        }

        // 4. 如果配置了 Logo，原位合成中心 Logo
        if let Some(logo_cfg) = &self.logo {
            let logo_img = image::load_from_memory(&logo_cfg.logo_bytes)?;
            let logo_target_size = (actual_size as f32 * logo_cfg.scale_ratio) as u32;

            if logo_target_size > 0 {
                let resized_logo = logo_img.resize(
                    logo_target_size,
                    logo_target_size,
                    imageops::FilterType::Lanczos3,
                );
                let (lw, lh) = resized_logo.dimensions();
                let start_x = (actual_size - lw) / 2;
                let start_y = (actual_size - lh) / 2;

                // 绘制背景衬垫 (避免 Logo 重叠二维码点阵)
                let pad = logo_cfg.border_padding;
                let pad_x = start_x.saturating_sub(pad);
                let pad_y = start_y.saturating_sub(pad);
                let pad_w = (lw + 2 * pad).min(actual_size - pad_x);
                let pad_h = (lh + 2 * pad).min(actual_size - pad_y);

                for py in pad_y..(pad_y + pad_h) {
                    for px in pad_x..(pad_x + pad_w) {
                        img.put_pixel(px, py, self.bg_color);
                    }
                }

                // 混合合成 Logo 图片
                imageops::overlay(&mut img, &resized_logo, start_x as i64, start_y as i64);
            }
        }

        let mut dynamic_img = DynamicImage::ImageRgba8(img);

        // 5. 保持最近邻采样缩放至精确定制的尺寸 self.size
        if actual_size != self.size {
            dynamic_img =
                dynamic_img.resize_exact(self.size, self.size, imageops::FilterType::Nearest);
        }

        Ok(dynamic_img)
    }

    /// 导出为 PNG 格式二进制 Byte 数组 (返回 `bytes::Bytes` 实现零拷贝)
    pub fn to_png_bytes(&self) -> MediaResult<Bytes> {
        let img = self.to_dynamic_image()?;
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        img.write_to(&mut cursor, ImageFormat::Png)?;
        Ok(Bytes::from(buffer))
    }

    /// 导出为 Base64 Data URL 字符串 (用于前端 `<img src="...">` 直接展示)
    pub fn to_data_url(&self) -> MediaResult<String> {
        let bytes = self.to_png_bytes()?;
        let b64 = BASE64.encode(&bytes);
        Ok(format!("data:image/png;base64,{b64}"))
    }

    /// 导出为 SVG 矢量文本字符串
    pub fn to_svg_string(&self) -> MediaResult<String> {
        if self.content.is_empty() {
            return Err(MediaError::InvalidInput(
                "QR code content cannot be empty".to_string(),
            ));
        }

        let code = QrCode::with_error_correction_level(self.content.as_bytes(), self.ec_level)?;
        let fg_hex = rgba_to_hex(self.fg_color);
        let bg_hex = rgba_to_hex(self.bg_color);

        let svg_str = code
            .render::<svg::Color>()
            .min_dimensions(self.size, self.size)
            .dark_color(svg::Color(&fg_hex))
            .light_color(svg::Color(&bg_hex))
            .quiet_zone(self.margin > 0)
            .build();

        Ok(svg_str)
    }
}
