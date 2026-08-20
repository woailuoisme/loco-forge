//! Excel 处理模块（仿照 Laravel Excel 架构）
//!
//! # 架构概览
//!
//! | 模块 | 对应 Laravel Excel 概念 |
//! |------|------------------------|
//! | [`export`] | `FromCollection` + `WithHeadings` + `WithStyles` |
//! | [`import`] | `ToModel` + `WithHeadingRow` + `WithValidation` |
//! | [`cell`] | `CellValue` 数据类型封装 |
//! | [`response`] | `Excel::download()` |
//! | [`async_task`] | `spawn_excel_export` / `spawn_excel_import` |
//!
//! # 快速开始（导出）
//!
//! ```rust,ignore
//! use crate::common::excel::{export::ExcelExport, export::export_to_bytes, CellValue, response::ExcelResponse};
//!
//! struct UserRow { name: String, age: u32 }
//!
//! impl ExcelExport for UserRow {
//!     fn headings() -> Vec<&'static str> {
//!         vec!["姓名", "年龄"]
//!     }
//!     fn row_cells(&self) -> Vec<CellValue> {
//!         vec![self.name.clone().into(), self.age.into()]
//!     }
//!     fn sheet_name() -> &'static str { "用户列表" }
//! }
//!
//! // 在 Axum 控制器中：
//! let bytes = export_to_bytes(&rows)?;
//! Ok(ExcelResponse::download(bytes, "users-2026-08-06.xlsx"))
//! ```

pub mod async_task;
pub mod cell;
pub mod export;
pub mod import;
pub mod response;

// 便捷 re-export —— 最常用的类型无需深层 use 路径
pub use async_task::{spawn_excel_export, spawn_excel_import};
pub use cell::CellValue;
pub use export::{ExcelExport, ExportOptions, export_chunked_to_file, export_to_bytes};
pub use import::{
    ChunkImportSummary, ExcelImport, ExcelRow, ImportOptions, ImportResult, RowError,
    import_from_bytes, import_in_chunks_from_reader,
};
pub use response::ExcelResponse;

/// Excel 统一门面引擎 (Excel Facade Engine)
///
/// 提供开箱即用的同步与流式 Excel 导入导出核心能力。
pub struct Excel;

impl Excel {
    /// 内存全量导出为 Excel 二进制字节流
    pub fn export<T: ExcelExport>(items: &[T]) -> loco_rs::Result<bytes::Bytes> {
        export::export_to_bytes(items)
    }

    /// 内存全量解析 Excel 二进制字节流
    pub fn import<T: ExcelImport>(bytes: impl AsRef<[u8]>) -> loco_rs::Result<ImportResult<T>> {
        import::import_from_bytes(bytes)
    }

    /// 构造 Excel 文件下载 HTTP 响应
    pub fn download(bytes: impl Into<bytes::Bytes>, filename: impl Into<String>) -> ExcelResponse {
        ExcelResponse::download(bytes, filename)
    }
}
