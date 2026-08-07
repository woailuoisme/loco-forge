//! # 日志宏定义
//!
//! 封装 `tracing` 宏，提供类 Laravel `Log::*()` 风格的统一日志调用接口。
//!
//! ## 示例
//!
//! ```rust
//! use locos::common::logger::*;
//!
//! log_info!("用户登录成功", user_id = 42);
//! log_warn!("请求速率过高", user_id = 42, limit = 100);
//! log_error!("服务异常", error = %"connection error");
//! log_debug!("请求体", payload = ?vec![1u8, 2, 3]);
//! ```

/// 输出 `ERROR` 级别日志（对应 Laravel `Log::error()`）
#[macro_export]
macro_rules! log_error {
    ($msg:expr) => {
        ::tracing::error!(target: "app", "{}", $msg)
    };
    ($msg:expr, $($key:ident = $val:expr),+ $(,)?) => {
        ::tracing::error!(target: "app", $($key = $val,)+ "{}", $msg)
    };
    ($msg:expr, $($key:ident = %$val:expr),+ $(,)?) => {
        ::tracing::error!(target: "app", $($key = %$val,)+ "{}", $msg)
    };
    ($msg:expr, $($key:ident = ?$val:expr),+ $(,)?) => {
        ::tracing::error!(target: "app", $($key = ?$val,)+ "{}", $msg)
    };
}

/// 输出 `WARN` 级别日志（对应 Laravel `Log::warning()`）
#[macro_export]
macro_rules! log_warn {
    ($msg:expr) => {
        ::tracing::warn!(target: "app", "{}", $msg)
    };
    ($msg:expr, $($key:ident = $val:expr),+ $(,)?) => {
        ::tracing::warn!(target: "app", $($key = $val,)+ "{}", $msg)
    };
    ($msg:expr, $($key:ident = %$val:expr),+ $(,)?) => {
        ::tracing::warn!(target: "app", $($key = %$val,)+ "{}", $msg)
    };
    ($msg:expr, $($key:ident = ?$val:expr),+ $(,)?) => {
        ::tracing::warn!(target: "app", $($key = ?$val,)+ "{}", $msg)
    };
}

/// 输出 `INFO` 级别日志（对应 Laravel `Log::info()`）
#[macro_export]
macro_rules! log_info {
    ($msg:expr) => {
        ::tracing::info!(target: "app", "{}", $msg)
    };
    ($msg:expr, $($key:ident = $val:expr),+ $(,)?) => {
        ::tracing::info!(target: "app", $($key = $val,)+ "{}", $msg)
    };
    ($msg:expr, $($key:ident = %$val:expr),+ $(,)?) => {
        ::tracing::info!(target: "app", $($key = %$val,)+ "{}", $msg)
    };
    ($msg:expr, $($key:ident = ?$val:expr),+ $(,)?) => {
        ::tracing::info!(target: "app", $($key = ?$val,)+ "{}", $msg)
    };
}

/// 输出 `DEBUG` 级别日志（对应 Laravel `Log::debug()`）
#[macro_export]
macro_rules! log_debug {
    ($msg:expr) => {
        ::tracing::debug!(target: "app", "{}", $msg)
    };
    ($msg:expr, $($key:ident = $val:expr),+ $(,)?) => {
        ::tracing::debug!(target: "app", $($key = $val,)+ "{}", $msg)
    };
    ($msg:expr, $($key:ident = %$val:expr),+ $(,)?) => {
        ::tracing::debug!(target: "app", $($key = %$val,)+ "{}", $msg)
    };
    ($msg:expr, $($key:ident = ?$val:expr),+ $(,)?) => {
        ::tracing::debug!(target: "app", $($key = ?$val,)+ "{}", $msg)
    };
}

/// 输出 `TRACE` 级别日志（对应 Laravel `Log::channel('trace')`）
#[macro_export]
macro_rules! log_trace {
    ($msg:expr) => {
        ::tracing::trace!(target: "app", "{}", $msg)
    };
    ($msg:expr, $($key:ident = $val:expr),+ $(,)?) => {
        ::tracing::trace!(target: "app", $($key = $val,)+ "{}", $msg)
    };
    ($msg:expr, $($key:ident = %$val:expr),+ $(,)?) => {
        ::tracing::trace!(target: "app", $($key = %$val,)+ "{}", $msg)
    };
    ($msg:expr, $($key:ident = ?$val:expr),+ $(,)?) => {
        ::tracing::trace!(target: "app", $($key = ?$val,)+ "{}", $msg)
    };
}

pub use log_debug;
pub use log_error;
pub use log_info;
pub use log_trace;
pub use log_warn;

#[cfg(test)]
mod tests {
    use crate::common::logger::config::{DEFAULT_FILENAME_PREFIX, DEFAULT_LOG_DIR, LogDriver, LoggerConfig};

    #[test]
    fn test_logger_config_default() {
        let cfg = LoggerConfig::default();
        assert_eq!(cfg.driver, LogDriver::Stack);
        assert_eq!(cfg.log_dir, std::path::Path::new(DEFAULT_LOG_DIR));
        assert_eq!(cfg.filename_prefix, DEFAULT_FILENAME_PREFIX);
        assert!(cfg.enable_ansi);
    }

    #[test]
    fn test_log_driver_traits() {
        use std::str::FromStr;

        assert_eq!(LogDriver::Console.to_string(), "console");
        assert_eq!(LogDriver::Daily.to_string(), "daily");
        assert_eq!(LogDriver::Single.to_string(), "single");
        assert_eq!(LogDriver::Json.to_string(), "json");
        assert_eq!(LogDriver::Stack.to_string(), "stack");

        assert_eq!("daily".parse::<LogDriver>(), Ok(LogDriver::Daily));
        assert_eq!("single".parse::<LogDriver>(), Ok(LogDriver::Single));
        assert_eq!("json".parse::<LogDriver>(), Ok(LogDriver::Json));
        assert_eq!(LogDriver::from_str("STACK"), Ok(LogDriver::Stack));
        assert!("invalid".parse::<LogDriver>().is_err());
    }

    #[test]
    fn test_logger_config_builder() {
        let cfg = LoggerConfig::new()
            .with_driver(LogDriver::Stack)
            .with_log_dir("./custom_logs")
            .with_filename_prefix("my_app")
            .with_ansi(false);

        assert_eq!(cfg.driver, LogDriver::Stack);
        assert_eq!(cfg.log_dir, std::path::PathBuf::from("./custom_logs"));
        assert_eq!(cfg.filename_prefix, "my_app");
        assert!(!cfg.enable_ansi);
        assert_eq!(cfg.log_path(), std::path::Path::new("./custom_logs"));
    }

    #[test]
    fn test_logger_macros_syntax() {
        log_info!("测试日志消息");
        log_info!("测试结构化日志", user_id = 100, action = "test");
        log_warn!("警告消息", reason = "timeout");
        log_error!("错误消息", code = 500);
        log_debug!("调试消息", payload = "debug_data");
        log_trace!("追踪消息", trace_id = "abc-123");
    }

    #[test]
    fn test_panic_formatting() {
        use crate::common::logger::panic::format_panic_message;
        let str_payload: &(dyn std::any::Any + Send) = &"test panic string";
        assert_eq!(format_panic_message(str_payload), "test panic string");
    }

    #[test]
    fn test_logger_file_output() {
        use crate::common::logger::init::init_logger;

        let mut log_dir_buf = std::env::temp_dir();
        log_dir_buf.push(format!(
            "loco_test_logs_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let log_dir_path = log_dir_buf.to_str().unwrap().to_string();

        let cfg = LoggerConfig::new()
            .with_driver(LogDriver::Daily)
            .with_log_dir(&log_dir_path)
            .with_filename_prefix("test_app");

        // 初始化 logger 并持有 guard
        let _guard = init_logger(&cfg);

        // 使用日志宏写入消息
        log_info!("单元测试写入文件消息", test_key = "test_value_123");
        log_error!("单元测试写入错误消息", err_code = 999);

        // 刷新并发给 WorkerGuard 刷盘
        drop(_guard);

        // 检查临时日志目录中是否成功生成了日志文件并包含写入的内容
        let mut log_files = std::fs::read_dir(&log_dir_path)
            .expect("Failed to read log dir")
            .filter_map(|res| res.ok())
            .collect::<Vec<_>>();

        assert!(
            !log_files.is_empty(),
            "Expected at least one log file created in {}",
            log_dir_path
        );

        let log_file_path = log_files.remove(0).path();
        let content = std::fs::read_to_string(&log_file_path).expect("Failed to read created log file");

        assert!(
            content.contains("单元测试写入文件消息"),
            "Log file content missing expected text. Content:\n{}",
            content
        );
        assert!(
            content.contains("test_value_123"),
            "Log file content missing structured field. Content:\n{}",
            content
        );
        assert!(
            content.contains("单元测试写入错误消息"),
            "Log file content missing error message. Content:\n{}",
            content
        );

        let _ = std::fs::remove_dir_all(&log_dir_path);
    }
}
