//! 管理后台控制器模块 (Admin Controllers)
//!
//! 业务管理域的后台接口，挂载于 `/v1/admin/*`。
//! 与 `/v1/auth/admin/*` (better-auth admin 插件契约) 并存，职责互不重叠：
//! - `auth/admin`：认证域插件端点，面向 SDK 调用方；
//! - `admin`：业务管理域，面向后台运维界面 (认证 / CRUD / 软删 / Excel 批量)。

pub mod auth;
pub mod user;

use loco_rs::prelude::*;

use crate::{common::consts::api::API_PREFIX, setup::middleware::rate_limit};

/// 后台登录路由 (单独挂载更严格的限流层)
///
/// 配额 `per_second(1)` + `burst_size(3)`，显著高于其余端点的防护强度 ——
/// 后台登录成功一次即获得全量用户管理权限，是最需要防爆破的单点。
pub fn login_routes() -> Routes {
    Routes::new()
        .prefix(&format!("{API_PREFIX}/admin/auth"))
        .add("/login", post(auth::login))
        .layer(rate_limit::admin_login_rate_limit_layer())
}

/// 后台其余路由 (认证辅助 + 用户管理)
pub fn routes() -> Routes {
    Routes::new()
        .prefix(&format!("{API_PREFIX}/admin"))
        // 后台认证辅助
        .add("/auth/logout", post(auth::logout))
        .add("/auth/me", get(auth::me))
        // 用户列表与创建
        .add("/users", get(user::list).post(user::create))
        // Excel 批量导入导出 (须在 `/{id}` 之前注册，避免被路径参数吞掉)
        .add("/users/export", get(user::export))
        .add("/users/import", post(user::import))
        // 单用户 CRUD
        .add(
            "/users/{id}",
            get(user::get_one)
                .patch(user::update)
                .delete(user::soft_delete),
        )
        // 封禁 / 解封 (复用 AuthService 的实现)
        .add("/users/{id}/bans", post(user::ban).delete(user::unban))
}
