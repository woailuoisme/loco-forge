use axum::{
    body::Body,
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};

/// 媒体处理 HTTP Response 辅助工具类 (MediaResponse)
pub struct MediaResponse;

impl MediaResponse {
    /// 包装 PNG 格式二进制图片响应 (Content-Type: image/png)
    pub fn png(bytes: Vec<u8>) -> Response {
        Self::build_response(bytes, "image/png", None)
    }

    /// 包装 JPEG 格式二进制图片响应 (Content-Type: image/jpeg)
    pub fn jpeg(bytes: Vec<u8>) -> Response {
        Self::build_response(bytes, "image/jpeg", None)
    }

    /// 包装 WebP 高效格式二进制图片响应 (Content-Type: image/webp)
    pub fn webp(bytes: Vec<u8>) -> Response {
        Self::build_response(bytes, "image/webp", None)
    }

    /// 包装 SVG 矢量格式字符串响应 (Content-Type: image/svg+xml)
    pub fn svg(svg_content: impl Into<String>) -> Response {
        Self::build_response(svg_content.into().into_bytes(), "image/svg+xml", None)
    }

    /// 包装带 HTTP 缓存 (Cache-Control: public, max-age=N) 的媒体响应
    pub fn with_cache(bytes: Vec<u8>, content_type: &str, max_age_secs: u32) -> Response {
        let cache_header = format!("public, max-age={}", max_age_secs);
        Self::build_response(bytes, content_type, Some(&cache_header))
    }

    /// 通用 HTTP 响应构建函数
    fn build_response(bytes: Vec<u8>, content_type: &str, cache_control: Option<&str>) -> Response {
        let mut response = (StatusCode::OK, Body::from(bytes)).into_response();

        if let Ok(val) = HeaderValue::from_str(content_type) {
            response.headers_mut().insert(header::CONTENT_TYPE, val);
        }

        if let Some(cache) = cache_control {
            if let Ok(val) = HeaderValue::from_str(cache) {
                response.headers_mut().insert(header::CACHE_CONTROL, val);
            }
        } else {
            // 默认推荐 HTTP 响应头部
            response
                .headers_mut()
                .insert(header::CACHE_CONTROL, HeaderValue::from_static("public, max-age=86400"));
        }

        response
    }
}
