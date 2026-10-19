//! 用户资源 Excel 导出定义

use crate::{
    common::excel::{CellValue, ExcelExport},
    dtos::admin::AdminUserResource,
};

/// 用户导出表头 (与 [`AdminUserResource::row_cells`] 的列顺序严格对应)
pub const USER_EXPORT_HEADINGS: &[&str] = &[
    "ID",
    "名称",
    "邮箱",
    "邮箱已验证",
    "角色",
    "状态",
    "已封禁",
    "两步验证",
    "用户名",
    "手机号",
    "手机号已验证",
    "匿名用户",
    "最近登录方式",
    "封禁理由",
    "封禁到期",
    "最近登录时间",
    "创建时间",
    "更新时间",
    "注销时间",
];

impl ExcelExport for AdminUserResource {
    fn headings() -> Vec<&'static str> {
        USER_EXPORT_HEADINGS.to_vec()
    }

    fn row_cells(&self) -> Vec<CellValue> {
        vec![
            CellValue::from(self.id.clone()),
            CellValue::from(self.name.clone()),
            CellValue::from(self.email.clone()),
            CellValue::from(self.email_verified),
            CellValue::from(self.role.clone()),
            CellValue::from(self.status.clone()),
            CellValue::from(self.banned),
            CellValue::from(self.two_factor_enabled),
            CellValue::from(self.username.clone()),
            CellValue::from(self.phone_number.clone()),
            CellValue::from(self.phone_number_verified),
            CellValue::from(self.is_anonymous),
            CellValue::from(self.last_login_method.clone()),
            CellValue::from(self.ban_reason.clone()),
            CellValue::from(self.ban_expires.map(|d| d.naive_utc())),
            CellValue::from(self.last_login_at.map(|d| d.naive_utc())),
            CellValue::from(self.created_at.naive_utc()),
            CellValue::from(self.updated_at.naive_utc()),
            CellValue::from(self.deleted_at.map(|d| d.naive_utc())),
        ]
    }

    fn sheet_name() -> &'static str {
        "用户列表"
    }
}
