//! Excel 导入定义模块 (Excel Import Definitions)
//!
//! 集中存放各业务资源的 [`ExcelImport`] 实现，与 `src/exports` 对称：
//! - 本模块：`impl ExcelImport for XxxImportRow`，定义按列名解析与行级校验；
//! - `src/exports`：`impl ExcelExport for XxxResource`，定义表头与单元格取值。
//!
//! 底层读写引擎见 [`crate::common::excel`]。

pub mod user;

pub use user::USER_IMPORT_HEADINGS;
