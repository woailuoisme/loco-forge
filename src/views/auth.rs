//! Auth 视图契约重定向
//!
//! 所有认证响应 DTO 已统一下沉收拢至 `crate::dtos::auth::*`。
//! 本模块重定向保持 100% 向上兼容。

pub use crate::dtos::auth::{CurrentUserResource, LoginResource};
