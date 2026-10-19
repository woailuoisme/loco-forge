//! # 全局统一异常处理模块 (Unified Application Error System)
//!
//! 提供全项目通用的 `AppError` 枚举与 `AppResult<T>` 类型别名。
//! 自动实现 `axum::response::IntoResponse`，统一输出标准 JSON 格式 [`crate::dtos::common::ApiError`]。
//! 遵循纯 3 位 HTTP 状态码标准（如 401, 403, 404, 500）。
//!
//! ## JSON 响应契约示例 (JSON Response Example)
//! ```json
//! {
//!   "code": "AUTH_TOKEN_INVALID",
//!   "message": "Token 已过期"
//! }
//! ```

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

use crate::dtos::common::{ApiError, error_code_for};

/// 全局统一应用异常枚举
///
/// 支持标准 HTTP 状态码异常 (`Custom`)、显式错误码异常 (`Coded`)、
/// 600~999 业务逻辑错误 (`Biz`) 以及底层系统/数据库错误包装 (`Internal`)。
#[derive(Debug, Error)]
pub enum AppError {
    /// 3 位标准 HTTP 状态码异常 (如 StatusCode::UNAUTHORIZED, StatusCode::FORBIDDEN)
    #[error("{1}")]
    Custom(StatusCode, String),

    /// 携带显式错误码的异常 (如 `AUTH_INVALID_CREDENTIALS`、`AUTH_OTP_EXPIRED`)
    #[error("{2}")]
    Coded(StatusCode, &'static str, String),

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
            Self::Custom(status, _) | Self::Coded(status, _, _) => *status,
            Self::Biz(_, _) => StatusCode::BAD_REQUEST,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// 获取标准错误码数字 (3位 HTTP 状态码数字或 600~999 业务错误码)
    pub fn error_code(&self) -> u32 {
        match self {
            Self::Custom(status, _) | Self::Coded(status, _, _) => u32::from(status.as_u16()),
            Self::Biz(code, _) => *code,
            Self::Internal(_) => u32::from(StatusCode::INTERNAL_SERVER_ERROR.as_u16()),
        }
    }

    /// 提取异常的真实原始消息
    pub fn message(&self) -> &str {
        match self {
            Self::Custom(_, msg)
            | Self::Coded(_, _, msg)
            | Self::Biz(_, msg)
            | Self::Internal(msg) => msg.as_str(),
        }
    }

    /// 获取 Wire 层错误码字符串 (api.md §5.2 词表)
    #[must_use]
    pub fn code(&self) -> String {
        match self {
            Self::Coded(_, code, _) => (*code).to_string(),
            Self::Biz(code, _) => format!("BIZ_{code}"),
            _ => error_code_for(self.status_code()).to_string(),
        }
    }

    /// 构造 401 Unauthorized 异常
    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self::Custom(StatusCode::UNAUTHORIZED, msg.into())
    }

    /// 构造 404 Not Found 异常
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::Custom(StatusCode::NOT_FOUND, msg.into())
    }

    /// 构造 400 Bad Request 异常
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::Custom(StatusCode::BAD_REQUEST, msg.into())
    }

    /// 构造 403 Forbidden 异常
    pub fn forbidden(msg: impl Into<String>) -> Self {
        Self::Custom(StatusCode::FORBIDDEN, msg.into())
    }

    /// 构造 409 Conflict 异常
    pub fn conflict(msg: impl Into<String>) -> Self {
        Self::Custom(StatusCode::CONFLICT, msg.into())
    }

    /// 构造 500 Internal Server Error 异常
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }

    /// 构造带显式错误码的异常 (`code` 取自 api.md §5.2 词表)
    pub fn with_code(status: StatusCode, code: &'static str, msg: impl Into<String>) -> Self {
        Self::Coded(status, code, msg.into())
    }

    /// 判定错误是否为唯一约束冲突（并发下唯一索引兜底，避免 500）
    #[must_use]
    pub fn is_unique_violation(err: impl std::fmt::Display) -> bool {
        let msg = err.to_string();
        msg.contains("duplicate key") || msg.contains("unique constraint")
    }

    /// 401 邮箱/用户名或口令不正确 (防枚举)
    pub fn invalid_credentials(msg: impl Into<String>) -> Self {
        Self::with_code(
            StatusCode::UNAUTHORIZED,
            "AUTH_INVALID_CREDENTIALS",
            msg.into(),
        )
    }

    /// 401 访问令牌已过期
    pub fn token_expired(msg: impl Into<String>) -> Self {
        Self::with_code(StatusCode::UNAUTHORIZED, "AUTH_TOKEN_EXPIRED", msg.into())
    }

    /// 401 令牌已被撤销
    pub fn token_revoked(msg: impl Into<String>) -> Self {
        Self::with_code(StatusCode::UNAUTHORIZED, "AUTH_TOKEN_REVOKED", msg.into())
    }

    /// 403 账号已被封禁
    pub fn account_banned(msg: impl Into<String>) -> Self {
        Self::with_code(StatusCode::FORBIDDEN, "AUTH_ACCOUNT_BANNED", msg.into())
    }

    /// 409 邮箱已被占用
    pub fn email_exists(msg: impl Into<String>) -> Self {
        Self::with_code(
            StatusCode::CONFLICT,
            "AUTH_EMAIL_ALREADY_EXISTS",
            msg.into(),
        )
    }

    /// 422 验证码不正确
    pub fn otp_invalid(msg: impl Into<String>) -> Self {
        Self::with_code(
            StatusCode::UNPROCESSABLE_ENTITY,
            "AUTH_OTP_INVALID",
            msg.into(),
        )
    }

    /// 422 验证码已过期或已被使用
    pub fn otp_expired(msg: impl Into<String>) -> Self {
        Self::with_code(
            StatusCode::UNPROCESSABLE_ENTITY,
            "AUTH_OTP_EXPIRED",
            msg.into(),
        )
    }

    /// 429 请求凭据过于频繁
    pub fn rate_limited(msg: impl Into<String>) -> Self {
        Self::with_code(StatusCode::TOO_MANY_REQUESTS, "RATE_LIMITED", msg.into())
    }

    /// 构造 600~999 业务异常
    pub fn biz(code: u32, msg: impl Into<String>) -> Self {
        Self::Biz(code, msg.into())
    }
}

impl From<loco_rs::Error> for AppError {
    fn from(err: loco_rs::Error) -> Self {
        match err {
            loco_rs::Error::NotFound => {
                Self::Custom(StatusCode::NOT_FOUND, "Resource not found".to_string())
            },
            loco_rs::Error::Unauthorized(msg) => Self::Custom(StatusCode::UNAUTHORIZED, msg),
            loco_rs::Error::BadRequest(msg) => Self::Custom(StatusCode::BAD_REQUEST, msg),
            loco_rs::Error::Model(model_err) => Self::from(model_err),
            loco_rs::Error::CustomError(status, data) => {
                let msg = data
                    .error
                    .or(data.description)
                    .unwrap_or_else(|| "Custom error".to_string());
                Self::Custom(status, msg)
            },
            _ => Self::Internal(err.to_string()),
        }
    }
}

impl From<loco_rs::model::ModelError> for AppError {
    fn from(err: loco_rs::model::ModelError) -> Self {
        match err {
            loco_rs::model::ModelError::EntityNotFound => {
                Self::Custom(StatusCode::NOT_FOUND, "Entity not found".to_string())
            },
            loco_rs::model::ModelError::EntityAlreadyExists => {
                Self::Custom(StatusCode::CONFLICT, "Entity already exists".to_string())
            },
            loco_rs::model::ModelError::Validation(v) => {
                Self::Custom(StatusCode::UNPROCESSABLE_ENTITY, v.to_string())
            },
            _ => Self::Internal(err.to_string()),
        }
    }
}

impl From<AppError> for loco_rs::Error {
    fn from(err: AppError) -> Self {
        Self::string(&err.to_string())
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
            crate::common::media::error::MediaError::InvalidInput(msg) => {
                Self::Custom(StatusCode::BAD_REQUEST, msg)
            },
            _ => Self::Internal(err.to_string()),
        }
    }
}

/// 实现 Axum `IntoResponse` Trait，将 AppError 统一渲染为标准 JSON [`ApiError`]
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();

        // 500 生产脱敏 (api.md §5.2)：原始异常消息只写入服务端日志，
        // 对外统一返回通用提示，杜绝泄漏数据库错误、异常名与源码路径。
        if status == StatusCode::INTERNAL_SERVER_ERROR {
            tracing::error!(detail = %self.message(), "服务器内部故障");
            return (
                status,
                Json(ApiError::new(
                    "INTERNAL_SERVER_ERROR",
                    "An unexpected error occurred",
                )),
            )
                .into_response();
        }

        let code = self.code();
        (status, Json(ApiError::new(code, self.message()))).into_response()
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

    #[test]
    fn test_wire_codes_follow_api_md_vocabulary() {
        // 显式错误码优先于状态码默认映射
        assert_eq!(
            AppError::invalid_credentials("bad password").code(),
            "AUTH_INVALID_CREDENTIALS"
        );
        assert_eq!(
            AppError::token_expired("expired").code(),
            "AUTH_TOKEN_EXPIRED"
        );
        assert_eq!(
            AppError::token_revoked("revoked").code(),
            "AUTH_TOKEN_REVOKED"
        );
        assert_eq!(
            AppError::account_banned("banned").code(),
            "AUTH_ACCOUNT_BANNED"
        );
        assert_eq!(
            AppError::email_exists("dup").code(),
            "AUTH_EMAIL_ALREADY_EXISTS"
        );
        assert_eq!(AppError::otp_invalid("wrong").code(), "AUTH_OTP_INVALID");
        assert_eq!(AppError::otp_expired("stale").code(), "AUTH_OTP_EXPIRED");
        assert_eq!(AppError::rate_limited("slow down").code(), "RATE_LIMITED");

        // 无显式码时按状态码兜底
        assert_eq!(
            AppError::Custom(StatusCode::UNAUTHORIZED, "x".into()).code(),
            "AUTH_TOKEN_INVALID"
        );
        assert_eq!(AppError::Biz(601, "x".into()).code(), "BIZ_601");
        assert_eq!(
            AppError::Internal("x".into()).code(),
            "INTERNAL_SERVER_ERROR"
        );
    }

    #[test]
    fn test_error_statuses_match_documented_codes() {
        for err in [
            AppError::invalid_credentials("x"),
            AppError::token_expired("x"),
            AppError::account_banned("x"),
            AppError::email_exists("x"),
            AppError::otp_invalid("x"),
            AppError::rate_limited("x"),
        ] {
            assert_eq!(
                crate::dtos::common::status_for_code(&err.code()),
                err.status_code(),
                "code {} 与状态码不一致",
                err.code()
            );
        }
    }
}
