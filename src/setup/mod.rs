//! # Setup 模块注册中心
//!
//! 该模块集中管理 Loco 应用启动时的基础设施初始化与服务注册逻辑，
//! 包括 HTTP 日志中间件、路由组装、命令行任务 (Tasks) 和后台工作者 (Workers)。

/// 多语言国际化 (i18n) 解析与中间件
pub mod i18n;
/// HTTP 日志与中间件钩子
pub mod logger;
/// Controller 控制器路由组装
pub mod routes;
/// 命令行 / 管理员 Task 注册
pub mod tasks;
/// 后台异步 Worker 队列注册
pub mod workers;
