use axum::{
    body::Body,
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};

/// 媒体处理 HTTP Response 结构体 (MediaResponse)
#[derive(Debug, Clone)]
pub struct MediaResponse {
    pub bytes: Vec<u8>,
    pub content_type: &'static str,
    pub cache_control: Option<String>,
}

impl MediaResponse {
    /// 构造新的 MediaResponse 实例
    pub fn new(bytes: Vec<u8>, content_type: &'static str) -> Self {
        Self {
            bytes,
            content_type,
            cache_control: None,
        }
    }

    /// 设置 HTTP 缓存控制 (Cache-Control: public, max-age=N)
    pub fn with_cache_secs(mut self, max_age_secs: u32) -> Self {
        self.cache_control = Some(format!("public, max-age={max_age_secs}"));
        self
    }

    /// 包装 PNG 格式二进制图片响应 (Content-Type: image/png)
    pub fn png(bytes: Vec<u8>) -> Self {
        Self::new(bytes, "image/png")
    }

    /// 包装 JPEG 格式二进制图片响应 (Content-Type: image/jpeg)
    pub fn jpeg(bytes: Vec<u8>) -> Self {
        Self::new(bytes, "image/jpeg")
    }

    /// 包装 WebP 高效格式二进制图片响应 (Content-Type: image/webp)
    pub fn webp(bytes: Vec<u8>) -> Self {
        Self::new(bytes, "image/webp")
    }

    /// 包装 SVG 矢量格式字符串响应 (Content-Type: image/svg+xml)
    pub fn svg(svg_content: impl Into<String>) -> Self {
        Self::new(svg_content.into().into_bytes(), "image/svg+xml")
    }

    /// 包装带 HTTP 缓存 (Cache-Control: public, max-age=N) 的媒体响应
    pub fn with_cache(bytes: Vec<u8>, content_type: &'static str, max_age_secs: u32) -> Self {
        Self::new(bytes, content_type).with_cache_secs(max_age_secs)
    }
}

impl IntoResponse for MediaResponse {
    fn into_response(self) -> Response {
        let mut response = (StatusCode::OK, Body::from(self.bytes)).into_response();

        if let Ok(val) = HeaderValue::from_str(self.content_type) {
            response.headers_mut().insert(header::CONTENT_TYPE, val);
        }

        let cache_str = self
            .cache_control
            .unwrap_or_else(|| "public, max-age=86400".to_string());

        if let Ok(val) = HeaderValue::from_str(&cache_str) {
            response.headers_mut().insert(header::CACHE_CONTROL, val);
        }

        response
    }
}
