//! RBAC (Role-Based Access Control) 提取器与权限校验助手

use axum::{
    extract::{FromRef, FromRequestParts},
    http::{StatusCode, request::Parts},
};
use loco_rs::prelude::*;

use crate::{errors::AppError, models::users};

/// 管理员用户提取器 (Admin User Extractor)
///
/// 基于 Loco 原生 [`auth::JWTWithUser`] 适配器进行鉴权与用户装载，
/// 并校验 `user.is_admin()`，非管理员拒绝访问并返回 403 Forbidden。
#[derive(Debug, Clone)]
pub struct AdminUser(pub users::Model);

impl<S> FromRequestParts<S> for AdminUser
where
    S: Send + Sync,
    AppContext: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth = auth::JWTWithUser::<users::Model>::from_request_parts(parts, state)
            .await
            .map_err(|e| AppError::Custom(StatusCode::UNAUTHORIZED, e.to_string()))?;

        if !auth.user.is_admin() {
            return Err(AppError::Custom(
                StatusCode::FORBIDDEN,
                "Access denied: admin role required".to_string(),
            ));
        }

        Ok(Self(auth.user))
    }
}

/// 常规已登录用户提取器 (支持自动获取 user 模型)
#[derive(Debug, Clone)]
pub struct AuthenticatedUser(pub users::Model);

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
    AppContext: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth = auth::JWTWithUser::<users::Model>::from_request_parts(parts, state)
            .await
            .map_err(|e| AppError::Custom(StatusCode::UNAUTHORIZED, e.to_string()))?;

        Ok(Self(auth.user))
    }
}
