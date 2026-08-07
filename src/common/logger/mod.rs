//! # 统一日志模块
//!
//! 提供类 Laravel 风格的简洁日志宏、Panic 堆栈写入与多 Driver (Console / Daily File / Stack) 日志初始化。
//!
//! ## 快速使用
//!
//! ```rust
//! use locos::common::logger::*;
//!
//! // 简单消息
//! log_info!("用户登录成功");
//!
//! // 带结构化字段
//! log_info!("用户登录", user_id = 1, action = "login");
//! log_warn!("权限不足", user_id = 1, resource = "admin_panel");
//! log_error!("数据库查询失败", error = %err);
//! log_debug!("调试信息", payload = ?request_body);
//! ```

pub mod config;
pub mod init;
#[macro_use]
pub mod macros;
pub mod panic;

pub use config::{DEFAULT_FILENAME_PREFIX, DEFAULT_LOG_DIR, LogDriver, LoggerConfig};
pub use init::init_logger;
pub use macros::*;
pub use panic::{init_panic_hook, log_panic};
