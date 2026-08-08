//! 海量数据大文件分块流式导出实现 (Zip 内存池 & 多 Sheet 自动平滑拆分)

use loco_rs::prelude::*;
use rust_xlsxwriter::Workbook;

use super::{ExcelExport, ExportOptions, engine};

/// 流式导出海量数据（支持 100 万+ 行）到物理/临时 `.xlsx` 文件
///
/// # 特性
/// 1. **游标分批 Fetch**：传入 `fetch_chunk(page, page_size)` 闭包，避免全量数据驻留内存。
/// 2. **低内存流式模式**：使用 `worksheet.add_worksheet_with_constant_memory()` 常数级内存写入，数据流式落盘，内存占用维持在几 MB 级别。
/// 3. **自动分 Sheet 规避 104 万行上限**：当数据行数达到 `max_rows_per_sheet`（默认 100 万），自动平滑无缝创建 `Sheet2`, `Sheet3`...
///
/// # 返回
/// - `Ok(total_rows)`: 成功导出的数据总行数
pub fn export_chunked_to_file<T, F>(
    mut fetch_chunk: F,
    file_path: impl AsRef<std::path::Path>,
    options: ExportOptions,
) -> Result<usize, Error>
where
    T: ExcelExport,
    F: FnMut(usize, usize) -> Result<Vec<T>, Error>,
{
    let mut workbook = Workbook::new();
    let styles = engine::ExportStyles::default();

    let base_sheet_name = T::sheet_name();
    let headings = T::headings();

    let mut sheet_count = 1usize;
    let mut total_exported_rows = 0usize;
    let mut current_sheet_row_idx = 0u32;

    let mut current_worksheet = engine::init_worksheet(
        &mut workbook,
        sheet_count,
        base_sheet_name,
        &headings,
        &styles.header_format,
        T::column_widths().as_deref(),
    )?;

    let mut page = 1usize;
    loop {
        let chunk = fetch_chunk(page, options.chunk_size)?;
        if chunk.is_empty() {
            break;
        }

        for item in chunk {
            // 判断是否需要自动分 Sheet
            if current_sheet_row_idx >= options.max_rows_per_sheet as u32 {
                sheet_count += 1;
                current_worksheet = engine::init_worksheet(
                    &mut workbook,
                    sheet_count,
                    base_sheet_name,
                    &headings,
                    &styles.header_format,
                    T::column_widths().as_deref(),
                )?;
                current_sheet_row_idx = 0;
            }

            current_sheet_row_idx += 1;
            total_exported_rows += 1;

            let row_idx = current_sheet_row_idx;
            let cells = item.row_cells();

            for (col_idx, cell_value) in cells.into_iter().enumerate() {
                engine::write_cell(current_worksheet, row_idx, col_idx as u16, cell_value, &styles)?;
            }
        }

        page += 1;
    }

    workbook.save(file_path).map_err(|e| {
        tracing::error!("保存 Excel 流式文件失败: {:?}", e);
        Error::InternalServerError
    })?;

    Ok(total_exported_rows)
}
