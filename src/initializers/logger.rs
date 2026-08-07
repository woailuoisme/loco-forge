//! # 日志系统 Initializer
//!
//! 在应用启动引导阶段初始化类 Laravel 风格的统一日志模块，
//! 挂载 Panic Hook 并根据配置开启控制台 / 按天滚动日志文件落盘。

use async_trait::async_trait;
use loco_rs::{
    Result,
    app::{AppContext, Initializer},
};
use tracing_appender::non_blocking::WorkerGuard;

use crate::common::logger::{DEFAULT_FILENAME_PREFIX, DEFAULT_LOG_DIR};

/// 全局持有的 Logger WorkerGuard 避免进程退出前缓冲区日志丢失
static LOGGER_GUARD: std::sync::Mutex<Option<WorkerGuard>> = std::sync::Mutex::new(None);

pub struct LoggerInitializer;

#[async_trait]
impl Initializer for LoggerInitializer {
    fn name(&self) -> String {
        "logger".to_string()
    }

    async fn before_run(&self, _ctx: &AppContext) -> Result<()> {
        let driver = std::env::var("LOG_DRIVER")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(crate::common::logger::LogDriver::Stack);

        let log_dir = std::env::var("LOG_DIR").unwrap_or_else(|_| DEFAULT_LOG_DIR.to_string());
        let prefix = std::env::var("LOG_FILENAME_PREFIX").unwrap_or_else(|_| DEFAULT_FILENAME_PREFIX.to_string());

        let config = crate::common::logger::LoggerConfig::new()
            .with_driver(driver)
            .with_log_dir(&log_dir)
            .with_filename_prefix(&prefix);

        let driver_str = config.driver.to_string();
        let log_path_display = config.log_path().display().to_string();

        if let Some(guard) = crate::common::logger::init_logger(&config)
            && let Ok(mut g) = LOGGER_GUARD.lock()
        {
            *g = Some(guard);
        }

        crate::log_info!(
            "统一日志初始化成功",
            dir = %log_path_display,
            driver = %driver_str
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[serial_test::serial]
    async fn test_logger_initializer() {
        let initializer = LoggerInitializer;
        assert_eq!(initializer.name(), "logger");

        loco_rs::testing::prelude::request::<crate::app::App, _, _>(|_req, ctx| async move {
            let initializer = LoggerInitializer;
            let _ = initializer.before_run(&ctx).await;
            crate::log_info!("初始化测试写入文件日志", dir = DEFAULT_LOG_DIR);
        })
        .await;
    }
}
