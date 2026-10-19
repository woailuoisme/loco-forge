//! RBAC (Role-Based Access Control) 提取器与权限校验助手

use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use loco_rs::prelude::*;

use crate::{controllers::v1::auth::AuthSession, errors::AppError, models::user};

/// 管理员用户提取器 (Admin User Extractor)
///
/// 从 AuthSession 提取已认证会话，并校验 `user.is_admin()`，非管理员拒绝访问并返回 403 Forbidden。
#[derive(Debug, Clone)]
pub struct AdminUser(pub user::Model);

impl<S> FromRequestParts<S> for AdminUser
where
    S: Send + Sync,
    AppContext: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth = AuthSession::from_request_parts(parts, state).await?;

        if !auth.user.is_admin() {
            return Err(AppError::forbidden("Access denied: admin role required"));
        }

        Ok(Self(auth.user))
    }
}

/// 常规已登录用户提取器 (支持自动获取 user 模型)
#[derive(Debug, Clone)]
pub struct AuthenticatedUser(pub user::Model);

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
    AppContext: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth = AuthSession::from_request_parts(parts, state).await?;
        Ok(Self(auth.user))
    }
}
