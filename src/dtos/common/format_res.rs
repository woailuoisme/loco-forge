//! Controller 响应快速格式化辅助模块 (Response Formatting Helpers)
//!
//! 提供一系列快速将业务数据或错误包装为标准 Axum `Result<Response>` 的函数，
//! 按 HTTP 状态码分组（2xx 成功 / 4xx 客户端错误 / 5xx 服务端错误）。
//!
//! # 示例
//! ```rust,ignore
//! use crate::dtos::common::{format_res, ErrorCode};
//!
//! // 成功响应
//! format_res::ok(user_dto)
//! format_res::page(items, page, per_page, total)
//!
//! // 预置 ErrorCode 业务错误
//! format_res::biz(ErrorCode::USER_NOT_FOUND)
//! format_res::biz_msg(ErrorCode::PARAM_INVALID, "用户名不能为空")
//!
//! // 自定义 HTTP 错误
//! format_res::not_found("用户不存在")
//! format_res::validation_error_with_details("校验失败", field_errors)
//! ```

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use ts_rs::TS;

use super::error_code::ErrorCode;
use super::response::ApiResponse;

/// 内部通用辅助：快速构建 HTTP 响应
fn respond<T: Serialize>(status: StatusCode, res: ApiResponse<T>) -> loco_rs::prelude::Result<Response> {
    Ok((status, Json(res)).into_response())
}

/// 内部通用辅助：快速构建标准 HTTP 错误响应
fn error_response(status: StatusCode, message: impl Into<String>) -> loco_rs::prelude::Result<Response> {
    let res = ApiResponse::<serde_json::Value>::error_simple(status.as_u16() as u32, message);
    respond(status, res)
}

/// 内部通用辅助：快速构建带结构化 details 的 HTTP 错误响应
fn error_response_with_details<D: Serialize>(
    status: StatusCode,
    message: impl Into<String>,
    details: D,
) -> loco_rs::prelude::Result<Response> {
    let mut res = ApiResponse::<serde_json::Value>::error_simple(status.as_u16() as u32, message);
    res.errors = serde_json::to_value(details).ok();
    respond(status, res)
}

// =========================================================================
// 2xx 成功类响应
// =========================================================================

/// 格式化标准 200 成功响应为 Controller `Result<Response>`
pub fn ok<T: Serialize>(data: T) -> loco_rs::prelude::Result<Response> {
    respond(StatusCode::OK, ApiResponse::ok(data))
}

/// 格式化带自定义消息的 200 成功响应
pub fn ok_msg<T: Serialize>(data: T, message: impl Into<String>) -> loco_rs::prelude::Result<Response> {
    respond(StatusCode::OK, ApiResponse::ok_with_message(data, message))
}

/// 格式化 201 资源创建成功响应 (默认提示消息 "创建成功")
pub fn created<T: Serialize>(data: T) -> loco_rs::prelude::Result<Response> {
    respond(StatusCode::CREATED, ApiResponse::ok_with_message(data, "创建成功"))
}

/// 格式化带自定义消息的 201 资源创建成功响应
pub fn created_msg<T: Serialize>(data: T, message: impl Into<String>) -> loco_rs::prelude::Result<Response> {
    respond(StatusCode::CREATED, ApiResponse::ok_with_message(data, message))
}

/// 格式化 202 已接受（异步处理中）响应
pub fn accepted_msg(message: impl Into<String>) -> loco_rs::prelude::Result<Response> {
    respond(StatusCode::ACCEPTED, ApiResponse::success_empty(message))
}

/// 格式化无载荷成功响应 (如注册成功、密码重置受理、删除成功等)
pub fn empty_msg(message: impl Into<String>) -> loco_rs::prelude::Result<Response> {
    respond(StatusCode::OK, ApiResponse::success_empty(message))
}

/// 格式化分页列表响应
pub fn page<T: Serialize + TS>(
    items: Vec<T>,
    current_page: u64,
    per_page: u64,
    total: u64,
) -> loco_rs::prelude::Result<Response> {
    respond(
        StatusCode::OK,
        ApiResponse::page_response(items, current_page, per_page, total),
    )
}

// =========================================================================
// 4xx 客户端错误响应
// =========================================================================

/// 格式化 400 Bad Request 客户端无效请求响应
pub fn bad_request(message: impl Into<String>) -> loco_rs::prelude::Result<Response> {
    error_response(StatusCode::BAD_REQUEST, message)
}

/// 格式化带结构化细节的 400 Bad Request 响应
pub fn bad_request_with_details<D: Serialize>(
    message: impl Into<String>,
    details: D,
) -> loco_rs::prelude::Result<Response> {
    error_response_with_details(StatusCode::BAD_REQUEST, message, details)
}

/// 格式化 401 Unauthorized 未登录或 Token 效验失败响应
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

/// 格式化 404 Not Found 路由/接口路径不存在响应 (使用 ErrorCode::ROUTE_NOT_FOUND 错误码 404003)
pub fn route_not_found() -> loco_rs::prelude::Result<Response> {
    biz(ErrorCode::ROUTE_NOT_FOUND)
}

/// 格式化 405 Method Not Allowed 请求方法不允许响应
pub fn method_not_allowed(message: impl Into<String>) -> loco_rs::prelude::Result<Response> {
    error_response(StatusCode::METHOD_NOT_ALLOWED, message)
}

/// 格式化 409 Conflict 资源状态冲突（如唯一约束重复）响应
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

/// 格式化 500 Internal Server Error 服务器内部故障响应 (对外脱敏)
pub fn internal_error(message: impl Into<String>) -> loco_rs::prelude::Result<Response> {
    error_response(StatusCode::INTERNAL_SERVER_ERROR, message)
}

/// 格式化带完整调试诊断信息的 500 响应 (仅在 Debug/Test 模式保留 exception 与 trace)
pub fn internal_error_with_debug(
    message: impl Into<String>,
    exception: impl Into<String>,
    trace: Vec<String>,
) -> loco_rs::prelude::Result<Response> {
    let (opt_exception, opt_trace) = if cfg!(debug_assertions) {
        (Some(exception.into()), Some(trace))
    } else {
        (None, None)
    };
    let res = ApiResponse::<serde_json::Value>::error(
        StatusCode::INTERNAL_SERVER_ERROR.as_u16() as u32,
        message,
        None,
        opt_exception,
        opt_trace,
    );
    respond(StatusCode::INTERNAL_SERVER_ERROR, res)
}

/// 格式化 503 Service Unavailable 服务不可用/维护中响应
pub fn service_unavailable(message: impl Into<String>) -> loco_rs::prelude::Result<Response> {
    error_response(StatusCode::SERVICE_UNAVAILABLE, message)
}

// =========================================================================
// 自定义业务错误码响应 (如 400001 用户冻结, 404001 用户不存在)
// =========================================================================

fn status_from_code(code: u32) -> StatusCode {
    u16::try_from(code / 1000)
        .ok()
        .and_then(|s| StatusCode::from_u16(s).ok())
        .unwrap_or(StatusCode::BAD_REQUEST)
}

/// 格式化带自定义业务错误码 (如 400001) 的错误响应
pub fn custom_error(code: u32, message: impl Into<String>) -> loco_rs::prelude::Result<Response> {
    let status = status_from_code(code);
    let res = ApiResponse::<serde_json::Value>::error_simple(code, message);
    respond(status, res)
}

/// 格式化带自定义业务错误码与结构化细节的错误响应
pub fn custom_error_with_details<D: Serialize>(
    code: u32,
    message: impl Into<String>,
    details: D,
) -> loco_rs::prelude::Result<Response> {
    let status = status_from_code(code);
    let mut res = ApiResponse::<serde_json::Value>::error_simple(code, message);
    res.errors = serde_json::to_value(details).ok();
    respond(status, res)
}

/// 使用预定义 ErrorCode 快速格式化业务错误响应
pub fn biz(err_code: ErrorCode) -> loco_rs::prelude::Result<Response> {
    custom_error(err_code.code, err_code.message)
}

/// 使用预定义 ErrorCode 与自定义描述消息格式化业务错误响应
pub fn biz_msg(err_code: ErrorCode, message: impl Into<String>) -> loco_rs::prelude::Result<Response> {
    custom_error(err_code.code, message)
}

/// 使用预定义 ErrorCode 及结构化 details 格式化业务错误响应
pub fn biz_with_details<D: Serialize>(err_code: ErrorCode, details: D) -> loco_rs::prelude::Result<Response> {
    custom_error_with_details(err_code.code, err_code.message, details)
}
