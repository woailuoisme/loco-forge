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

/// 格式化 201 资源创建成功响应 (默认提示消息 "创建成功"，JSON code 为 201)
pub fn created<T: Serialize>(data: T) -> loco_rs::prelude::Result<Response> {
    respond(StatusCode::CREATED, ApiResponse::created(data))
}

/// 格式化带自定义消息的 201 资源创建成功响应 (JSON code 为 201)
pub fn created_msg<T: Serialize>(data: T, message: impl Into<String>) -> loco_rs::prelude::Result<Response> {
    respond(StatusCode::CREATED, ApiResponse::created_with_message(data, message))
}

/// 格式化 202 已接受（异步处理中）响应 (JSON code 为 202)
pub fn accepted_msg(message: impl Into<String>) -> loco_rs::prelude::Result<Response> {
    respond(
        StatusCode::ACCEPTED,
        ApiResponse::success_empty_with_code(StatusCode::ACCEPTED.as_u16() as u32, message),
    )
}

/// 格式化无载荷成功响应 (如注册成功、密码重置受理、删除成功等)
pub fn empty_msg(message: impl Into<String>) -> loco_rs::prelude::Result<Response> {
    respond(StatusCode::OK, ApiResponse::success_empty(message))
}

/// 格式化 CRUD 查询成功响应 (默认提示消息 "数据获取成功")
pub fn retrieved<T: Serialize>(data: T) -> loco_rs::prelude::Result<Response> {
    ok_msg(data, "数据获取成功")
}

/// 格式化 CRUD 更新成功响应 (默认提示消息 "数据更新成功")
pub fn updated<T: Serialize>(data: T) -> loco_rs::prelude::Result<Response> {
    ok_msg(data, "数据更新成功")
}

/// 格式化 CRUD 删除成功响应 (默认提示消息 "数据删除成功"，无载荷)
pub fn deleted() -> loco_rs::prelude::Result<Response> {
    empty_msg("数据删除成功")
}

/// 格式化 CRUD 删除成功响应 (默认提示消息 "数据删除成功"，带载荷)
pub fn deleted_with_data<T: Serialize>(data: T) -> loco_rs::prelude::Result<Response> {
    ok_msg(data, "数据删除成功")
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

/// 格式化带扩展数据的分页列表响应
pub fn page_with_extras<T: Serialize + TS, E: Serialize>(
    items: Vec<T>,
    current_page: u64,
    per_page: u64,
    total: u64,
    extras: E,
) -> loco_rs::prelude::Result<Response> {
    respond(
        StatusCode::OK,
        ApiResponse::page_with_extras_response(items, current_page, per_page, total, extras),
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

/// 格式化 404 Not Found 路由/接口路径不存在响应
pub fn route_not_found() -> loco_rs::prelude::Result<Response> {
    not_found("请求的接口路径不存在")
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

/// 格式化带完整调试诊断信息的 500 响应 (仅在 Debug/Test 模式保留 trace 堆栈跟踪)
pub fn internal_error_with_debug(
    message: impl Into<String>,
    _exception: impl Into<String>,
    trace: Vec<String>,
) -> loco_rs::prelude::Result<Response> {
    let opt_trace = if cfg!(debug_assertions) { Some(trace) } else { None };
    let res = ApiResponse::<serde_json::Value>::error_with_trace(
        StatusCode::INTERNAL_SERVER_ERROR.as_u16() as u32,
        message,
        None,
        opt_trace,
    );
    respond(StatusCode::INTERNAL_SERVER_ERROR, res)
}

/// 格式化 503 Service Unavailable 服务不可用/维护中响应
pub fn service_unavailable(message: impl Into<String>) -> loco_rs::prelude::Result<Response> {
    error_response(StatusCode::SERVICE_UNAVAILABLE, message)
}

/// 格式化带数据载荷的 503 Service Unavailable 响应
pub fn service_unavailable_data<T: Serialize>(
    data: T,
    message: impl Into<String>,
) -> loco_rs::prelude::Result<Response> {
    let res = ApiResponse {
        success: false,
        code: 503,
        message: message.into(),
        data: Some(data),
        errors: None,
        trace: None,
    };
    respond(StatusCode::SERVICE_UNAVAILABLE, res)
}

// =========================================================================
// 600 ~ 999 业务逻辑错误码响应
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
    let res = ApiResponse::<serde_json::Value>::error_simple(code, message);
    respond(status, res)
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
    let mut res = ApiResponse::<serde_json::Value>::error_simple(code, message);
    res.errors = serde_json::to_value(details).ok();
    respond(status, res)
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
