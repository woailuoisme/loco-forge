//! 管理后台服务模块 (Admin Services)
//!
//! 承载业务管理域能力：
//! - [`auth`]：后台管理员认证 (管理员闸门 + 复用认证域实现)；
//! - [`user`]：用户运维 (分页检索、字段编辑、软删注销、Excel 批量导入导出)。
//!
//! 封禁 / 角色设置等认证域操作复用 [`crate::services::SessionService`]，不在此重复实现。

pub mod auth;
pub mod user;

pub use auth::AdminAuthService;
pub use user::AdminUserService;
