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
//! | [`async_task`] | `spawn_excel_export` / `spawn_excel_blocking` |
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
pub use async_task::{spawn_excel_blocking, spawn_excel_export};
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

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    struct TestUserExport {
        id: i64,
        name: String,
        score: f64,
        is_active: bool,
    }

    impl ExcelExport for TestUserExport {
        fn headings() -> Vec<&'static str> {
            vec!["ID", "姓名", "得分", "是否激活"]
        }

        fn row_cells(&self) -> Vec<CellValue> {
            vec![
                self.id.into(),
                self.name.clone().into(),
                self.score.into(),
                self.is_active.into(),
            ]
        }

        fn sheet_name() -> &'static str {
            "测试数据"
        }
    }

    #[rstest]
    #[case(CellValue::from("hello"), CellValue::String("hello".to_string()))]
    #[case(CellValue::from(42i64), CellValue::Int(42))]
    #[case(CellValue::from(3.5f64), CellValue::Float(3.5))]
    #[case(CellValue::from(true), CellValue::Bool(true))]
    #[case(CellValue::from(Option::<i32>::None), CellValue::Empty)]
    #[case(CellValue::from(Some(100i32)), CellValue::Int(100))]
    fn test_cell_value_conversions(#[case] actual: CellValue, #[case] expected: CellValue) {
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_excel_export_native_types() {
        let rows = vec![
            TestUserExport {
                id: 1,
                name: "Alice".into(),
                score: 95.5,
                is_active: true,
            },
            TestUserExport {
                id: 2,
                name: "Bob".into(),
                score: 88.0,
                is_active: false,
            },
        ];

        let bytes = export_to_bytes(&rows).expect("Should export excel to bytes");
        assert!(!bytes.is_empty());

        // 校验 PKZIP / XLSX 魔数标识 (50 4B 03 04)
        assert_eq!(&bytes[0..4], &[0x50, 0x4B, 0x03, 0x04]);
    }

    #[tokio::test]
    async fn test_async_spawn_excel_export() {
        let rows = vec![TestUserExport {
            id: 10,
            name: "Charlie".into(),
            score: 99.0,
            is_active: true,
        }];

        let bytes = spawn_excel_export(move || export_to_bytes(&rows))
            .await
            .expect("Async excel export should succeed");

        assert!(!bytes.is_empty());
    }

    struct TestUserImport {
        name: String,
        score: f64,
    }

    impl ExcelImport for TestUserImport {
        fn from_row(row: &ExcelRow) -> Result<Self, Vec<RowError>> {
            let name = row.get_string("姓名");
            let score = row.get_f64("得分").unwrap_or(0.0);
            Ok(Self { name, score })
        }

        fn sheet_name() -> Option<&'static str> {
            Some("测试数据")
        }
    }

    #[test]
    fn test_excel_import_from_bytes() {
        let rows = vec![TestUserExport {
            id: 1,
            name: "Alice".into(),
            score: 95.5,
            is_active: true,
        }];
        let bytes = export_to_bytes(&rows).unwrap();
        let res = import_from_bytes::<TestUserImport>(&bytes).expect("Should import");
        assert_eq!(res.valid_rows.len(), 1);
        assert_eq!(res.valid_rows[0].name, "Alice");
        assert_eq!(res.valid_rows[0].score, 95.5);
    }

    #[test]
    fn test_excel_export_chunked_to_file_and_chunked_import() {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join(format!("test_chunked_export_{}.xlsx", std::process::id()));

        let total_mock_items = 2500usize;
        let chunk_size = 1000usize;

        // 1. 模拟数据库分批 Fetch：每次 1000 条，最大 2500 条
        let fetch_fn = |page: usize,
                        page_size: usize|
         -> Result<Vec<TestUserExport>, loco_rs::prelude::Error> {
            let start = (page - 1) * page_size;
            if start >= total_mock_items {
                return Ok(Vec::new());
            }
            let end = (start + page_size).min(total_mock_items);
            let items = (start..end)
                .map(|i| TestUserExport {
                    id: i as i64 + 1,
                    name: format!("User_{i}"),
                    score: 80.0 + (i as f64 * 0.01),
                    is_active: i % 2 == 0,
                })
                .collect();
            Ok(items)
        };

        // 设置每个 Sheet 最多 1000 行，2500 条数据将自动拆分为 3 个 Sheet
        let export_options = ExportOptions {
            chunk_size,
            max_rows_per_sheet: 1000,
        };

        let exported_rows = export_chunked_to_file(fetch_fn, &file_path, &export_options)
            .expect("Should export chunked to file successfully");
        assert_eq!(exported_rows, 2500);
        assert!(file_path.exists());

        // 2. 模拟分批回调导入：验证 import_in_chunks_from_reader
        let file = std::fs::File::open(&file_path).expect("Should open exported excel file");
        let import_options = ImportOptions {
            chunk_size: 1000,
            max_errors: 10,
        };

        let mut imported_chunk_count = 0usize;
        let mut total_imported_rows = 0usize;

        let summary = import_in_chunks_from_reader::<_, TestUserImport, _>(
            file,
            &import_options,
            |chunk: &[TestUserImport]| {
                imported_chunk_count += 1;
                total_imported_rows += chunk.len();
                Ok(())
            },
        )
        .expect("Should import chunked successfully");

        assert_eq!(summary.total_valid_rows, 2500);
        assert_eq!(total_imported_rows, 2500);
        assert!(!summary.has_errors());
        assert!(imported_chunk_count >= 3);

        // 清理临时测试文件
        let _ = std::fs::remove_file(file_path);
    }

    #[test]
    fn test_excel_facade_export_and_import() {
        let users = vec![TestUserExport {
            id: 101,
            name: "FacadeUser".to_string(),
            score: 99.5,
            is_active: true,
        }];

        let bytes = Excel::export(&users).expect("Excel::export should succeed");
        assert!(!bytes.is_empty());

        let result = Excel::import::<TestUserImport>(&bytes).expect("Excel::import should succeed");
        assert_eq!(result.valid_rows.len(), 1);
        assert_eq!(result.valid_rows[0].name, "FacadeUser");
        assert_eq!(result.valid_rows[0].score, 99.5);
    }
}
