use std::any::Any;
use std::backtrace::Backtrace;

/// 格式化捕获到的 Panic Payload 为可读字符串
pub fn format_panic_message(payload: &(dyn Any + Send)) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "Unspecified or unknown panic payload".to_string()
    }
}

/// 捕获并使用 `log_error!` 打印 Panic 堆栈信息
pub fn log_panic(payload: &(dyn Any + Send)) {
    let msg = format_panic_message(payload);
    let backtrace = Backtrace::capture();

    crate::log_error!(
        "HTTP Handler Panic Executed",
        error = %msg,
        backtrace = %backtrace
    );
}

/// 挂载全局 `std::panic::set_hook`
/// 确保除了 HTTP 中间件外，任何线程出现未捕获的 Panic 都会自动写入 Logger
pub fn init_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let payload = info.payload();
        let msg = format_panic_message(payload);
        let location = info
            .location()
            .map_or_else(|| "unknown location".to_string(), |loc| loc.to_string());

        let backtrace = Backtrace::capture();

        crate::log_error!(
            "Application Crash/Panic Triggered",
            error = %msg,
            location = %location,
            backtrace = %backtrace
        );
    }));
}
