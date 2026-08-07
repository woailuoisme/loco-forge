//! # 控制器路由组装模块
//!
//! 负责聚合与注册应用中所有的 HTTP 控制器 (Controller) 路由。

use crate::controllers;
use loco_rs::{app::AppContext, controller::AppRoutes};

/// 构建并返回所有 HTTP 业务路由
///
/// 包含全局默认路由（健康检查、_ping 等）以及已注册的各个控制器路由。
pub fn routes(_ctx: &AppContext) -> AppRoutes {
    // 步骤 1: 初始化包含系统默认路由 (如 _ping, _health) 的路由集合
    AppRoutes::with_default_routes()
        // 步骤 2: 挂载首页 (Home) 控制器路由
        .add_route(controllers::home::routes())
        // 步骤 3: 挂载文章 (Posts v1 Admin) CRUD 控制器路由
        .add_route(controllers::v1::admin::posts::routes())
        // 步骤 4: 挂载身份认证 (Auth v1) 控制器路由 (注册、登录、魔术链接等)
        .add_route(controllers::v1::auth::routes())
        // 步骤 5: 挂载健康检查 (Health) 控制器路由 (liveness, readiness)
        .add_route(controllers::health::routes())
        // 步骤 6: 挂载文件上传下载 (Files v1) 控制器路由
        .add_route(controllers::v1::files::routes())
}
