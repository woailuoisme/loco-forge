//! 管理后台 DTO 模块 (Admin DTOs)
//!
//! 与 `dtos::auth::admin` (better-auth admin 插件契约) 并存：
//! 本模块服务**业务管理域**——后台可视化列表、字段编辑与 Excel 批量运维。

pub mod auth;
pub mod user;

pub use auth::*;
pub use user::*;
