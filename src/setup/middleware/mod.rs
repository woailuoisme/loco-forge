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
