//! # HTTP 日志中间件
//!
//! 提供 HTTP 请求响应统计、耗时测量与日志输出中间件。

use loco_rs::{app::AppContext, Result};

/// 全局 HTTP 请求日志处理中间件
///
/// 记录每一个经过 Axum Router 的请求，输出请求方法、请求路径、状态码与执行耗时。
pub async fn log_request(req: axum::extract::Request, next: axum::middleware::Next) -> axum::response::Response {
    // 步骤 1: 提取 HTTP 请求方法与 URI 路径
    let method = req.method().to_string();
    let path = req.uri().path().to_string();

    // 步骤 2: 记录请求处理开始时间
    let start = std::time::Instant::now();

    // 步骤 3: 传递给下一个中间件 / 控制器处理
    let response = next.run(req).await;

    // 步骤 4: 统计处理耗时与响应 HTTP 状态码
    let duration = start.elapsed();
    let status = response.status().as_u16();

    // 步骤 5: 强制打到 stderr 控制台 (防过滤) 并同步打入 Tracing 日志系统
    eprintln!("   --> [HTTP] {} {} -> {} ({:?})", method, path, status, duration);
    tracing::info!(
        target: "app",
        "[HTTP] {} {} -> {} ({:?})",
        method, path, status, duration
    );

    response
}

/// 路由组装后的中间件挂载钩子
///
/// 在路由完全构建后，向 Router 注册 `log_request` 全局中间件。
pub async fn after_routes(router: axum::Router, _ctx: &AppContext) -> Result<axum::Router> {
    // 1. 将 log_request 中间件绑定到全局 Axum 路由层
    Ok(router.layer(axum::middleware::from_fn(log_request)))
}
