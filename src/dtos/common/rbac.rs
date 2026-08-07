//! RBAC (Role-Based Access Control) 提取器与权限校验助手

use axum::{
    extract::{FromRef, FromRequestParts},
    http::{StatusCode, request::Parts},
};
use loco_rs::prelude::*;

use crate::{errors::AppError, models::users};

/// 管理员用户提取器 (Admin User Extractor)
///
/// 提取逻辑：
/// 1. 提取并校验客户端请求中的 JWT；
/// 2. 从数据库查找用户 Model；
/// 3. 校验 `user.is_admin()`，非管理员抛出 403 Forbidden 异常。
#[derive(Debug, Clone)]
pub struct AdminUser(pub users::Model);

impl<S> FromRequestParts<S> for AdminUser
where
    S: Send + Sync,
    AppContext: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth = auth::JWT::from_request_parts(parts, state)
            .await
            .map_err(|e| AppError::Custom(StatusCode::UNAUTHORIZED, e.to_string()))?;
        let ctx = AppContext::from_ref(state);
        let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid)
            .await
            .map_err(|_| AppError::Custom(StatusCode::UNAUTHORIZED, "User not found".to_string()))?;

        if !user.is_admin() {
            return Err(AppError::Custom(
                StatusCode::FORBIDDEN,
                "Access denied: admin role required".to_string(),
            ));
        }

        Ok(AdminUser(user))
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
        let auth = auth::JWT::from_request_parts(parts, state)
            .await
            .map_err(|e| AppError::Custom(StatusCode::UNAUTHORIZED, e.to_string()))?;
        let ctx = AppContext::from_ref(state);
        let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid)
            .await
            .map_err(|_| AppError::Custom(StatusCode::UNAUTHORIZED, "User not found".to_string()))?;

        Ok(AuthenticatedUser(user))
    }
}
