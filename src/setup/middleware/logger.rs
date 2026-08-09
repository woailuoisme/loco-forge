//! # HTTP 日志中间件
//!
//! 提供 HTTP 请求响应统计、耗时测量与日志输出中间件。

use axum::response::IntoResponse;

/// 全局未匹配路由 (404 Not Found) Fallback 处理器
///
/// 当请求的 URL 路径不存在时，统一返回包含 404003 错误码的标准 Route 404 JSON。
pub async fn not_found_fallback() -> axum::response::Response {
    crate::dtos::common::format_res::route_not_found()
        .unwrap_or_else(|_| (axum::http::StatusCode::NOT_FOUND, "Not Found").into_response())
}

/// 全局 HTTP 请求日志处理中间件
///
/// 记录每一个经过 Axum Router 的请求，输出请求方法、请求路径、状态码与执行耗时。
pub async fn log_request(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let method = req.method().clone();
    let path = req
        .uri()
        .path_and_query()
        .map_or_else(|| req.uri().path(), |pq| pq.as_str())
        .to_string();
    let version = req.version();

    let client_ip = req
        .headers()
        .get("x-forwarded-for")
        .or_else(|| req.headers().get("x-real-ip"))
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map_or("127.0.0.1", str::trim)
        .to_string();

    let start = std::time::Instant::now();
    let response = next.run(req).await;
    let duration = start.elapsed();
    let status = response.status().as_u16();

    // 格式化输出结构化 HTTP 请求日志
    tracing::info!(
        target: "app",
        version = ?version,
        method = %method,
        path = %path,
        status = status,
        latency_ms = %duration.as_secs_f64() * 1000.0,
        client_ip = %client_ip,
        "HTTP request"
    );

    response
}
