//! # Setup 模块注册中心
//!
//! 该模块集中管理 Loco 应用启动时的基础设施初始化与服务注册逻辑，
//! 包括 HTTP 日志中间件、路由组装、命令行任务 (Tasks) 和后台工作者 (Workers)。

/// HTTP 中间件注册与管理 (i18n, logger, rate_limit)
pub mod middleware;
/// Controller 控制器路由组装
pub mod routes;
/// 命令行 / 管理员 Task 注册
pub mod tasks;
/// 后台异步 Worker 队列注册
pub mod workers;
