//! Excel 单元格通用数据类型与其核心转换 Trait

/// 单元格数据类型定义
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

impl<T: Into<Self>> From<Option<T>> for CellValue {
    fn from(opt: Option<T>) -> Self {
        opt.map_or(Self::Empty, Into::into)
    }
}
