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
/// 支持标准 HTTP 状态码异常 (`Custom`)、600~999 业务逻辑错误 (`Biz`) 以及底层系统/数据库错误包装 (`Internal`)。
#[derive(Debug, Error)]
pub enum AppError {
    /// 3 位标准 HTTP 状态码异常 (如 StatusCode::UNAUTHORIZED, StatusCode::FORBIDDEN)
    #[error("{1}")]
    Custom(StatusCode, String),

    /// 600 ~ 999 业务逻辑错误 (默认 Wire 状态码 HTTP 400 Bad Request)
    #[error("{1}")]
    Biz(u32, String),

    /// 内部故障 / 系统异常 (HTTP 500 Internal Server Error)
    #[error("{0}")]
    Internal(String),
}

impl AppError {
    /// 获取对应的 HTTP 状态码
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::Custom(status, _) => *status,
            Self::Biz(_, _) => StatusCode::BAD_REQUEST,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// 获取标准错误码数字 (3位 HTTP 状态码数字或 600~999 业务错误码)
    pub fn error_code(&self) -> u32 {
        match self {
            Self::Custom(status, _) => status.as_u16() as u32,
            Self::Biz(code, _) => *code,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR.as_u16() as u32,
        }
    }

    /// 提取异常的真实原始消息
    pub fn message(&self) -> &str {
        match self {
            Self::Custom(_, msg) => msg.as_str(),
            Self::Biz(_, msg) => msg.as_str(),
            Self::Internal(msg) => msg.as_str(),
        }
    }
}

impl From<loco_rs::Error> for AppError {
    fn from(err: loco_rs::Error) -> Self {
        Self::Internal(err.to_string())
    }
}

impl From<sea_orm::DbErr> for AppError {
    fn from(err: sea_orm::DbErr) -> Self {
        Self::Internal(err.to_string())
    }
}

impl From<crate::common::media::error::MediaError> for AppError {
    fn from(err: crate::common::media::error::MediaError) -> Self {
        match err {
            crate::common::media::error::MediaError::InvalidInput(msg) => Self::Custom(StatusCode::BAD_REQUEST, msg),
            _ => Self::Internal(err.to_string()),
        }
    }
}

/// 实现 Axum `IntoResponse` Trait，将 AppError 统一渲染为标准 JSON [`ApiError`]
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let code = self.error_code();
        let msg = self.message().to_string();

        let trace = if status.is_server_error() && cfg!(debug_assertions) {
            Some(vec![format!("AppError Exception: {msg}")])
        } else {
            None
        };

        let api_error = ApiError::error_with_trace(code, msg, None, trace);
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

    #[test]
    fn test_biz_and_internal_errors() {
        let biz_err = AppError::Biz(601, "账号冻结".to_string());
        assert_eq!(biz_err.status_code(), StatusCode::BAD_REQUEST);
        assert_eq!(biz_err.error_code(), 601);
        assert_eq!(biz_err.message(), "账号冻结");

        let sys_err = AppError::Internal("Database connection reset".to_string());
        assert_eq!(sys_err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(sys_err.error_code(), 500);
        assert_eq!(sys_err.message(), "Database connection reset");
    }
}
