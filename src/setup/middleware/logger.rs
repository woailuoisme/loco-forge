//! # HTTP 日志中间件
//!
//! 提供 HTTP 请求响应统计、耗时测量与日志输出中间件。

use axum::response::IntoResponse;
use loco_rs::{Result, app::AppContext};

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
pub async fn log_request(req: axum::extract::Request, next: axum::middleware::Next) -> axum::response::Response {
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

    // 协议版本置顶最前 (如: [HTTP/1.1] GET / 200 1.105ms | 127.0.0.1)
    tracing::info!(
        target: "app",
        "[{:?}] {} {} {} {:?} | {}",
        version, method, path, status, duration, client_ip
    );

    response
}

/// 路由组装后的中间件挂载钩子
///
/// 根据配置文件 `server.middlewares.logger` 的开关状态，决定是否挂载 `log_request` 全局中间件。
pub async fn after_routes(router: axum::Router, ctx: &AppContext) -> Result<axum::Router> {
    let enabled = ctx.config.server.middlewares.logger.as_ref().is_none_or(|m| m.enable);

    let router = router.fallback(not_found_fallback).layer(axum::middleware::from_fn(
        crate::setup::middleware::i18n::i18n_middleware,
    ));

    if enabled {
        Ok(router.layer(axum::middleware::from_fn(log_request)))
    } else {
        Ok(router)
    }
}
