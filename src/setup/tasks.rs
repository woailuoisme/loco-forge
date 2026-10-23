//! # 命令行任务 (Task) 注册模块
//!
//! 集中注册系统维护、调试与数据注入相关的命令行 CLI Task。

use crate::tasks;
use loco_rs::task::Tasks;

/// 注册所有命令行 Tasks
///
/// 允许通过 `cargo loco task <name>` 调起指定的后台任务。
pub fn register_tasks(tasks: &mut Tasks) {
    // 步骤 1: 注册 JWT Secret 生成器任务
    tasks.register(tasks::jwt_secret::JwtSecret);

    // 步骤 2: 注册 路由表打印任务
    tasks.register(tasks::route_list::RouteList);

    // 步骤 3: 注册 健康检查任务
    tasks.register(tasks::health_check::HealthCheck);

    // 步骤 4: 保留 Loco 脚手架注入锚点 (新增 task 生成器自动插入点)
    // tasks-inject (do not remove)
    tasks.register(tasks::clean_expired_tokens::CleanExpiredTokens);
    tasks.register(tasks::user_create::UserCreate);
    tasks.register(tasks::seed_data::SeedData);
}
