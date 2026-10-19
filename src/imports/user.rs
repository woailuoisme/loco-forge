//! 用户 Excel 导入定义

use crate::{
    common::excel::{ExcelImport, ExcelRow, RowError},
    dtos::admin::UserImportRow,
};

/// 用户导入表头 (按列名匹配，顺序无关)
pub const USER_IMPORT_HEADINGS: &[&str] = &["邮箱", "名称", "角色", "用户名", "手机号"];

impl ExcelImport for UserImportRow {
    fn from_row(row: &ExcelRow) -> Result<Self, Vec<RowError>> {
        let email = row.get_string("邮箱");
        let name = row.get_string("名称");
        let mut errors = Vec::new();

        if email.trim().is_empty() {
            errors.push(RowError {
                line: row.line,
                column: "邮箱".to_string(),
                raw_value: email.clone(),
                message: "邮箱不能为空".to_string(),
            });
        }
        if name.trim().is_empty() {
            errors.push(RowError {
                line: row.line,
                column: "名称".to_string(),
                raw_value: name.clone(),
                message: "名称不能为空".to_string(),
            });
        }
        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(Self {
            email: email.trim().to_lowercase(),
            name: name.trim().to_string(),
            role: trim_opt(&row.get_string("角色")),
            username: trim_opt(&row.get_string("用户名")),
            phone_number: trim_opt(&row.get_string("手机号")),
        })
    }

    fn sheet_name() -> Option<&'static str> {
        Some("用户导入")
    }
}

/// 去除首尾空白，全空白视为 `None`
fn trim_opt(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}
