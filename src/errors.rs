//! # 全局统一异常处理模块 (Unified Application Error System)
//!
//! 提供全项目通用的 `AppError` 枚举与 `AppResult<T>` 类型别名。
//! 自动实现 `axum::response::IntoResponse`，统一输出标准 JSON 格式 [`crate::dtos::common::ApiError`]。
//!
//! ## JSON 响应契约示例 (JSON Response Examples)
//!
//! ### 1. 404 / 400 / 401 普通业务异常 (默认使用 HTTP 状态码数字)
//! ```json
//! {
//!   "code": 404,
//!   "message": "Resource not found: User with pid_123 does not exist",
//!   "details": null
//! }
//! ```
//!
//! ### 2. 422 参数校验错误 / 细粒度自定义数字错误码 (如 400001)
//! ```json
//! {
//!   "code": 422,
//!   "message": "Invalid request payload parameters",
//!   "details": {
//!     "email": ["email format is invalid"],
//!     "password": ["password must be at least 8 characters"]
//!   }
//! }
//! ```
//!
//! ### 3. 500 未知系统内部错误 (自动敏感信息脱敏)
//! ```json
//! {
//!   "code": 500,
//!   "message": "An unexpected internal server error occurred",
//!   "details": null
//! }
//! ```

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use loco_rs::Error as LocoError;
use sea_orm::DbErr;
use thiserror::Error;

use crate::dtos::common::ApiError;

/// 全局统一应用异常枚举
#[derive(Debug, Error)]
pub enum AppError {
    /// 资源未找到 (默认错误码 404)
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// 请求参数不合法或无法处理 (默认错误码 400)
    #[error("Bad request: {0}")]
    BadRequest(String),

    /// 身份未认证 / Token 无效或过期 (默认错误码 401)
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    /// 权限不足被拒绝访问 (默认错误码 403)
    #[error("Forbidden: {0}")]
    Forbidden(String),

    /// 数据校验错误 (默认错误码 422)
    #[error("Validation error: {0}")]
    Validation(String, Option<serde_json::Value>),

    /// 业务/自定义特定数字错误码的异常 (如 400001, 10001 等)
    #[error("Custom error ({0}): {1}")]
    Custom(u32, String),

    /// 数据库操作失败 (500 Internal Server Error)
    #[error("Database error: {0}")]
    Db(#[from] DbErr),

    /// 底层 Loco 框架错误
    #[error("Loco framework error: {0}")]
    Loco(#[from] LocoError),

    /// 通用未捕获系统内部错误 (500 Internal Server Error)
    /// 自动从 eyre::Report 转换，保留完整的堆栈追踪 (Backtrace)
    #[error(transparent)]
    Internal(#[from] eyre::Report),
}

impl From<validator::ValidationErrors> for AppError {
    fn from(err: validator::ValidationErrors) -> Self {
        let details = serde_json::to_value(&err).ok();
        Self::Validation("Invalid request payload parameters".to_string(), details)
    }
}

impl AppError {
    /// 便捷构造函数：从任何消息描述创建 Internal 异常
    pub fn internal<M: std::fmt::Display>(msg: M) -> Self {
        Self::Internal(eyre::eyre!("{}", msg))
    }

    /// 获取对应的 HTTP 状态码
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::Validation(_, _) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::Custom(code, _) => parse_custom_status_code(*code),
            Self::Db(_) | Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Loco(err) => match err {
                LocoError::NotFound => StatusCode::NOT_FOUND,
                LocoError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
                LocoError::BadRequest(_) => StatusCode::BAD_REQUEST,
                LocoError::Validation(..) => StatusCode::UNPROCESSABLE_ENTITY,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            },
        }
    }

    /// 获取标准错误码数字
    /// 支持自定义数字错误码；若未指定则默认回退到 HTTP 状态码数字 (如 404, 400, 401, 403, 422, 500)
    pub fn error_code(&self) -> u32 {
        match self {
            Self::Custom(code, _) => *code,
            _ => self.status_code().as_u16() as u32,
        }
    }

    /// 提取详细错误描述消息 (内部未捕获错误自动脱敏，防止敏感数据泄露给客户端)
    pub fn message(&self) -> String {
        match self {
            Self::NotFound(msg)
            | Self::BadRequest(msg)
            | Self::Unauthorized(msg)
            | Self::Forbidden(msg)
            | Self::Validation(msg, _)
            | Self::Custom(_, msg) => msg.clone(),
            Self::Db(err) => format!("Database operation failed: {err}"),
            Self::Loco(err) => err.to_string(),
            Self::Internal(_) => "An unexpected internal server error occurred".to_string(),
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

/// 帮助解析自定义错误码对应的 HTTP 状态码
fn parse_custom_status_code(code: u32) -> StatusCode {
    u16::try_from(code)
        .ok()
        .and_then(|c| StatusCode::from_u16(c).ok())
        .unwrap_or_else(|| {
            if (100_000..=599_999).contains(&code) {
                u16::try_from(code / 1000)
                    .ok()
                    .and_then(|s| StatusCode::from_u16(s).ok())
                    .unwrap_or(StatusCode::BAD_REQUEST)
            } else {
                StatusCode::BAD_REQUEST
            }
        })
}

/// 实现 Axum `IntoResponse` Trait，将 AppError 统一渲染为标准 JSON [`ApiError`]
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();

        // 如果是 500 内部系统错误，记录 Error 级别的 Tracing 日志 (包含 eyre 错误链与堆栈) 以便追溯诊断
        if status.is_server_error() {
            match &self {
                Self::Internal(report) => {
                    tracing::error!(
                        target: "app",
                        error_chain = ?report,
                        "Internal server error occurred"
                    );
                }
                err => {
                    tracing::error!(target: "app", error = %err, "Internal server error occurred");
                }
            }
        }

        // 仅在开发/测试环境 (debug_assertions) 下收集 exception 与 trace 信息
        let (exception, trace) = if cfg!(debug_assertions) {
            let exception_name = match &self {
                Self::NotFound(_) => "AppError::NotFound",
                Self::BadRequest(_) => "AppError::BadRequest",
                Self::Unauthorized(_) => "AppError::Unauthorized",
                Self::Forbidden(_) => "AppError::Forbidden",
                Self::Validation(_, _) => "AppError::Validation",
                Self::Custom(_, _) => "AppError::Custom",
                Self::Db(_) => "AppError::Db",
                Self::Loco(_) => "AppError::Loco",
                Self::Internal(_) => "AppError::Internal",
            };
            let stacktrace = match &self {
                Self::Internal(report) => {
                    let bt = format!("{report:?}");
                    Some(bt.lines().map(String::from).collect())
                }
                _ => None,
            };
            (Some(exception_name.to_string()), stacktrace)
        } else {
            (None, None)
        };

        let api_error = ApiError::error(self.error_code(), self.message(), self.details(), exception, trace);

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
        assert_eq!(err.error_code(), 404);
        assert_eq!(err.message(), "item missing");

        let err = AppError::BadRequest("invalid query".to_string());
        assert_eq!(err.status_code(), StatusCode::BAD_REQUEST);
        assert_eq!(err.error_code(), 400);

        let err = AppError::Unauthorized("expired token".to_string());
        assert_eq!(err.status_code(), StatusCode::UNAUTHORIZED);
        assert_eq!(err.error_code(), 401);

        let err = AppError::Forbidden("access denied".to_string());
        assert_eq!(err.status_code(), StatusCode::FORBIDDEN);
        assert_eq!(err.error_code(), 403);

        let err = AppError::Custom(400001, "insufficient balance".to_string());
        assert_eq!(err.status_code(), StatusCode::BAD_REQUEST);
        assert_eq!(err.error_code(), 400001);
        assert_eq!(err.message(), "insufficient balance");

        let err = AppError::internal("db crash");
        assert_eq!(err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(err.error_code(), 500);
        assert_eq!(err.message(), "An unexpected internal server error occurred");
    }

    #[test]
    fn test_custom_status_code_parsing() {
        // 直接使用 HTTP 状态码作为错误码
        assert_eq!(parse_custom_status_code(404), StatusCode::NOT_FOUND);
        assert_eq!(parse_custom_status_code(400), StatusCode::BAD_REQUEST);

        // 6 位自定义错误码，从前 3 位推导 HTTP 状态码
        assert_eq!(parse_custom_status_code(404001), StatusCode::NOT_FOUND);
        assert_eq!(parse_custom_status_code(401002), StatusCode::UNAUTHORIZED);
        assert_eq!(parse_custom_status_code(422005), StatusCode::UNPROCESSABLE_ENTITY);

        // 无效错误码默认回退为 400 BAD_REQUEST
        assert_eq!(parse_custom_status_code(999999), StatusCode::BAD_REQUEST);
        assert_eq!(parse_custom_status_code(99), StatusCode::BAD_REQUEST);
    }
}
