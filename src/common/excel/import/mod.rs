//! Excel 导入主接口定义与二进制字节解析器

pub mod chunk;
pub mod reader;

pub use chunk::{ChunkImportSummary, import_in_chunks_from_reader};
pub use reader::ExcelRow;

use crate::common::consts;
use calamine::{Reader, Xlsx};
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use std::io::Cursor;

/// 校验失败的单行错误明细（类似于 Laravel Excel `ValidationException` 结构）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowError {
    /// Excel 中的行号（从 1 开始计数，第 1 行为 Header，数据行从 2 开始）
    pub line: usize,
    /// 触发错误的列标题 / 字段名
    pub column: String,
    /// 错误发生时的单元格原始输入文本
    pub raw_value: String,
    /// 具体的错误原因与校验描述
    pub message: String,
}

/// Excel 导入汇总结果包装
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult<T> {
    /// 成功解析并通过校验的强类型数据行
    pub valid_rows: Vec<T>,
    /// 解析失败或校验不通过的错误集合（按行号与列记录）
    pub errors: Vec<RowError>,
}

impl<T> ImportResult<T> {
    /// 是否存在校验或解析错误
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

/// Excel 导入接口（仿 Laravel Excel `ToModel` / `WithHeadingRow` / `WithValidation` 规范）
pub trait ExcelImport: Sized {
    /// 根据强类型零分配 `ExcelRow` 行上下文构造数据对象
    fn from_row(row: &ExcelRow) -> Result<Self, Vec<RowError>>;

    /// 指定要导入的工作表名称（默认读取第一个工作表）
    fn sheet_name() -> Option<&'static str> {
        None
    }
}

/// 海量数据分批导入配置选项
#[derive(Debug, Clone)]
pub struct ImportOptions {
    /// 每一批处理与写入数据库的数据行数（预设 1,000 行）
    pub chunk_size: usize,
    /// 允许收集的最大错误数量阈值（默认 100 条，达到阈值立即熔断提前停止解析，防止海量错误导致内存溢出）
    pub max_errors: usize,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            chunk_size: consts::excel::DEFAULT_CHUNK_SIZE,
            max_errors: consts::excel::MAX_IMPORT_ERRORS,
        }
    }
}

/// 将二进制 `.xlsx` 字节流解析为强类型列表与错误集合（支持自动拆分的多 Sheet 连续解析）
pub fn import_from_bytes<T: ExcelImport>(bytes: &[u8]) -> Result<ImportResult<T>, Error> {
    let cursor = Cursor::new(bytes);
    let mut workbook: Xlsx<_> = calamine::open_workbook_from_rs(cursor).map_err(|e| {
        tracing::error!("打开 Excel 文件字节流失败: {:?}", e);
        Error::BadRequest("无法识别的 Excel 文件格式".to_string())
    })?;

    let target_sheets = reader::resolve_target_sheets(&workbook.sheet_names(), T::sheet_name())?;

    let mut valid_rows = Vec::new();
    let mut errors = Vec::new();

    for sheet_name in target_sheets {
        let range = match workbook.worksheet_range(&sheet_name) {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("读取工作表 '{}' 失败: {:?}", sheet_name, e);
                continue;
            }
        };

        let mut rows_iter = range.rows();
        let Some(header_row) = rows_iter.next() else {
            continue;
        };

        let headers_map = reader::build_headers_map(header_row);

        for (row_idx, row_cells) in rows_iter.enumerate() {
            let line_number = row_idx + 2;

            if reader::is_empty_row(row_cells) {
                continue;
            }

            let excel_row = ExcelRow::new(line_number, &headers_map, row_cells);

            match T::from_row(&excel_row) {
                Ok(item) => valid_rows.push(item),
                Err(mut errs) => errors.append(&mut errs),
            }
        }
    }

    Ok(ImportResult { valid_rows, errors })
}
