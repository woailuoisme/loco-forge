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

    /// 执行路由收集与表格打印
    async fn run(&self, app_context: &AppContext, _vars: &task::Vars) -> Result<()> {
        // 步骤 1: 调用 App::routes 收集全局组装好的路由集合
        let routes = App::routes(app_context);

        // 步骤 2: 打印表格 Header 头格式
        println!("\n+------------+---------------------------------------------------------+");
        println!("| {:<10} | {:<55} |", "Method", "URI");
        println!("+------------+---------------------------------------------------------+");

        // 步骤 3: 遍历路由项并按 HTTP 方法与 URI 格式化输出
        for route in routes.collect() {
            let methods = route.actions.iter().map(|a| a.as_str()).collect::<Vec<_>>().join("|");
            println!("| {:<10} | {:<55} |", methods, route.uri);
        }
        println!("+------------+---------------------------------------------------------+\n");
        Ok(())
    }
}
