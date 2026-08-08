pub mod excel_test;
pub mod media_test;
pub mod notification_test;
pub mod scout_test;
pub mod utils_test;

/// 统一 insta 快照配置：关闭模块前缀，可选设置快照后缀。
///
/// 用法：
/// - `configure_insta!();` 仅关闭模块前缀
/// - `configure_insta!("auth_request");` 额外设置快照后缀
#[macro_export]
macro_rules! configure_insta {
    () => {{
        let mut settings = insta::Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        let _guard = settings.bind_to_scope();
    }};
    ($suffix:expr) => {{
        let mut settings = insta::Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_suffix($suffix);
        let _guard = settings.bind_to_scope();
    }};
}
