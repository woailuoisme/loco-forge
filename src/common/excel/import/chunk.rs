//! 海量数据分批流式导入与错误熔断机制实现

use calamine::{Reader, Xlsx};
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};

use super::{ExcelImport, ImportOptions, RowError, reader};

/// 分批流式导入执行结果汇总
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkImportSummary {
    /// 成功解析并通过校验处理的总数据行数
    pub total_valid_rows: usize,
    /// 解析失败或校验不通过的错误明细集合
    pub errors: Vec<RowError>,
}

impl ChunkImportSummary {
    /// 是否存在校验或解析错误
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

/// 针对海量数据（支持 100 万+ 行）的高性能 Chunk 分批回调导入 API
///
/// # 特性
/// 1. **分批回调落盘/写库**：每满 `options.chunk_size`（默认 1000 行）自动触发 `on_chunk_cb(&chunk_items)`，写完立即 drop 清空当前批次内存。
/// 2. **错误熔断机制**：当解析/校验错误数累积达到 `options.max_errors`（默认 100 条）时，立即中止后续解析并提前返回错误集合，避免几百万条错误压爆内存。
/// 3. **零分配 ExcelRow 读取**：每一行解构过程零额外堆内存分配。
/// 4. **多 Sheet 无缝跨表连续解析**：自动顺序解析按 `Sheet`, `Sheet_2`, `Sheet_3` 拆分的海量数据表。
pub fn import_in_chunks_from_reader<R, T, F>(
    reader: R,
    options: ImportOptions,
    mut on_chunk_cb: F,
) -> Result<ChunkImportSummary, Error>
where
    R: std::io::Read + std::io::Seek,
    T: ExcelImport,
    F: FnMut(&[T]) -> Result<(), Error>,
{
    let mut workbook: Xlsx<_> = calamine::open_workbook_from_rs(reader).map_err(|e| {
        tracing::error!("打开 Excel Reader 失败: {:?}", e);
        Error::BadRequest("无法识别的 Excel 文件格式".to_string())
    })?;

    let target_sheets = reader::resolve_target_sheets(&workbook.sheet_names(), T::sheet_name())?;

    let mut chunk_buffer = Vec::with_capacity(options.chunk_size);
    let mut errors = Vec::new();
    let mut total_valid_rows = 0usize;
    let mut should_stop = false;

    for sheet_name in target_sheets {
        if should_stop {
            break;
        }

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
            if errors.len() >= options.max_errors {
                tracing::warn!(
                    "Excel 导入校验错误已累积达到熔断上限 ({})，提前中止后续解析",
                    options.max_errors
                );
                should_stop = true;
                break;
            }

            let line_number = row_idx + 2;

            if reader::is_empty_row(row_cells) {
                continue;
            }

            let excel_row = reader::ExcelRow::new(line_number, &headers_map, row_cells);

            match T::from_row(&excel_row) {
                Ok(item) => {
                    chunk_buffer.push(item);
                    total_valid_rows += 1;
                }
                Err(mut errs) => errors.append(&mut errs),
            }

            if chunk_buffer.len() >= options.chunk_size {
                on_chunk_cb(&chunk_buffer)?;
                chunk_buffer.clear();
            }
        }
    }

    if !chunk_buffer.is_empty() {
        on_chunk_cb(&chunk_buffer)?;
        chunk_buffer.clear();
    }

    Ok(ChunkImportSummary {
        total_valid_rows,
        errors,
    })
}
