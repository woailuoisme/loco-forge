use ts_rs::TS;
use utoipa::ToSchema;

/// 通用分页数据响应包裹
#[derive(serde::Serialize, serde::Deserialize, TS, ToSchema)]
pub struct Page<T: TS> {
    /// 列表数据项
    pub items: Vec<T>,
    /// 总记录数
    #[ts(type = "number")]
    pub total: i64,
    /// 当前页码
    #[ts(type = "number")]
    pub page: i64,
    /// 每页数量
    #[ts(type = "number")]
    pub per_page: i64,
}

/// 通用 API 错误响应
#[derive(serde::Serialize, serde::Deserialize, TS, ToSchema)]
pub struct ApiError {
    /// 错误码数字 (默认使用 HTTP 状态码数值如 404, 400，或自定义业务错误码如 404001)
    pub code: u32,
    /// 错误详细信息
    pub message: String,
    /// 附加错误细节
    #[ts(type = "unknown")]
    pub details: Option<serde_json::Value>,
}
