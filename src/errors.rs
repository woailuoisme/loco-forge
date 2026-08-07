//! # 全局统一异常处理模块 (Unified Application Error System)
//!
//! 提供全项目通用的 `AppError` 枚举与 `AppResult<T>` 类型别名。
//! 自动实现 `axum::response::IntoResponse`，统一输出标准 JSON 格式 [`crate::dtos::common::ApiError`]。
//! 遵循纯 3 位 HTTP 状态码标准（如 401, 403, 404, 500）。
//!
//! ## JSON 响应契约示例 (JSON Response Example)
//! ```json
//! {
//!   "success": false,
//!   "code": 401,
//!   "message": "Token 已过期",
//!   "errors": null
//! }
//! ```

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

use crate::dtos::common::ApiError;

/// 全局统一应用异常枚举
///
/// 设计原则：最小化包装，优先使用 `loco_rs::Error` 通过 `?` 传播。
/// 仅在需要显式返回非 200 HTTP 状态码 JSON 错误时使用 `AppError::Custom`。
#[derive(Debug, Error)]
pub enum AppError {
    /// 3 位标准 HTTP 状态码异常 (如 StatusCode::UNAUTHORIZED, StatusCode::FORBIDDEN)
    #[error("HTTP error ({0}): {1}")]
    Custom(StatusCode, String),
}

impl AppError {
    /// 获取对应的 HTTP 状态码
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::Custom(status, _) => *status,
        }
    }

    /// 获取标准错误码数字 (3位 HTTP 状态码数字)
    pub fn error_code(&self) -> u32 {
        match self {
            Self::Custom(status, _) => status.as_u16() as u32,
        }
    }

    /// 提取错误消息
    pub fn message(&self) -> String {
        match self {
            Self::Custom(_, msg) => msg.clone(),
        }
    }
}

/// 实现 Axum `IntoResponse` Trait，将 AppError 统一渲染为标准 JSON [`ApiError`]
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();

        // 仅在开发/测试环境 (debug_assertions) 下收集 exception 信息
        let exception = if cfg!(debug_assertions) {
            Some("AppError::Custom".to_string())
        } else {
            None
        };

        let api_error = ApiError::error(self.error_code(), self.message(), None, exception, None);

        (status, Json(api_error)).into_response()
    }
}

/// 全局应用 Result 简化类型别名
pub type AppResult<T> = std::result::Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_error_status_codes() {
        let err = AppError::Custom(StatusCode::UNAUTHORIZED, "expired token".to_string());
        assert_eq!(err.status_code(), StatusCode::UNAUTHORIZED);
        assert_eq!(err.error_code(), 401);
        assert_eq!(err.message(), "expired token");

        let err = AppError::Custom(StatusCode::FORBIDDEN, "access denied".to_string());
        assert_eq!(err.status_code(), StatusCode::FORBIDDEN);
        assert_eq!(err.error_code(), 403);
    }
}
