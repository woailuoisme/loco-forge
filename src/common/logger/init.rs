use super::config::{LogDriver, LoggerConfig};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

/// 根据 `LoggerConfig` 初始化全局日志 Subscriber
///
/// # 返回值
/// 如果开启了文件日志（`Daily` 或 `Stack` 模式），会返回 `Some(WorkerGuard)`。
/// 该守护句柄**必须在应用主进程中持续持有**，确保退出时刷新日志缓冲区。
/// 创建控制台日志 Layer
fn build_console_layer<S>(enable_ansi: bool) -> impl tracing_subscriber::Layer<S>
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    tracing_subscriber::fmt::layer()
        .with_ansi(enable_ansi)
        .with_target(true)
}

/// 创建按天滚动的离线文件日志 Appender 与 Layer
fn build_file_layer<S>(
    log_dir: &std::path::Path,
    prefix: &str,
) -> Result<(impl tracing_subscriber::Layer<S>, WorkerGuard), String>
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    let file_appender = tracing_appender::rolling::Builder::new()
        .rotation(tracing_appender::rolling::Rotation::DAILY)
        .filename_prefix(prefix)
        .filename_suffix("log")
        .build(log_dir)
        .map_err(|e| format!("初始化按天日志追加器失败: {e}"))?;

    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    let layer = tracing_subscriber::fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_target(true);

    Ok((layer, guard))
}

/// 创建单文件 (Single File) Appender 与 Layer (不按时间轮转，写入固定文件名如 loco-single.log)
fn build_single_file_layer<S>(
    log_dir: &std::path::Path,
    prefix: &str,
) -> Result<(impl tracing_subscriber::Layer<S>, WorkerGuard), String>
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    let filename = format!("{prefix}single.log");
    let file_appender = tracing_appender::rolling::never(log_dir, filename);
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    let layer = tracing_subscriber::fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_target(true);

    Ok((layer, guard))
}

/// 创建结构化 JSON 格式按天轮转文件 Appender 与 Layer
fn build_json_file_layer<S>(
    log_dir: &std::path::Path,
    prefix: &str,
) -> Result<(impl tracing_subscriber::Layer<S>, WorkerGuard), String>
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    let file_appender = tracing_appender::rolling::Builder::new()
        .rotation(tracing_appender::rolling::Rotation::DAILY)
        .filename_prefix(prefix)
        .filename_suffix("json.log")
        .build(log_dir)
        .map_err(|e| format!("初始化 JSON 日志追加器失败: {e}"))?;

    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    let layer = tracing_subscriber::fmt::layer()
        .json()
        .with_writer(non_blocking)
        .with_target(true);

    Ok((layer, guard))
}

/// 创建控制台 JSON Lines 日志 Layer
fn build_json_console_layer<S>() -> impl tracing_subscriber::Layer<S>
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    tracing_subscriber::fmt::layer().json().with_target(true)
}

/// 根据 `LoggerConfig` 初始化全局日志 Subscriber
///
/// # 返回值
/// 如果开启了文件日志（`Daily`、`Single`、`Json` 或 `Stack` 模式），会返回 `Some(WorkerGuard)`。
/// 该守护句柄**必须在应用主进程中持续持有**，确保退出时刷新日志缓冲区。
pub fn init_logger(config: &LoggerConfig) -> Option<WorkerGuard> {
    // 自动挂载 Panic 钩子，将未捕获崩溃与 Backtrace 自动记录至日志
    super::panic::init_panic_hook();

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,locos=debug,app=debug"));

    // 对于需要写文件的日志模式，首先尝试递归创建目标日志目录
    let need_file_log = matches!(
        config.driver,
        LogDriver::Daily | LogDriver::Single | LogDriver::Json | LogDriver::Stack
    );
    if need_file_log && let Err(err) = config.ensure_log_dir() {
        eprintln!(
            "[logger] 警告: 无法创建日志目录 '{:?}': {err}，安全降级为 Console 控制台日志模式。",
            config.log_path()
        );
        let _ = tracing_subscriber::registry()
            .with(env_filter)
            .with(build_console_layer(config.enable_ansi))
            .try_init();
        return None;
    }

    match config.driver {
        LogDriver::Console => {
            let _ = tracing_subscriber::registry()
                .with(env_filter)
                .with(build_console_layer(config.enable_ansi))
                .try_init();
            None
        }
        LogDriver::Daily => match build_file_layer(config.log_path(), &config.filename_prefix) {
            Ok((file_layer, guard)) => {
                let _ = tracing_subscriber::registry()
                    .with(env_filter)
                    .with(file_layer)
                    .try_init();
                Some(guard)
            }
            Err(err_msg) => {
                eprintln!("[logger] 警告: {err_msg}，安全降级为 Console 控制台日志模式。");
                let _ = tracing_subscriber::registry()
                    .with(env_filter)
                    .with(build_console_layer(config.enable_ansi))
                    .try_init();
                None
            }
        },
        LogDriver::Single => match build_single_file_layer(config.log_path(), &config.filename_prefix) {
            Ok((file_layer, guard)) => {
                let _ = tracing_subscriber::registry()
                    .with(env_filter)
                    .with(file_layer)
                    .try_init();
                Some(guard)
            }
            Err(err_msg) => {
                eprintln!("[logger] 警告: {err_msg}，安全降级为 Console 控制台日志模式。");
                let _ = tracing_subscriber::registry()
                    .with(env_filter)
                    .with(build_console_layer(config.enable_ansi))
                    .try_init();
                None
            }
        },
        LogDriver::Json => match build_json_file_layer(config.log_path(), &config.filename_prefix) {
            Ok((file_layer, guard)) => {
                let json_console_layer = build_json_console_layer();
                let _ = tracing_subscriber::registry()
                    .with(env_filter)
                    .with(json_console_layer)
                    .with(file_layer)
                    .try_init();
                Some(guard)
            }
            Err(err_msg) => {
                eprintln!("[logger] 警告: {err_msg}，安全降级为 Console JSON 日志模式。");
                let json_console_layer = build_json_console_layer();
                let _ = tracing_subscriber::registry()
                    .with(env_filter)
                    .with(json_console_layer)
                    .try_init();
                None
            }
        },
        LogDriver::Stack => match build_file_layer(config.log_path(), &config.filename_prefix) {
            Ok((file_layer, guard)) => {
                let console_layer = build_console_layer(config.enable_ansi);
                let _ = tracing_subscriber::registry()
                    .with(env_filter)
                    .with(console_layer)
                    .with(file_layer)
                    .try_init();
                Some(guard)
            }
            Err(err_msg) => {
                eprintln!("[logger] 警告: {err_msg}，降级为单 Console 控制台日志模式。");
                let console_layer = build_console_layer(config.enable_ansi);
                let _ = tracing_subscriber::registry()
                    .with(env_filter)
                    .with(console_layer)
                    .try_init();
                None
            }
        },
    }
}
