//! 分页相关数据结构 (Pagination Types)
//!
//! 提供标准分页元数据 [`PageMeta`]、分页响应容器 [`PaginatedResource`] 与
//! 标准分页查询提取器 [`PaginationRequest`]。
//!
//! 全部字段遵循 `api.md` §6.1 的 `camelCase` JSON 命名与
//! `{page, pageSize, total, totalPages}` 元数据结构。

use crate::common::consts::pagination::{DEFAULT_PAGE, DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;
use validator::Validate;

/// 通用分页查询提取参数 (Pagination Query Params)
///
/// 查询串形式：`?page=1&pageSize=20`
#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, Validate, utoipa::IntoParams, utoipa::ToSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct PaginationRequest {
    /// 页码 (从 1 开始，默认 1)
    pub page: Option<u64>,
    /// 每页数量 (默认 20，最大 100)
    pub page_size: Option<u64>,
}

impl PaginationRequest {
    /// 构造新的分页查询参数
    pub fn new(page: Option<u64>, page_size: Option<u64>) -> Self {
        Self { page, page_size }
    }

    /// 获取合法化的当前页码 (>= 1)
    pub fn page(&self) -> u64 {
        self.page.unwrap_or(DEFAULT_PAGE).max(1)
    }

    /// 获取合法化的每页数量 (1..=100)
    pub fn page_size(&self) -> u64 {
        self.page_size
            .unwrap_or(DEFAULT_PAGE_SIZE)
            .clamp(1, MAX_PAGE_SIZE)
    }

    /// 获取 SQL 数据库查询 offset 偏移量
    pub fn offset(&self) -> u64 {
        (self.page() - 1) * self.page_size()
    }
}

impl Default for PaginationRequest {
    fn default() -> Self {
        Self {
            page: Some(DEFAULT_PAGE),
            page_size: Some(DEFAULT_PAGE_SIZE),
        }
    }
}

/// 通用分页元数据 (Pagination Metadata)
///
/// JSON 形态：`{"page": 1, "pageSize": 20, "total": 100, "totalPages": 5}`
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PageMeta {
    /// 当前页码 (从 1 开始)
    #[ts(type = "number")]
    pub page: u64,
    /// 每页记录数量
    #[ts(type = "number")]
    pub page_size: u64,
    /// 总记录数
    #[ts(type = "number")]
    pub total: u64,
    /// 总页数
    #[ts(type = "number")]
    pub total_pages: u64,
}

impl PageMeta {
    /// 构造分页元数据并自动计算 total_pages
    pub fn new(page: u64, page_size: u64, total: u64) -> Self {
        let page_size = page_size.max(1);
        let total_pages = total.div_ceil(page_size).max(1);
        Self {
            page,
            page_size,
            total,
            total_pages,
        }
    }
}

/// 通用分页数据响应结构
///
/// 全系统唯一的分页信封：`{"items": [...], "meta": {...}}`。
/// 列表为空时 `items` 必须为 `[]`，严禁 `null`。
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
pub struct PaginatedResource<T: TS> {
    /// 列表数据项
    pub items: Vec<T>,
    /// 分页元数据信息
    pub meta: PageMeta,
}

/// 兼容别名 `PaginatedData<T>`
pub type PaginatedData<T> = PaginatedResource<T>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_query_defaults_and_bounds() {
        let q = PaginationRequest::new(None, None);
        assert_eq!(q.page(), 1);
        assert_eq!(q.page_size(), 20);
        assert_eq!(q.offset(), 0);

        let q = PaginationRequest::new(Some(0), Some(500));
        assert_eq!(q.page(), 1);
        assert_eq!(q.page_size(), 100); // capped at MAX_PAGE_SIZE

        let q = PaginationRequest::new(Some(3), Some(15));
        assert_eq!(q.page(), 3);
        assert_eq!(q.page_size(), 15);
        assert_eq!(q.offset(), 30);
    }

    #[test]
    fn test_pagination_request_camel_case_keys() {
        let json = serde_json::json!({ "page": 2, "pageSize": 30 });
        let q: PaginationRequest = serde_json::from_value(json).unwrap();
        assert_eq!(q.page(), 2);
        assert_eq!(q.page_size(), 30);
    }

    #[test]
    fn test_page_meta_calculation() {
        let meta = PageMeta::new(1, 20, 100);
        assert_eq!(meta.page, 1);
        assert_eq!(meta.page_size, 20);
        assert_eq!(meta.total, 100);
        assert_eq!(meta.total_pages, 5);
    }

    #[test]
    fn test_page_meta_serializes_camel_case() {
        let json = serde_json::to_value(PageMeta::new(1, 20, 25)).unwrap();
        assert_eq!(json["page"], 1);
        assert_eq!(json["pageSize"], 20);
        assert_eq!(json["total"], 25);
        assert_eq!(json["totalPages"], 2);
    }
}
