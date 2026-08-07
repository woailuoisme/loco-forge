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

/// 初始化类似 Laravel `daily` 模式的按天自动切割离线日志文件写入器
///
/// 日志文件将自动保存至指定目录（如 `./logs`），生成的日志文件名形如 `2026-08-06.log` 或 `app-2026-08-06.log`。
///
/// # 返回值
/// 返回 `tracing_appender::non_blocking::WorkerGuard` 异步写入守护句柄。
/// **必须在主程序生命周期中持续持有该句柄**，确保应用退出时缓冲区内的日志被安全刷入磁盘。
pub fn init_daily_file_logger(folder: &str, filename_prefix: &str) -> tracing_appender::non_blocking::WorkerGuard {
    // 使用 Builder 配置生成类似 Laravel 风格的 YYYY-MM-DD.log 日志文件名
    let file_appender = tracing_appender::rolling::Builder::new()
        .rotation(tracing_appender::rolling::Rotation::DAILY)
        .filename_prefix(filename_prefix)
        .filename_suffix("log")
        .build(folder)
        .expect("初始化文件日志追加器失败");

    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_ansi(false)
        .init();

    guard
}
