//! # 全局统一异常处理模块 (Unified Application Error System)
//!
//! 提供全项目通用的 `AppError` 枚举与 `AppResult<T>` 类型别名。
//! 自动实现 `axum::response::IntoResponse`，统一输出标准 JSON 格式 [`crate::dtos::common::ApiError`]。

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use loco_rs::Error as LocoError;
use sea_orm::DbErr;
use thiserror::Error;

use crate::dtos::common::ApiError;

/// 全局统一应用异常枚举
#[derive(Debug, Error)]
pub enum AppError {
    /// 资源未找到 (404 Not Found)
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// 请求参数不合法或无法处理 (400 Bad Request)
    #[error("Bad request: {0}")]
    BadRequest(String),

    /// 身份未认证 / Token 无效或过期 (401 Unauthorized)
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    /// 权限不足被拒绝访问 (403 Forbidden)
    #[error("Forbidden: {0}")]
    Forbidden(String),

    /// 数据校验错误 (422 Unprocessable Entity)
    #[error("Validation error: {0}")]
    Validation(String, Option<serde_json::Value>),

    /// 数据库操作失败 (500 Internal Server Error)
    #[error("Database error: {0}")]
    Db(#[from] DbErr),

    /// 底层 Loco 框架错误
    #[error("Loco framework error: {0}")]
    Loco(#[from] LocoError),

    /// 通用未捕获系统内部错误 (500 Internal Server Error)
    #[error("Internal server error: {0}")]
    Internal(String),
}

impl AppError {
    /// 获取对应的 HTTP 状态码
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::Validation(_, _) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::Db(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Loco(err) => match err {
                LocoError::NotFound => StatusCode::NOT_FOUND,
                LocoError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
                LocoError::BadRequest(_) => StatusCode::BAD_REQUEST,
                LocoError::Validation(..) => StatusCode::UNPROCESSABLE_ENTITY,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            },
        }
    }

    /// 获取标准错误码标识 (如 "not_found", "unauthorized", "internal_server_error")
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "not_found",
            Self::BadRequest(_) => "bad_request",
            Self::Unauthorized(_) => "unauthorized",
            Self::Forbidden(_) => "forbidden",
            Self::Validation(_, _) => "validation_error",
            Self::Db(_) => "database_error",
            Self::Internal(_) => "internal_server_error",
            Self::Loco(err) => match err {
                LocoError::NotFound => "not_found",
                LocoError::Unauthorized(_) => "unauthorized",
                LocoError::BadRequest(_) => "bad_request",
                LocoError::Validation(..) => "validation_error",
                _ => "internal_server_error",
            },
        }
    }

    /// 提取详细错误描述消息
    pub fn message(&self) -> String {
        match self {
            Self::NotFound(msg)
            | Self::BadRequest(msg)
            | Self::Unauthorized(msg)
            | Self::Forbidden(msg)
            | Self::Internal(msg) => msg.clone(),
            Self::Validation(msg, _) => msg.clone(),
            Self::Db(err) => format!("Database operation failed: {}", err),
            Self::Loco(err) => err.to_string(),
        }
    }

    /// 提取可选的附加细节 (serde_json::Value)
    pub fn details(&self) -> Option<serde_json::Value> {
        match self {
            Self::Validation(_, details) => details.clone(),
            _ => None,
        }
    }
}

/// 实现 Axum `IntoResponse` Trait，将 AppError 统一渲染为标准 JSON [`ApiError`]
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let api_error = ApiError {
            code: self.error_code().to_string(),
            message: self.message(),
            details: self.details(),
        };

        // 如果是 500 内部系统错误，记录 Error 级别的 Tracing 日志以便追溯诊断
        if status.is_server_error() {
            tracing::error!(target: "app", error = %self, "Internal server error occurred");
        }

        (status, Json(api_error)).into_response()
    }
}

/// 全局应用 Result 简化类型别名
pub type AppResult<T> = std::result::Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_status_codes_and_codes() {
        let err = AppError::NotFound("item missing".to_string());
        assert_eq!(err.status_code(), StatusCode::NOT_FOUND);
        assert_eq!(err.error_code(), "not_found");
        assert_eq!(err.message(), "item missing");

        let err = AppError::BadRequest("invalid query".to_string());
        assert_eq!(err.status_code(), StatusCode::BAD_REQUEST);
        assert_eq!(err.error_code(), "bad_request");

        let err = AppError::Unauthorized("expired token".to_string());
        assert_eq!(err.status_code(), StatusCode::UNAUTHORIZED);
        assert_eq!(err.error_code(), "unauthorized");

        let err = AppError::Forbidden("access denied".to_string());
        assert_eq!(err.status_code(), StatusCode::FORBIDDEN);
        assert_eq!(err.error_code(), "forbidden");

        let err = AppError::Internal("db crash".to_string());
        assert_eq!(err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(err.error_code(), "internal_server_error");
    }
}
