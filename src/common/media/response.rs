use axum::{
    body::Body,
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use bytes::Bytes;

/// 媒体处理 HTTP Response 结构体 (MediaResponse)
#[derive(Debug, Clone)]
pub struct MediaResponse {
    /// 媒体二进制字节数据 (采用 `bytes::Bytes` 实现 O(1) 廉价克隆与零拷贝 HTTP Body 对接)
    pub bytes: Bytes,
    /// HTTP Content-Type 类型
    pub content_type: &'static str,
    /// HTTP 缓存控制指令
    pub cache_control: Option<String>,
}

impl MediaResponse {
    /// 构造新的 MediaResponse 实例 (零拷贝接收 `impl Into<Bytes>`)
    pub fn new(bytes: impl Into<Bytes>, content_type: &'static str) -> Self {
        Self {
            bytes: bytes.into(),
            content_type,
            cache_control: None,
        }
    }

    /// 设置 HTTP 缓存控制 (Cache-Control: public, max-age=N)
    #[must_use]
    pub fn with_cache_secs(mut self, max_age_secs: u32) -> Self {
        self.cache_control = Some(format!("public, max-age={max_age_secs}"));
        self
    }

    /// 包装 PNG 格式二进制图片响应 (Content-Type: image/png)
    pub fn png(bytes: impl Into<Bytes>) -> Self {
        Self::new(bytes, "image/png")
    }

    /// 包装 JPEG 格式二进制图片响应 (Content-Type: image/jpeg)
    pub fn jpeg(bytes: impl Into<Bytes>) -> Self {
        Self::new(bytes, "image/jpeg")
    }

    /// 包装 WebP 高效格式二进制图片响应 (Content-Type: image/webp)
    pub fn webp(bytes: impl Into<Bytes>) -> Self {
        Self::new(bytes, "image/webp")
    }

    /// 包装 SVG 矢量格式字符串响应 (Content-Type: image/svg+xml)
    pub fn svg(svg_content: impl Into<String>) -> Self {
        Self::new(Bytes::from(svg_content.into()), "image/svg+xml")
    }

    /// 包装带 HTTP 缓存 (Cache-Control: public, max-age=N) 的媒体响应
    pub fn with_cache(bytes: impl Into<Bytes>, content_type: &'static str, max_age_secs: u32) -> Self {
        Self::new(bytes, content_type).with_cache_secs(max_age_secs)
    }
}

impl IntoResponse for MediaResponse {
    fn into_response(self) -> Response {
        let mut response = (StatusCode::OK, Body::from(self.bytes)).into_response();

        if let Ok(val) = HeaderValue::from_str(self.content_type) {
            response.headers_mut().insert(header::CONTENT_TYPE, val);
        }

        let cache_val = match self.cache_control {
            Some(ref cache_str) => {
                HeaderValue::from_str(cache_str).unwrap_or_else(|_| HeaderValue::from_static("public, max-age=86400"))
            }
            None => HeaderValue::from_static("public, max-age=86400"),
        };

        response.headers_mut().insert(header::CACHE_CONTROL, cache_val);

        response
    }
}
