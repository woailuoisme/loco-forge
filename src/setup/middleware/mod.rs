//! # HTTP 中间件注册与管理子模块
//!
//! 该模块集中管理 Loco 应用的所有全局/局部 HTTP 中间件，
//! 包含多语言国际化 (i18n)、日志记录 (logger) 与速率限制 (rate_limit)。

/// 多语言国际化 (i18n) 解析与中间件
pub mod i18n;
/// HTTP 请求耗时统计与全局日志中间件
pub mod logger;
/// HTTP 请求速率限制 (Rate Limiting) 防暴力破解中间件
pub mod rate_limit;

use loco_rs::{Result, app::AppContext};

/// 统一注册挂载全局 HTTP 中间件与 404 Fallback
///
/// 包含逻辑：
/// 1. 挂载 404 Route Not Found 回退响应处理器 (`logger::not_found_fallback`)
/// 2. 挂载全局多语言解析中间件 (`i18n::i18n_middleware`)
/// 3. 根据配置开关选择性挂载全局请求日志中间件 (`logger::log_request`)
pub async fn register(router: axum::Router, ctx: &AppContext) -> Result<axum::Router> {
    let enabled = ctx.config.server.middlewares.logger.as_ref().is_none_or(|m| m.enable);

    let router = router
        .fallback(logger::not_found_fallback)
        .layer(axum::middleware::from_fn(i18n::i18n_middleware));

    if enabled {
        Ok(router.layer(axum::middleware::from_fn(logger::log_request)))
    } else {
        Ok(router)
    }
}
