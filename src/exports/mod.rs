//! Excel 导出定义模块 (Excel Export Definitions)
//!
//! 集中存放各业务资源的 [`ExcelExport`](crate::common::excel::ExcelExport) 实现，与 `src/imports` 对称：
//! - 本模块：`impl ExcelExport for XxxResource`，定义表头、列宽与单元格取值；
//! - `src/imports`：`impl ExcelImport for XxxImportRow`，定义行解析与校验。
//!
//! 底层读写引擎见 [`crate::common::excel`]。

pub mod user;

pub use user::USER_EXPORT_HEADINGS;
