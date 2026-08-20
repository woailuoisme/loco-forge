pub mod diagnostics_test;
pub mod excel_test;
pub mod media_test;
pub mod notification_test;
pub mod scout_test;
pub mod utils_test;

/// 统一 insta 快照配置：关闭模块前缀，可选设置快照后缀。
///
/// 宏会直接在调用作用域内生成 `let _guard`，使快照配置在整个函数体生效
/// （不可包裹在块表达式中，否则 guard 会立即被丢弃导致配置失效）。
///
/// 用法：
/// - `configure_insta!();` 默认后缀为 `"users"`
/// - `configure_insta!("auth_request");` 显式指定快照后缀
#[macro_export]
macro_rules! configure_insta {
    () => {
        $crate::configure_insta!("users");
    };
    ($suffix:expr) => {
        let _insta_guard = {
            let mut settings = insta::Settings::clone_current();
            settings.set_prepend_module_to_snapshot(false);
            settings.set_snapshot_suffix($suffix);
            settings.bind_to_scope()
        };
    };
}
