//! 统一 API 错误响应结构体 (Unified API Error Response)
//!
//! 遵循 `api.md` 第 5 章响应契约：
//!
//! - **分页列表**：`{"items": [...], "meta": {...}}` (见 [`super::pagination`])
//! - **其余成功响应**：直接返回裸 JSON 对象，**不包** `success` / `code` / `message` 外壳
//! - **所有错误**：统一收敛于扁平的 `{"code", "message", "errors"?}`
//!
//! [`ApiError`] 的 `code` 取值与 `api.md` §5.2 错误码清单一一对应，
//! 客户端据 `code` 分支处理，`message` 仅供展示。

use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// 统一 API JSON 错误响应结构体 (遵循 api.md 5.2 节)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ApiError {
    /// 机器可读错误代码 (如 `AUTH_TOKEN_INVALID`、`VALIDATION_FAILED`)
    #[schema(example = "AUTH_TOKEN_INVALID")]
    pub code: String,
    /// 人类可读的错误描述
    #[schema(example = "访问令牌无效")]
    pub message: String,
    /// 字段级校验失败归集表 (仅 422 `VALIDATION_FAILED` 出现)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<serde_json::Value>,
}

/// 将 HTTP 状态码映射为默认的机器可读错误代码。
///
/// 仅用于没有更具体语义的兜底场景；能确定语义的调用点应直接使用
/// [`ApiError`] 的具名构造函数。
#[must_use]
pub fn error_code_for(status: StatusCode) -> &'static str {
    match status {
        StatusCode::BAD_REQUEST => "BAD_REQUEST",
        StatusCode::UNAUTHORIZED => "AUTH_TOKEN_INVALID",
        StatusCode::FORBIDDEN => "FORBIDDEN",
        StatusCode::NOT_FOUND => "NOT_FOUND",
        StatusCode::METHOD_NOT_ALLOWED => "METHOD_NOT_ALLOWED",
        StatusCode::CONFLICT => "CONFLICT",
        StatusCode::PAYLOAD_TOO_LARGE => "REQUEST_TOO_LARGE",
        StatusCode::UNPROCESSABLE_ENTITY => "VALIDATION_FAILED",
        StatusCode::TOO_MANY_REQUESTS => "RATE_LIMITED",
        StatusCode::INTERNAL_SERVER_ERROR => "INTERNAL_SERVER_ERROR",
        StatusCode::SERVICE_UNAVAILABLE => "SERVICE_UNAVAILABLE",
        StatusCode::GATEWAY_TIMEOUT => "GATEWAY_TIMEOUT",
        _ => "UNKNOWN_ERROR",
    }
}

/// 将错误代码反查为 HTTP 状态码（与 [`error_code_for`] 双向一致）。
#[must_use]
pub fn status_for_code(code: &str) -> StatusCode {
    match code {
        "BAD_REQUEST" | "IDEMPOTENCY_REQUIRED" | "EXCEL_OPEN_ERROR" | "EXCEL_READ_ERROR" => {
            StatusCode::BAD_REQUEST
        },
        "AUTH_TOKEN_INVALID"
        | "AUTH_TOKEN_EXPIRED"
        | "AUTH_TOKEN_REVOKED"
        | "AUTH_INVALID_CREDENTIALS" => StatusCode::UNAUTHORIZED,
        "FORBIDDEN" | "AUTH_ACCOUNT_BANNED" | "AUTH_EMAIL_NOT_VERIFIED" => StatusCode::FORBIDDEN,
        "NOT_FOUND" => StatusCode::NOT_FOUND,
        "METHOD_NOT_ALLOWED" => StatusCode::METHOD_NOT_ALLOWED,
        "CONFLICT"
        | "AUTH_EMAIL_ALREADY_EXISTS"
        | "AUTH_USERNAME_TAKEN"
        | "AUTH_PHONE_NUMBER_TAKEN"
        | "IDEMPOTENCY_CONFLICT" => StatusCode::CONFLICT,
        "REQUEST_TOO_LARGE" | "EXCEL_FILE_TOO_LARGE" => StatusCode::PAYLOAD_TOO_LARGE,
        "VALIDATION_FAILED"
        | "AUTH_OTP_INVALID"
        | "AUTH_OTP_EXPIRED"
        | "AUTH_PASSWORD_MISMATCH"
        | "AUTH_PASSWORD_TOO_WEAK"
        | "IDEMPOTENCY_MISMATCH"
        | "EXCEL_MAP_ERROR"
        | "EXCEL_ROWS_ERROR" => StatusCode::UNPROCESSABLE_ENTITY,
        "RATE_LIMITED" | "AUTH_OTP_TOO_MANY_ATTEMPTS" | "AUTH_OTP_RATE_LIMITED" => {
            StatusCode::TOO_MANY_REQUESTS
        },
        "INTERNAL_SERVER_ERROR" => StatusCode::INTERNAL_SERVER_ERROR,
        "SERVICE_UNAVAILABLE" => StatusCode::SERVICE_UNAVAILABLE,
        "GATEWAY_TIMEOUT" => StatusCode::GATEWAY_TIMEOUT,
        // 600~999 业务错误码统一以 HTTP 400 下发
        c if c.starts_with("BIZ_") => StatusCode::BAD_REQUEST,
        _ => StatusCode::BAD_REQUEST,
    }
}

impl ApiError {
    /// 创建标准错误响应体
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            errors: None,
        }
    }

    /// 创建带字段错误归集的 422 响应体
    pub fn with_errors(
        code: impl Into<String>,
        message: impl Into<String>,
        errors: Option<serde_json::Value>,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            errors,
        }
    }

    /// 401 令牌无效
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new("AUTH_TOKEN_INVALID", message)
    }

    /// 401 邮箱/用户名或口令不正确（防枚举，账号不存在返回同一码）
    pub fn invalid_credentials(message: impl Into<String>) -> Self {
        Self::new("AUTH_INVALID_CREDENTIALS", message)
    }

    /// 400 请求非法
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new("BAD_REQUEST", message)
    }

    /// 404 资源不存在
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new("NOT_FOUND", message)
    }

    /// 403 无权限
    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::new("FORBIDDEN", message)
    }

    /// 409 资源冲突
    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new("CONFLICT", message)
    }

    /// 422 字段校验失败
    pub fn validation(message: impl Into<String>, errors: Option<serde_json::Value>) -> Self {
        Self::with_errors("VALIDATION_FAILED", message, errors)
    }

    /// 429 触发限流
    pub fn rate_limited(message: impl Into<String>) -> Self {
        Self::new("RATE_LIMITED", message)
    }

    /// 500 内部故障
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new("INTERNAL_SERVER_ERROR", message)
    }
}

impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let status = status_for_code(&self.code);
        (status, axum::Json(self)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_status_roundtrip() {
        for status in [
            StatusCode::BAD_REQUEST,
            StatusCode::UNAUTHORIZED,
            StatusCode::FORBIDDEN,
            StatusCode::NOT_FOUND,
            StatusCode::CONFLICT,
            StatusCode::PAYLOAD_TOO_LARGE,
            StatusCode::UNPROCESSABLE_ENTITY,
            StatusCode::TOO_MANY_REQUESTS,
            StatusCode::INTERNAL_SERVER_ERROR,
            StatusCode::SERVICE_UNAVAILABLE,
            StatusCode::GATEWAY_TIMEOUT,
        ] {
            let code = error_code_for(status);
            assert_eq!(
                status_for_code(code),
                status,
                "code {code} 反查状态码不一致"
            );
        }
    }

    #[test]
    fn test_error_body_is_flat() {
        let body = serde_json::to_value(ApiError::validation("Validation failed", None)).unwrap();
        assert_eq!(body["code"], "VALIDATION_FAILED");
        assert_eq!(body["message"], "Validation failed");
        assert!(body.get("error").is_none(), "错误体不应再包 error 外壳");
        assert!(body.get("errors").is_none(), "无字段错误时应省略 errors");
    }

    #[test]
    fn test_biz_code_maps_to_bad_request() {
        assert_eq!(status_for_code("BIZ_601"), StatusCode::BAD_REQUEST);
        assert_eq!(
            status_for_code("AUTH_OTP_EXPIRED"),
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }
}
