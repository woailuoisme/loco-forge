//! 统一 API 响应结构体 (Unified API Response)
//!
//! 提供所有控制器返回的标准 JSON 外层响应载荷 [`ApiResponse`] 与
//! 其无数据变体的类型别名 [`ApiError`]。

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

use super::pagination::PageMeta;
use super::pagination::PaginatedData;

/// 统一 API JSON 响应载荷规范结构 (用于 2xx 成功响应 OpenAPI 规范)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ApiResponse<T = serde_json::Value> {
    /// 请求是否成功标识
    #[schema(example = true)]
    pub success: bool,
    /// HTTP 状态码或业务自定义错误码
    #[schema(example = 200)]
    pub code: u32,
    /// 结果或错误的人类可读描述
    #[schema(example = "Success")]
    pub message: String,
    /// 响应数据载荷 (仅 Controller 显式传递载荷时序列化返回，可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    /// 忽略在 2xx 成功响应 Schema 中的 errors 字段
    #[schema(ignore)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<serde_json::Value>,
    /// 忽略在 2xx 成功响应 Schema 中的 trace 字段
    #[schema(ignore)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace: Option<Vec<String>>,
}

/// 统一 API JSON 错误/异常响应结构体 (用于 4xx/5xx/6xx 错误响应 OpenAPI 规范)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "success": false,
    "code": 400,
    "message": "请求处理失败描述"
}))]
pub struct ApiError {
    /// 请求是否成功标识 (固定为 false)
    #[schema(example = false)]
    pub success: bool,
    /// HTTP 状态码或业务自定义错误码
    #[schema(example = 400)]
    pub code: u32,
    /// 错误的人类可读描述消息
    #[schema(example = "请求处理失败描述")]
    pub message: String,
    /// 数据校验或详细错误描述信息 (仅 422 Validation Error 等包含 details 时序列化返回，可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<serde_json::Value>,
    /// 忽略在错误响应 Schema 中的 trace 字段 (调试堆栈信息)
    #[schema(ignore)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace: Option<Vec<String>>,
}

impl<T: Serialize> ApiResponse<T> {
    /// 创建标准 200 成功响应 (自定义消息与数据载荷)
    pub fn ok_with_message(data: T, message: impl Into<String>) -> Self {
        Self {
            success: true,
            code: 200,
            message: message.into(),
            data: Some(data),
            errors: None,
            trace: None,
        }
    }

    /// 创建标准 200 成功响应 (默认消息 "操作成功完成")
    pub fn ok(data: T) -> Self {
        Self::ok_with_message(data, "操作成功完成")
    }

    /// 创建 201 资源创建成功响应 (自定义消息与数据载荷)
    pub fn created_with_message(data: T, message: impl Into<String>) -> Self {
        Self {
            success: true,
            code: 201,
            message: message.into(),
            data: Some(data),
            errors: None,
            trace: None,
        }
    }

    /// 创建 201 资源创建成功响应 (默认消息 "创建成功")
    pub fn created(data: T) -> Self {
        Self::created_with_message(data, "创建成功")
    }

    /// 创建分页成功响应
    pub fn page_response(
        items: Vec<T>,
        current_page: u64,
        per_page: u64,
        total: u64,
    ) -> ApiResponse<PaginatedData<T>>
    where
        T: TS,
    {
        let paginated = PaginatedData {
            items,
            meta: PageMeta::new(current_page, per_page, total),
            extras: None,
        };
        ApiResponse::ok_with_message(paginated, "success")
    }

    /// 创建带额外扩展数据的分页成功响应
    pub fn page_with_extras_response<E: Serialize>(
        items: Vec<T>,
        current_page: u64,
        per_page: u64,
        total: u64,
        extras: E,
    ) -> ApiResponse<PaginatedData<T>>
    where
        T: TS,
    {
        let extras_json = serde_json::to_value(extras).ok();
        let paginated = PaginatedData {
            items,
            meta: PageMeta::new(current_page, per_page, total),
            extras: extras_json,
        };
        ApiResponse::ok_with_message(paginated, "success")
    }
}

impl ApiResponse<serde_json::Value> {
    /// 创建简单无载荷成功响应 (默认 200)
    pub fn success_empty(message: impl Into<String>) -> Self {
        Self::success_empty_with_code(200, message)
    }

    /// 创建指定 HTTP Code 的无载荷成功响应 (如 200, 202)
    pub fn success_empty_with_code(code: u32, message: impl Into<String>) -> Self {
        Self {
            success: true,
            code,
            message: message.into(),
            data: None,
            errors: None,
            trace: None,
        }
    }

    /// 创建简单失败响应 (不带详细 errors 与 trace)
    pub fn error_simple(code: u32, message: impl Into<String>) -> Self {
        Self::error(code, message, None)
    }

    /// 创建失败/异常响应
    pub fn error(code: u32, message: impl Into<String>, errors: Option<serde_json::Value>) -> Self {
        Self::error_with_trace(code, message, errors, None)
    }

    /// 创建带调试堆栈跟踪信息的失败/异常响应 (如 500 内部错误)
    pub fn error_with_trace(
        code: u32,
        message: impl Into<String>,
        errors: Option<serde_json::Value>,
        trace: Option<Vec<String>>,
    ) -> Self {
        Self {
            success: false,
            code,
            message: message.into(),
            data: None,
            errors,
            trace,
        }
    }
}

impl ApiError {
    /// 创建简单失败响应 (不带详细 errors 与 trace)
    pub fn error_simple(code: u32, message: impl Into<String>) -> Self {
        Self::error(code, message, None)
    }

    /// 创建失败/异常响应
    pub fn error(code: u32, message: impl Into<String>, errors: Option<serde_json::Value>) -> Self {
        Self::error_with_trace(code, message, errors, None)
    }

    /// 创建带调试堆栈跟踪信息的失败/异常响应 (如 500 内部错误)
    pub fn error_with_trace(
        code: u32,
        message: impl Into<String>,
        errors: Option<serde_json::Value>,
        trace: Option<Vec<String>>,
    ) -> Self {
        Self {
            success: false,
            code,
            message: message.into(),
            errors,
            trace,
        }
    }
}

impl From<ApiError> for ApiResponse<serde_json::Value> {
    fn from(err: ApiError) -> Self {
        Self {
            code: err.code,
            success: err.success,
            message: err.message,
            data: None,
            errors: err.errors,
            trace: err.trace,
        }
    }
}

impl From<ApiResponse<serde_json::Value>> for ApiError {
    fn from(res: ApiResponse<serde_json::Value>) -> Self {
        Self {
            code: res.code,
            success: res.success,
            message: res.message,
            errors: res.errors,
            trace: res.trace,
        }
    }
}
