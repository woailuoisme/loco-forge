use loco_rs::prelude::*;
use rust_xlsxwriter::{Color, Format, FormatAlign, FormatBorder, Workbook};

/// Excel 单元格类型枚举 (支持原生数据类型，避免全量转为字符串，提升性能与 Excel 统计能力)
#[derive(Debug, Clone, PartialEq)]
pub enum CellValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Date(chrono::NaiveDate),
    DateTime(chrono::NaiveDateTime),
    Empty,
}

impl From<String> for CellValue {
    fn from(val: String) -> Self {
        Self::String(val)
    }
}

impl From<&str> for CellValue {
    fn from(val: &str) -> Self {
        Self::String(val.to_string())
    }
}

impl From<i32> for CellValue {
    fn from(val: i32) -> Self {
        Self::Int(val as i64)
    }
}

impl From<i64> for CellValue {
    fn from(val: i64) -> Self {
        Self::Int(val)
    }
}

impl From<u32> for CellValue {
    fn from(val: u32) -> Self {
        Self::Int(val as i64)
    }
}

impl From<u64> for CellValue {
    fn from(val: u64) -> Self {
        Self::Int(val as i64)
    }
}

impl From<f32> for CellValue {
    fn from(val: f32) -> Self {
        Self::Float(val as f64)
    }
}

impl From<f64> for CellValue {
    fn from(val: f64) -> Self {
        Self::Float(val)
    }
}

impl From<bool> for CellValue {
    fn from(val: bool) -> Self {
        Self::Bool(val)
    }
}

impl From<chrono::NaiveDate> for CellValue {
    fn from(val: chrono::NaiveDate) -> Self {
        Self::Date(val)
    }
}

impl From<chrono::NaiveDateTime> for CellValue {
    fn from(val: chrono::NaiveDateTime) -> Self {
        Self::DateTime(val)
    }
}

impl<T: Into<CellValue>> From<Option<T>> for CellValue {
    fn from(opt: Option<T>) -> Self {
        match opt {
            Some(v) => v.into(),
            None => Self::Empty,
        }
    }
}

/// Excel 导出接口（仿 Laravel Excel `FromCollection` / `WithHeadings` 规范）
pub trait ExcelExport {
    /// 定义 Excel 表头（列名列表）
    fn headings() -> Vec<String>;

    /// 将结构体转换为整行的单元格数据列表
    fn row_cells(&self) -> Vec<CellValue>;

    /// Excel 工作表名称（默认为 "Sheet1"）
    fn sheet_name() -> String {
        "Sheet1".to_string()
    }

    /// 自定义各列列宽（可选，例如 `Some(vec![15.0, 25.0, 30.0])`）
    /// 若返回 `None` 则自动启用自适应列宽 (autofit)
    fn column_widths() -> Option<Vec<f64>> {
        None
    }
}

/// 将实现了 `ExcelExport` 的结构体切片导出为内存中的 `.xlsx` 二进制字节数组
pub fn export_to_bytes<T: ExcelExport>(items: &[T]) -> Result<Vec<u8>, Error> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet.set_name(T::sheet_name()).map_err(|e| {
        tracing::error!("Excel 表名设置失败: {:?}", e);
        Error::InternalServerError
    })?;

    // 1. 样式定义
    let header_format = Format::new()
        .set_bold()
        .set_background_color(Color::RGB(0x2B579A)) // 经典 Word/Excel 深蓝色
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

    // 2. 写入表头 (Header Row)
    let headings = T::headings();
    for (col_idx, heading) in headings.iter().enumerate() {
        worksheet
            .write_string_with_format(0, col_idx as u16, heading, &header_format)
            .map_err(|e| {
                tracing::error!("写入 Excel 表头失败: {:?}", e);
                Error::InternalServerError
            })?;
    }

    // 3. 写入数据行 (Data Rows)
    for (row_offset, item) in items.iter().enumerate() {
        let row_idx = (row_offset + 1) as u32;
        let cells = item.row_cells();
        for (col_idx, cell_value) in cells.into_iter().enumerate() {
            let col = col_idx as u16;
            let res = match cell_value {
                CellValue::String(s) => worksheet.write_string_with_format(row_idx, col, &s, &data_format),
                CellValue::Int(n) => worksheet.write_number_with_format(row_idx, col, n as f64, &data_format),
                CellValue::Float(f) => worksheet.write_number_with_format(row_idx, col, f, &data_format),
                CellValue::Bool(b) => worksheet.write_boolean_with_format(row_idx, col, b, &data_format),
                CellValue::Date(d) => {
                    worksheet.write_string_with_format(row_idx, col, d.format("%Y-%m-%d").to_string(), &date_format)
                }
                CellValue::DateTime(dt) => worksheet.write_string_with_format(
                    row_idx,
                    col,
                    dt.format("%Y-%m-%d %H:%M:%S").to_string(),
                    &datetime_format,
                ),
                CellValue::Empty => worksheet.write_blank(row_idx, col, &data_format),
            };

            res.map_err(|e| {
                tracing::error!("写入 Excel 数据失败 [行 {}, 列 {}]: {:?}", row_idx, col, e);
                Error::InternalServerError
            })?;
        }
    }

    // 4. 列宽配置与自适应处理
    if let Some(widths) = T::column_widths() {
        for (col_idx, width) in widths.iter().enumerate() {
            worksheet.set_column_width(col_idx as u16, *width).map_err(|e| {
                tracing::error!("设置列宽失败: {:?}", e);
                Error::InternalServerError
            })?;
        }
    } else {
        worksheet.autofit();
    }

    // 5. 保存到内存字节流并返回
    workbook.save_to_buffer().map_err(|e| {
        tracing::error!("生成 Excel 字节流失败: {:?}", e);
        Error::InternalServerError
    })
}
