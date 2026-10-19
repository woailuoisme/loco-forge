//! Controller 响应快速格式化辅助模块 (Response Formatting Helpers)
//!
//! 提供一系列将业务数据或错误包装为标准 Axum `Result<Response>` 的函数，
//! 严格遵循 `api.md` 第 5 章响应契约：
//!
//! - **成功响应**：直接返回裸 JSON 对象 (仅分页列表使用 `{items, meta}` 信封)；
//! - **错误响应**：统一收敛于 `{"error": {"code", "message", "fields"}}`。
//!
//! # 示例
//! ```rust,ignore
//! use crate::dtos::common::format_res;
//!
//! // 成功响应 (裸 JSON)
//! format_res::ok(user_dto)
//! format_res::created(new_entity)
//! format_res::page(items, 1, 20, 100)
//!
//! // 错误响应 (嵌套 error 信封)
//! format_res::not_found("用户不存在")
//! format_res::validation_error_with_details("校验失败", field_errors)
//! format_res::biz(ErrorCode::USER_FROZEN, "账号已被冻结")
//! ```

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use ts_rs::TS;

use super::pagination::{PageMeta, PaginatedData};
use super::response::{ApiError, error_code_for};

// =========================================================================
// 内部通用辅助
// =========================================================================

/// 内部通用辅助：直接返回裸 JSON 成功响应
fn respond<T: Serialize>(status: StatusCode, data: T) -> loco_rs::prelude::Result<Response> {
    Ok((status, Json(data)).into_response())
}

/// 内部通用辅助：构建标准错误响应 `{"code", "message"}`
fn error_response(
    status: StatusCode,
    message: impl Into<String>,
) -> loco_rs::prelude::Result<Response> {
    let body = ApiError::new(error_code_for(status), message);
    Ok((status, Json(body)).into_response())
}

/// 内部通用辅助：构建带字段级 errors 的错误响应
fn error_response_with_details<D: Serialize>(
    status: StatusCode,
    message: impl Into<String>,
    details: D,
) -> loco_rs::prelude::Result<Response> {
    let fields = serde_json::to_value(details).ok();
    let body = ApiError::with_errors(error_code_for(status), message, fields);
    Ok((status, Json(body)).into_response())
}

/// 内部通用辅助：构建 600~999 业务错误码信封响应 (`code` 形如 `BIZ_601`)
fn biz_error_response(
    status: StatusCode,
    code: u32,
    message: impl Into<String>,
    fields: Option<serde_json::Value>,
) -> loco_rs::prelude::Result<Response> {
    let body = ApiError::with_errors(format!("BIZ_{code}"), message, fields);
    Ok((status, Json(body)).into_response())
}

// =========================================================================
// 2xx 成功类响应 (裸 JSON)
// =========================================================================

/// 格式化标准 200 成功响应，直接返回裸 Resource JSON
pub fn ok<T: Serialize>(data: T) -> loco_rs::prelude::Result<Response> {
    respond(StatusCode::OK, data)
}

/// 格式化 201 资源创建成功响应，直接返回裸 Resource JSON
pub fn created<T: Serialize>(data: T) -> loco_rs::prelude::Result<Response> {
    respond(StatusCode::CREATED, data)
}

/// 格式化 202 异步任务受理响应，直接返回裸 JSON (如 `{"taskId": ..., "status": "PENDING"}`)
pub fn accepted<T: Serialize>(data: T) -> loco_rs::prelude::Result<Response> {
    respond(StatusCode::ACCEPTED, data)
}

/// 格式化 204 No Content 空响应 (删除、登出、改密等无返回体操作)
pub fn no_content() -> loco_rs::prelude::Result<Response> {
    Ok(StatusCode::NO_CONTENT.into_response())
}

/// 格式化分页列表响应 `{"items": [...], "meta": {...}}` (全系统唯一分页信封)
pub fn page<T: Serialize + TS>(
    items: Vec<T>,
    page: u64,
    page_size: u64,
    total: u64,
) -> loco_rs::prelude::Result<Response> {
    respond(
        StatusCode::OK,
        PaginatedData {
            items,
            meta: PageMeta::new(page, page_size, total),
        },
    )
}

// =========================================================================
// 4xx 客户端错误响应
// =========================================================================

/// 格式化 400 Bad Request 客户端无效请求响应
pub fn bad_request(message: impl Into<String>) -> loco_rs::prelude::Result<Response> {
    error_response(StatusCode::BAD_REQUEST, message)
}

/// 格式化默认 400 Bad Request 客户端无效请求响应 (默认提示消息 "请求参数无效")
pub fn bad_request_default() -> loco_rs::prelude::Result<Response> {
    bad_request("请求参数无效")
}

/// 格式化带结构化细节的 400 Bad Request 响应
pub fn bad_request_with_details<D: Serialize>(
    message: impl Into<String>,
    details: D,
) -> loco_rs::prelude::Result<Response> {
    error_response_with_details(StatusCode::BAD_REQUEST, message, details)
}

/// 格式化 401 Unauthorized 未登录或 Token 校验失败响应
pub fn unauthorized(message: impl Into<String>) -> loco_rs::prelude::Result<Response> {
    error_response(StatusCode::UNAUTHORIZED, message)
}

/// 格式化 403 Forbidden 权限不足或操作被禁止响应
pub fn forbidden(message: impl Into<String>) -> loco_rs::prelude::Result<Response> {
    error_response(StatusCode::FORBIDDEN, message)
}

/// 格式化 404 Not Found 目标资源未找到响应
pub fn not_found(message: impl Into<String>) -> loco_rs::prelude::Result<Response> {
    error_response(StatusCode::NOT_FOUND, message)
}

/// 格式化 404 Not Found 路由/接口路径不存在响应
pub fn route_not_found() -> loco_rs::prelude::Result<Response> {
    not_found("请求的接口路径不存在")
}

/// 格式化 405 Method Not Allowed 请求方法不允许响应
pub fn method_not_allowed(message: impl Into<String>) -> loco_rs::prelude::Result<Response> {
    error_response(StatusCode::METHOD_NOT_ALLOWED, message)
}

/// 格式化 409 Conflict 资源状态冲突 (如唯一约束重复) 响应
pub fn conflict(message: impl Into<String>) -> loco_rs::prelude::Result<Response> {
    error_response(StatusCode::CONFLICT, message)
}

/// 格式化 422 Unprocessable Entity 数据校验错误响应
pub fn validation_error(message: impl Into<String>) -> loco_rs::prelude::Result<Response> {
    error_response(StatusCode::UNPROCESSABLE_ENTITY, message)
}

/// 格式化带详细字段错误映射的 422 Validation Error 响应
pub fn validation_error_with_details<D: Serialize>(
    message: impl Into<String>,
    details: D,
) -> loco_rs::prelude::Result<Response> {
    error_response_with_details(StatusCode::UNPROCESSABLE_ENTITY, message, details)
}

/// 格式化 429 Too Many Requests 请求频率限制响应
pub fn too_many_requests(message: impl Into<String>) -> loco_rs::prelude::Result<Response> {
    error_response(StatusCode::TOO_MANY_REQUESTS, message)
}

// =========================================================================
// 5xx 服务端错误响应
// =========================================================================

/// 格式化 500 Internal Server Error 服务器内部故障响应
///
/// **注意**：`message` 会原样返回给调用方，调用方必须传入**对外脱敏**的通用文案，
/// 严禁传入数据库错误、异常名或源码路径 (见 `api.md` §5.2 生产脱敏规范)。
pub fn internal_error(message: impl Into<String>) -> loco_rs::prelude::Result<Response> {
    error_response(StatusCode::INTERNAL_SERVER_ERROR, message)
}

/// 格式化 500 响应，并将完整诊断信息写入服务端日志 (不进入响应体)
///
/// `exception` 与 `trace` 仅记录到 `tracing::error!`，对外只返回脱敏后的 `message`。
pub fn internal_error_with_debug(
    message: impl Into<String>,
    exception: impl Into<String>,
    trace: &[String],
) -> loco_rs::prelude::Result<Response> {
    let message = message.into();
    let exception = exception.into();

    tracing::error!(
        exception = %exception,
        trace = ?trace,
        "服务器内部故障: {message}"
    );

    error_response(StatusCode::INTERNAL_SERVER_ERROR, message)
}

/// 格式化 503 Service Unavailable 服务不可用/维护中响应
pub fn service_unavailable(message: impl Into<String>) -> loco_rs::prelude::Result<Response> {
    error_response(StatusCode::SERVICE_UNAVAILABLE, message)
}

// =========================================================================
// 600 ~ 999 业务逻辑错误码响应 (code 序列化为 BIZ_<code>)
// =========================================================================

/// 格式化带自定义业务错误码 (如 601) 的错误响应 (默认 HTTP 400 Bad Request)
pub fn custom_error(code: u32, message: impl Into<String>) -> loco_rs::prelude::Result<Response> {
    custom_error_with_status(StatusCode::BAD_REQUEST, code, message)
}

/// 格式化带指定 HTTP 状态码与自定义业务错误码的错误响应
pub fn custom_error_with_status(
    status: StatusCode,
    code: u32,
    message: impl Into<String>,
) -> loco_rs::prelude::Result<Response> {
    biz_error_response(status, code, message, None)
}

/// 格式化带自定义业务错误码与结构化细节的错误响应 (默认 HTTP 400 Bad Request)
pub fn custom_error_with_details<D: Serialize>(
    code: u32,
    message: impl Into<String>,
    details: D,
) -> loco_rs::prelude::Result<Response> {
    custom_error_with_status_and_details(StatusCode::BAD_REQUEST, code, message, details)
}

/// 格式化带指定 HTTP 状态码、自定义业务错误码与结构化细节的错误响应
pub fn custom_error_with_status_and_details<D: Serialize>(
    status: StatusCode,
    code: u32,
    message: impl Into<String>,
    details: D,
) -> loco_rs::prelude::Result<Response> {
    let fields = serde_json::to_value(details).ok();
    biz_error_response(status, code, message, fields)
}

/// 格式化 600~999 业务逻辑错误响应 (默认 HTTP 400 Bad Request)
pub fn biz(code: u32, message: impl Into<String>) -> loco_rs::prelude::Result<Response> {
    custom_error(code, message)
}

/// 格式化 600~999 业务逻辑错误响应 (带结构化 details，默认 HTTP 400 Bad Request)
pub fn biz_with_details<D: Serialize>(
    code: u32,
    message: impl Into<String>,
    details: D,
) -> loco_rs::prelude::Result<Response> {
    custom_error_with_details(code, message, details)
}

/// 格式化 600~999 业务逻辑错误响应 (带指定 HTTP Wire 状态码)
pub fn biz_with_status(
    status: StatusCode,
    code: u32,
    message: impl Into<String>,
) -> loco_rs::prelude::Result<Response> {
    custom_error_with_status(status, code, message)
}
