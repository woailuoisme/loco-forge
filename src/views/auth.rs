use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::models::_entities::users;

/// 登录成功响应数据对象
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct LoginResponse {
    /// JWT 认证 Token
    pub token: String,
    /// 用户公共 PID (UUID)
    pub pid: String,
    /// 用户姓名
    pub name: String,
    /// 邮箱是否已验证
    pub is_verified: bool,
}

impl LoginResponse {
    #[must_use]
    pub fn new(user: &users::Model, token: &str) -> Self {
        Self {
            token: token.to_owned(),
            pid: user.pid.to_string(),
            name: user.name.clone(),
            is_verified: user.email_verified_at.is_some(),
        }
    }
}

/// 当前登录用户信息响应
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CurrentResponse {
    /// 用户公共 PID (UUID)
    pub pid: String,
    /// 用户姓名
    pub name: String,
    /// 用户邮箱
    pub email: String,
}

impl CurrentResponse {
    #[must_use]
    pub fn new(user: &users::Model) -> Self {
        Self {
            pid: user.pid.to_string(),
            name: user.name.clone(),
            email: user.email.clone(),
        }
    }
}
