//! 统一 API 响应结构体 (Unified API Response)
//!
//! 提供所有控制器返回的标准 JSON 外层响应载荷 [`ApiResponse`] 与
//! 其无数据变体的类型别名 [`ApiError`]。

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

use super::pagination::PageMeta;
use super::pagination::PaginatedData;

/// 统一 API JSON 响应载荷规范结构
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ApiResponse<T = serde_json::Value> {
    /// 请求是否成功标识
    pub success: bool,
    /// HTTP 状态码或业务自定义错误码
    pub code: u32,
    /// 结果或错误的人类可读描述
    pub message: String,
    /// 响应数据载荷 (成功时返回，可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    /// 数据校验或详细错误描述信息 (失败时返回，可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<serde_json::Value>,
    /// 异常类名 (仅在开发/测试环境返回，可选)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exception: Option<String>,
    /// 用于调试的堆栈跟踪信息 (仅在开发/测试环境返回，可选)
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
            exception: None,
            trace: None,
        }
    }

    /// 创建标准 200 成功响应 (默认消息 "操作成功完成")
    pub fn ok(data: T) -> Self {
        Self::ok_with_message(data, "操作成功完成")
    }

    /// 创建分页成功响应
    pub fn page_response(items: Vec<T>, current_page: u64, per_page: u64, total: u64) -> ApiResponse<PaginatedData<T>>
    where
        T: TS,
    {
        let paginated = PaginatedData {
            items,
            meta: PageMeta::new(current_page, per_page, total),
        };
        ApiResponse::ok_with_message(paginated, "success")
    }
}

impl ApiResponse<serde_json::Value> {
    /// 创建简单无载荷成功响应
    pub fn success_empty(message: impl Into<String>) -> Self {
        Self {
            success: true,
            code: 200,
            message: message.into(),
            data: None,
            errors: None,
            exception: None,
            trace: None,
        }
    }

    /// 创建简单失败响应 (不带详细 errors / exception / trace)
    pub fn error_simple(code: u32, message: impl Into<String>) -> Self {
        Self::error(code, message, None, None, None)
    }

    /// 创建失败/异常响应
    pub fn error(
        code: u32,
        message: impl Into<String>,
        errors: Option<serde_json::Value>,
        exception: Option<String>,
        trace: Option<Vec<String>>,
    ) -> Self {
        Self {
            success: false,
            code,
            message: message.into(),
            data: None,
            errors,
            exception,
            trace,
        }
    }
}

/// 兼容别名 ApiError (指向统一 ApiResponse<serde_json::Value>)
pub type ApiError = ApiResponse<serde_json::Value>;
