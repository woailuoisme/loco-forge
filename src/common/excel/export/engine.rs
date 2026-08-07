//! Excel 导出核心渲染引擎（样式设置、表头格式化、单元格写入）

use loco_rs::prelude::*;
use rust_xlsxwriter::{Color, Format, FormatAlign, FormatBorder, Workbook, Worksheet};

use crate::common::excel::cell::CellValue;

/// 框架内置标准表格样式定义
pub struct ExportStyles {
    pub header_format: Format,
    pub data_format: Format,
    pub date_format: Format,
    pub datetime_format: Format,
}

impl Default for ExportStyles {
    fn default() -> Self {
        let header_format = Format::new()
            .set_bold()
            .set_background_color(Color::RGB(0x2B579A))
            .set_font_color(Color::White)
            .set_align(FormatAlign::Center)
            .set_border(FormatBorder::Thin)
            .set_border_color(Color::RGB(0xD9D9D9));

        let data_format = Format::new()
            .set_align(FormatAlign::Left)
            .set_border(FormatBorder::Thin)
            .set_border_color(Color::RGB(0xE0E0E0));

        let date_format = Format::new()
            .set_align(FormatAlign::Center)
            .set_border(FormatBorder::Thin)
            .set_border_color(Color::RGB(0xE0E0E0))
            .set_num_format("yyyy-mm-dd");

        let datetime_format = Format::new()
            .set_align(FormatAlign::Center)
            .set_border(FormatBorder::Thin)
            .set_border_color(Color::RGB(0xE0E0E0))
            .set_num_format("yyyy-mm-dd hh:mm:ss");

        Self {
            header_format,
            data_format,
            date_format,
            datetime_format,
        }
    }
}

/// 辅助函数：创建并初始化新的 Worksheet（常数级低内存模式）
pub fn init_worksheet<'a>(
    wb: &'a mut Workbook,
    idx: usize,
    base_sheet_name: &str,
    headings: &[&'static str],
    header_format: &Format,
    column_widths: Option<&[f64]>,
) -> Result<&'a mut Worksheet, Error> {
    let worksheet = wb.add_worksheet_with_constant_memory();

    let name = if idx == 1 {
        base_sheet_name.to_string()
    } else {
        format!("{base_sheet_name}_{idx}")
    };

    let _ = worksheet.set_name(&name);

    // 写入表头 (Header Row)
    for (col_idx, heading) in headings.iter().enumerate() {
        worksheet
            .write_string_with_format(0, col_idx as u16, *heading, header_format)
            .map_err(|e| {
                tracing::error!("写入 Excel 表头失败: {:?}", e);
                Error::InternalServerError
            })?;
    }

    // 配置自定义列宽
    if let Some(widths) = column_widths {
        for (col_idx, width) in widths.iter().enumerate() {
            let _ = worksheet.set_column_width(col_idx as u16, *width);
        }
    }

    Ok(worksheet)
}

/// 将单个单元格数据写入指定的 Worksheet 坐标
pub fn write_cell(
    worksheet: &mut Worksheet,
    row_idx: u32,
    col_idx: u16,
    cell_value: CellValue,
    styles: &ExportStyles,
) -> Result<(), Error> {
    let res = match cell_value {
        CellValue::String(s) => worksheet.write_string_with_format(row_idx, col_idx, &s, &styles.data_format),
        CellValue::Int(n) => worksheet.write_number_with_format(row_idx, col_idx, n as f64, &styles.data_format),
        CellValue::Float(f) => worksheet.write_number_with_format(row_idx, col_idx, f, &styles.data_format),
        CellValue::Bool(b) => worksheet.write_boolean_with_format(row_idx, col_idx, b, &styles.data_format),
        CellValue::Date(d) => {
            worksheet.write_string_with_format(row_idx, col_idx, d.format("%Y-%m-%d").to_string(), &styles.date_format)
        }
        CellValue::DateTime(dt) => worksheet.write_string_with_format(
            row_idx,
            col_idx,
            dt.format("%Y-%m-%d %H:%M:%S").to_string(),
            &styles.datetime_format,
        ),
        CellValue::Empty => worksheet.write_blank(row_idx, col_idx, &styles.data_format),
    };

    res.map(|_| ()).map_err(|e| {
        tracing::error!("写入 Excel 数据失败 [行 {}, 列 {}]: {:?}", row_idx, col_idx, e);
        Error::InternalServerError
    })
}
