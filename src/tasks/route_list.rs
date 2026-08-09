//! # 路由列表打印任务
//!
//! 提供 `cargo loco task route:list` 命令行任务，模仿 Laravel 的 `route:list` 风格在终端格式化输出当前应用所有注册的 HTTP API 路由表。

use crate::app::App;
use loco_rs::app::Hooks;
use loco_rs::prelude::*;

/// 路由列表打印任务
pub struct RouteList;

#[async_trait]
impl Task for RouteList {
    /// 定义 Task 的 CLI 名称与描述信息
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "route:list".to_string(),
            detail: "以 Laravel route:list 格式化表格样式打印所有已注册的 HTTP API 路由".to_string(),
        }
    }

    /// 执行路由收集与日志打印
    async fn run(&self, app_context: &AppContext, _vars: &task::Vars) -> Result<()> {
        // 步骤 1: 调用 App::routes 收集全局组装好的路由集合
        let routes = App::routes(app_context).collect();

        // 步骤 2: 动态计算 Method 与 URI 的最大长度（以 10 和 55 为基准下限）
        let mut max_method_len = 10;
        let mut max_uri_len = 55;

        let mut route_entries = Vec::with_capacity(routes.len());
        for route in routes {
            let methods = route.actions.iter().map(reqwest::Method::as_str).collect::<Vec<_>>().join("|");
            max_method_len = max_method_len.max(methods.len());
            max_uri_len = max_uri_len.max(route.uri.len());
            route_entries.push((methods, route.uri));
        }

        // 步骤 3: 绘制自适应表格 Header 与分隔线
        let border = format!("+-{}-+-{}-+", "-".repeat(max_method_len), "-".repeat(max_uri_len));

        let mut output = String::new();
        output.push('\n');
        output.push_str(&border);
        output.push('\n');
        output.push_str(&format!(
            "| {:<width_m$} | {:<width_u$} |\n",
            "Method",
            "URI",
            width_m = max_method_len,
            width_u = max_uri_len
        ));
        output.push_str(&border);
        output.push('\n');

        // 步骤 4: 填充每条路由记录
        for (methods, uri) in route_entries {
            output.push_str(&format!(
                "| {:<width_m$} | {:<width_u$} |\n",
                methods,
                uri,
                width_m = max_method_len,
                width_u = max_uri_len
            ));
        }
        output.push_str(&border);

        tracing::info!(target: "app", "{}", output);
        Ok(())
    }
}
