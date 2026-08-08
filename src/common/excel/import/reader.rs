//! Excel 读取驱动与单元格解构助手 (Based on Calamine)

use calamine::Data;
use loco_rs::prelude::*;
use std::collections::HashMap;

/// 零拷贝/零分配 Excel 单行数据读取上下文 (`ExcelRow`)
#[derive(Debug)]
pub struct ExcelRow<'a> {
    /// Excel 中的物理行号（从 1 开始计数，表头为 1，数据行从 2 开始）
    pub line: usize,
    headers: &'a HashMap<String, usize>,
    cells: &'a [Data],
}

impl<'a> ExcelRow<'a> {
    pub fn new(line: usize, headers: &'a HashMap<String, usize>, cells: &'a [Data]) -> Self {
        Self { line, headers, cells }
    }

    /// 获取指定列名的原始 `calamine::Data` 引用
    pub fn get_cell(&self, column_name: &str) -> Option<&'a Data> {
        let &col_idx = self.headers.get(column_name)?;
        self.cells.get(col_idx)
    }

    /// 读取单元格纯文本切片（零拷贝 &str）
    pub fn get_str(&self, column_name: &str) -> Option<&'a str> {
        match self.get_cell(column_name)? {
            Data::String(s) => Some(s.as_str()),
            Data::DateTimeIso(s) | Data::DurationIso(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// 读取单元格纯文本字符串（自动格式化数字/日期）
    pub fn get_string(&self, column_name: &str) -> String {
        match self.get_cell(column_name) {
            Some(cell) => format_cell_value(cell),
            None => String::new(),
        }
    }

    /// 读取 64 位整数
    pub fn get_i64(&self, column_name: &str) -> Option<i64> {
        match self.get_cell(column_name)? {
            Data::Int(i) => Some(*i),
            Data::Float(f) => Some(*f as i64),
            Data::String(s) => s.trim().parse::<i64>().ok(),
            _ => None,
        }
    }

    /// 读取 64 位浮点数
    pub fn get_f64(&self, column_name: &str) -> Option<f64> {
        match self.get_cell(column_name)? {
            Data::Float(f) => Some(*f),
            Data::Int(i) => Some(*i as f64),
            Data::String(s) => s.trim().parse::<f64>().ok(),
            _ => None,
        }
    }

    /// 读取布尔值
    pub fn get_bool(&self, column_name: &str) -> Option<bool> {
        match self.get_cell(column_name)? {
            Data::Bool(b) => Some(*b),
            Data::String(s) => match s.trim().to_lowercase().as_str() {
                "true" | "1" | "yes" | "是" => Some(true),
                "false" | "0" | "no" | "否" => Some(false),
                _ => None,
            },
            Data::Int(i) => Some(*i != 0),
            _ => None,
        }
    }
}

/// 辅助函数：根据表名匹配要解析的全部 Sheet (支持匹配 Sheet, Sheet_2, Sheet_3...)
pub fn resolve_target_sheets(
    all_sheet_names: &[String],
    specified_sheet_name: Option<&str>,
) -> Result<Vec<String>, Error> {
    if all_sheet_names.is_empty() {
        return Err(Error::BadRequest("Excel 文件中不存在任何工作表".to_string()));
    }

    match specified_sheet_name {
        Some(target) => {
            let matched: Vec<String> = all_sheet_names
                .iter()
                .filter(|&name| {
                    name == target || name.strip_prefix(target).is_some_and(|rest| rest.starts_with('_'))
                })
                .cloned()
                .collect();
            if matched.is_empty() {
                return Err(Error::BadRequest(format!("未找到工作表: {target}")));
            }
            Ok(matched)
        }
        None => Ok(all_sheet_names.to_vec()),
    }
}

/// 格式化 `calamine::Data` 为纯文本字符串
pub fn format_cell_value(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(s) => s.clone(),
        Data::Int(i) => i.to_string(),
        Data::Float(f) => f.to_string(),
        Data::Bool(b) => b.to_string(),
        Data::DateTime(dt) => dt.to_string(),
        Data::DateTimeIso(dt) => dt.clone(),
        Data::DurationIso(d) => d.clone(),
        Data::Error(e) => format!("ERROR: {e:?}"),
    }
}

/// 构建「表头名 → 列索引」映射（自动跳过空白表头单元格）
pub fn build_headers_map(header_row: &[Data]) -> HashMap<String, usize> {
    header_row
        .iter()
        .enumerate()
        .filter_map(|(col_idx, cell)| {
            let header_name = format_cell_value(cell).trim().to_string();
            (!header_name.is_empty()).then_some((header_name, col_idx))
        })
        .collect()
}

/// 判断整行是否全部为空单元格（空行应直接跳过）
pub fn is_empty_row(row_cells: &[Data]) -> bool {
    row_cells.iter().all(|cell| matches!(cell, Data::Empty))
}
