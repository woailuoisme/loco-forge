//! Excel 处理模块（仿照 Laravel Excel 架构）
//!
//! # 架构概览
//!
//! | 模块 | 对应 Laravel Excel 概念 |
//! |------|------------------------|
//! | [`export`] | `FromCollection` + `WithHeadings` + `WithStyles` |
//! | [`import`] | `ToModel` + `WithHeadingRow` + `WithValidation` |
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
//!     fn headings() -> Vec<String> {
//!         vec!["姓名".into(), "年龄".into()]
//!     }
//!     fn row_cells(&self) -> Vec<CellValue> {
//!         vec![self.name.clone().into(), self.age.into()]
//!     }
//!     fn sheet_name() -> String { "用户列表".into() }
//! }
//!
//! // 在 Axum 控制器中：
//! let bytes = export_to_bytes(&rows)?;
//! Ok(ExcelResponse::download(bytes, "users-2026-08-06.xlsx"))
//! ```

pub mod async_task;
pub mod export;
pub mod import;
pub mod response;

// 便捷 re-export —— 最常用的类型无需深层 use 路径
pub use async_task::{spawn_excel_export, spawn_excel_import};
pub use export::{export_to_bytes, CellValue, ExcelExport};
pub use import::{import_from_bytes, ExcelImport, ImportResult, RowError};
pub use response::ExcelResponse;
