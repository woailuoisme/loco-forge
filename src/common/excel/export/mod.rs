//! Excel 导出主接口定义与字节流内存生成器

pub mod chunk;
pub mod engine;

pub use chunk::export_chunked_to_file;
pub use engine::ExportStyles;

use loco_rs::prelude::*;
use rust_xlsxwriter::Workbook;

use crate::common::excel::cell::CellValue;

/// Excel 导出接口（仿 Laravel Excel `FromCollection` / `WithHeadings` 规范）
pub trait ExcelExport {
    /// 定义 Excel 表头（列名列表，零堆分配字符串切片）
    fn headings() -> Vec<&'static str>;

    /// 将结构体转换为整行的单元格数据列表
    fn row_cells(&self) -> Vec<CellValue>;

    /// Excel 工作表名称（默认为 "Sheet1"）
    fn sheet_name() -> &'static str {
        "Sheet1"
    }

    /// 自定义各列列宽（可选，例如 `Some(vec![15.0, 25.0, 30.0])`）
    /// 若返回 `None` 则自动启用自适应列宽 (autofit)
    fn column_widths() -> Option<Vec<f64>> {
        None
    }
}

/// 导出配置选项（针对海量数据流式处理）
#[derive(Debug, Clone)]
pub struct ExportOptions {
    /// 每一批拉取的数据行数（预设 1,000 行）
    pub chunk_size: usize,
    /// 单个 Sheet 允许写入的最大数据行数（默认 1,000,000 行，达到阈值自动创建 Sheet2, Sheet3...）
    pub max_rows_per_sheet: usize,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            chunk_size: 1000,
            max_rows_per_sheet: 1_000_000,
        }
    }
}

/// 将实现了 `ExcelExport` 的结构体切片导出为内存中的 `.xlsx` 二进制字节数组（适合小数据量导出）
pub fn export_to_bytes<T: ExcelExport>(items: &[T]) -> Result<Vec<u8>, Error> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    let _ = worksheet.set_name(T::sheet_name());

    let styles = engine::ExportStyles::default();

    // 1. 写入表头 (Header Row)
    let headings = T::headings();
    for (col_idx, heading) in headings.iter().enumerate() {
        worksheet
            .write_string_with_format(0, col_idx as u16, *heading, &styles.header_format)
            .map_err(|e| {
                tracing::error!("写入 Excel 表头失败: {:?}", e);
                Error::InternalServerError
            })?;
    }

    // 2. 写入数据行 (Data Rows)
    for (row_offset, item) in items.iter().enumerate() {
        let row_idx = (row_offset + 1) as u32;
        let cells = item.row_cells();
        for (col_idx, cell_value) in cells.into_iter().enumerate() {
            engine::write_cell(worksheet, row_idx, col_idx as u16, cell_value, &styles)?;
        }
    }

    // 3. 列宽配置与自适应处理
    if let Some(widths) = T::column_widths() {
        for (col_idx, width) in widths.iter().enumerate() {
            let _ = worksheet.set_column_width(col_idx as u16, *width);
        }
    } else {
        worksheet.autofit();
    }

    // 4. 保存到内存字节流并返回
    workbook.save_to_buffer().map_err(|e| {
        tracing::error!("生成 Excel 字节流失败: {:?}", e);
        Error::InternalServerError
    })
}
