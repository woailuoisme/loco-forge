//! 分页相关数据结构 (Pagination Types)
//!
//! 提供标准分页元数据 [`PageMeta`]、分页响应容器 [`PaginatedData`] 与标准分页查询提取器 [`PaginationQuery`]。

use crate::common::consts::pagination::{DEFAULT_PAGE, DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

/// 通用分页查询提取参数 (Pagination Query Params)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, utoipa::IntoParams, utoipa::ToSchema)]
pub struct PaginationQuery {
    /// 页码 (从 1 开始，默认 1)
    pub page: Option<u64>,
    /// 每页数量 (默认 20，最大 100)
    pub per_page: Option<u64>,
}

impl PaginationQuery {
    /// 构造新的分页查询参数
    pub fn new(page: Option<u64>, per_page: Option<u64>) -> Self {
        Self { page, per_page }
    }

    /// 获取合法化的当前页码 (>= 1)
    pub fn page(&self) -> u64 {
        self.page.unwrap_or(DEFAULT_PAGE).max(1)
    }

    /// 获取合法化的每页数量 (1..=100)
    pub fn per_page(&self) -> u64 {
        self.per_page.unwrap_or(DEFAULT_PAGE_SIZE).clamp(1, MAX_PAGE_SIZE)
    }

    /// 获取 SQL 数据库查询 offset 偏移量
    pub fn offset(&self) -> u64 {
        (self.page() - 1) * self.per_page()
    }
}

impl Default for PaginationQuery {
    fn default() -> Self {
        Self {
            page: Some(DEFAULT_PAGE),
            per_page: Some(DEFAULT_PAGE_SIZE),
        }
    }
}

/// 通用分页元数据 (Pagination Metadata)
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
pub struct PageMeta {
    /// 当前页码 (从 1 开始)
    #[ts(type = "number")]
    pub current_page: u64,
    /// 每页记录数量
    #[ts(type = "number")]
    pub per_page: u64,
    /// 总页数
    #[ts(type = "number")]
    pub last_page: u64,
    /// 总记录数
    #[ts(type = "number")]
    pub total: u64,
}

impl PageMeta {
    /// 构造分页元数据并自动计算 last_page
    pub fn new(current_page: u64, per_page: u64, total: u64) -> Self {
        let per_page = per_page.max(1);
        let last_page = total.div_ceil(per_page).max(1);
        Self {
            current_page,
            per_page,
            last_page,
            total,
        }
    }
}

/// 通用分页数据响应结构
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
pub struct PaginatedData<T: TS> {
    /// 列表数据项
    pub items: Vec<T>,
    /// 分页元数据信息
    pub meta: PageMeta,
}

/// 兼容别名 `Page<T>`
pub type Page<T> = PaginatedData<T>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_query_defaults_and_bounds() {
        let q = PaginationQuery::new(None, None);
        assert_eq!(q.page(), 1);
        assert_eq!(q.per_page(), 20);
        assert_eq!(q.offset(), 0);

        let q = PaginationQuery::new(Some(0), Some(500));
        assert_eq!(q.page(), 1);
        assert_eq!(q.per_page(), 100); // capped at MAX_PAGE_SIZE

        let q = PaginationQuery::new(Some(3), Some(15));
        assert_eq!(q.page(), 3);
        assert_eq!(q.per_page(), 15);
        assert_eq!(q.offset(), 30);
    }
}
