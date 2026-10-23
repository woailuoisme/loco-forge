//! 管理端用户管理 DTO (Admin User DTOs)

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

use crate::dtos::common::PaginationRequest;

/// PATCH 三态语义下的可置空字符串字段。
///
/// 外层配合 `Option<NullableString>` 与 `deserialize_nullable` 使用：
/// - 字段**缺省** → `None`，保持原值不变
/// - 显式 `null` → `Some(NullableString::Null)`，置空
/// - 具体值 → `Some(NullableString::Value(v))`，覆盖
///
/// 用显式枚举替代 `Option<Option<T>>`，以通过项目的 `clippy::option_option` 检查。
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(untagged)]
pub enum NullableString {
    /// 传具体值，覆盖该字段
    Value(String),
    /// 显式传 `null`，置空该字段
    Null,
}

impl NullableString {
    /// 取出内部值；`Null` 返回 `None`
    #[must_use]
    pub fn into_option(self) -> Option<String> {
        match self {
            Self::Null => None,
            Self::Value(v) => Some(v),
        }
    }
}

/// 反序列化可空字段。
///
/// **必要性**：serde 对 `Option<T>` 会把 JSON `null` 直接吸收为 `None`，
/// 导致「字段缺省」与「显式 null」无法区分。此处用 `deserialize_with`
/// 绕过 `Option` 的默认实现，使 `null` 落到 [`NullableString::Null`]。
fn deserialize_nullable<'de, D>(deserializer: D) -> Result<Option<NullableString>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    NullableString::deserialize(deserializer).map(Some)
}

/// 管理端用户列表查询参数
///
/// 查询串示例：`?page=1&pageSize=20&role=admin&status=active&search=foo&sortBy=createdAt`
#[derive(Debug, Clone, Serialize, Deserialize, Validate, IntoParams, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminListUsersQuery {
    /// 页码 (从 1 开始，默认 1)
    pub page: Option<u64>,
    /// 每页数量 (默认 20，最大 100)
    pub page_size: Option<u64>,
    /// 模糊搜索关键词 (匹配 email / name / username)
    pub search: Option<String>,
    /// 角色精确过滤 (如 `admin` / `user`)
    pub role: Option<String>,
    /// 状态精确过滤 (`active` | `inactive` | `banned`)
    pub status: Option<String>,
    /// 封禁状态过滤
    pub banned: Option<bool>,
    /// 排序字段 (`createdAt` | `updatedAt` | `email` | `name` | `lastLoginAt`；默认 `createdAt` 倒序)
    pub sort_by: Option<String>,
    /// 是否包含已注销 (软删) 用户，默认 `false`
    #[serde(default)]
    pub include_deleted: bool,
}

impl AdminListUsersQuery {
    /// 提取并归一化分页参数
    #[must_use]
    pub fn pagination(&self) -> PaginationRequest {
        PaginationRequest::new(self.page, self.page_size)
    }
}

/// 管理端创建用户请求
///
/// 创建的用户**无密码凭证**，需由用户本人通过「忘记密码」或 Magic Link 完成首次登录。
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminCreateUserRequest {
    /// 邮箱 (全局唯一，大小写无关)
    #[validate(email(message = "邮箱格式无效"), length(max = 255))]
    pub email: String,
    /// 显示名称
    #[validate(length(min = 1, max = 100, message = "名称长度需在 1~100 字符之间"))]
    pub name: String,
    /// 全局角色 (默认 `user`)
    #[validate(length(max = 50))]
    pub role: Option<String>,
    /// 用户名 (可选，全局唯一，大小写无关)
    #[validate(length(min = 3, max = 50))]
    pub username: Option<String>,
    /// 手机号 (可选，全局唯一)
    #[validate(length(max = 50))]
    pub phone_number: Option<String>,
    /// 头像 URL
    pub image: Option<String>,
}

/// 管理端更新用户请求 (PATCH 局部更新)
///
/// 遵循 `api.md` §2.7 三态语义：字段缺省保持原值；显式 `null` 置空；传值则覆盖。
/// `email` 不在可编辑范围——变更邮箱需走用户侧的验证流程。
#[derive(Debug, Clone, Default, Serialize, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminUpdateUserRequest {
    /// 显示名称
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
    /// 全局角色
    #[validate(length(max = 50))]
    pub role: Option<String>,
    /// 账号状态 (`active` | `inactive` | `banned`)
    #[validate(length(max = 20))]
    pub status: Option<String>,
    /// 用户名 (三态：缺省不变 / `null` 置空 / 传值覆盖)
    #[serde(default, deserialize_with = "deserialize_nullable")]
    pub username: Option<NullableString>,
    /// 头像 URL (三态)
    #[serde(default, deserialize_with = "deserialize_nullable")]
    pub image: Option<NullableString>,
    /// 邮箱验证状态
    pub email_verified: Option<bool>,
}

/// 管理端封禁用户请求体 (目标用户由路径参数 `:id` 指定)
#[derive(Debug, Clone, Default, Serialize, Deserialize, Validate, ToSchema)]
#[schema(as = dtos::admin::AdminBanUserRequest)]
#[serde(rename_all = "camelCase")]
pub struct AdminBanUserRequest {
    /// 封禁理由
    #[validate(length(max = 500))]
    pub ban_reason: Option<String>,
    /// 封禁时长 (秒)；缺省表示永久封禁
    pub ban_expires_in: Option<i64>,
}

/// 管理端用户资源 (含全部运维字段)
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminUserResource {
    /// 用户主键 (UUIDv7)
    pub id: String,
    /// 显示名称
    pub name: String,
    /// 邮箱
    pub email: String,
    /// 邮箱是否已验证
    pub email_verified: bool,
    /// 头像 URL
    pub image: Option<String>,
    /// 全局角色
    pub role: String,
    /// 账号状态 (`active` | `inactive` | `banned`)
    pub status: String,
    /// 是否被封禁
    pub banned: bool,
    /// 是否启用两步验证
    pub two_factor_enabled: bool,
    /// 用户名
    pub username: Option<String>,
    /// 用户名展示形态 (保留原始大小写)
    pub display_username: Option<String>,
    /// 手机号
    pub phone_number: Option<String>,
    /// 手机号是否已验证
    pub phone_number_verified: bool,
    /// 是否为匿名用户
    pub is_anonymous: bool,
    /// 最近登录方式
    pub last_login_method: Option<String>,
    /// 封禁理由
    pub ban_reason: Option<String>,
    /// 封禁到期时间
    #[ts(type = "string | null")]
    pub ban_expires: Option<chrono::DateTime<chrono::FixedOffset>>,
    /// 最近登录时间
    #[ts(type = "string | null")]
    pub last_login_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    /// 创建时间
    #[ts(type = "string")]
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    /// 更新时间
    #[ts(type = "string")]
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
    /// 注销 (软删) 时间，`null` 表示未注销
    #[ts(type = "string | null")]
    pub deleted_at: Option<chrono::DateTime<chrono::FixedOffset>>,
}

impl From<crate::models::user::Model> for AdminUserResource {
    fn from(u: crate::models::user::Model) -> Self {
        Self {
            id: u.id,
            name: u.name,
            email: u.email,
            email_verified: u.email_verified,
            image: u.image,
            role: u.role,
            status: u.status,
            banned: u.banned,
            two_factor_enabled: u.two_factor_enabled,
            username: u.username,
            display_username: u.display_username,
            phone_number: u.phone_number,
            phone_number_verified: u.phone_number_verified,
            is_anonymous: u.is_anonymous,
            last_login_method: u.last_login_method,
            ban_reason: u.ban_reason,
            ban_expires: u.ban_expires,
            last_login_at: u.last_login_at,
            created_at: u.created_at,
            updated_at: u.updated_at,
            deleted_at: u.deleted_at,
        }
    }
}

/// Excel 导入行 (仅创建语义：email 已存在时计入失败行)
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserImportRow {
    /// 邮箱 (唯一键，用于判定冲突)
    #[validate(email(message = "邮箱格式无效"))]
    pub email: String,
    /// 显示名称
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    /// 全局角色
    pub role: Option<String>,
    /// 用户名
    pub username: Option<String>,
    /// 手机号
    pub phone_number: Option<String>,
}

/// Excel 导入结果摘要
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserImportSummary {
    /// 成功创建的用户数
    pub created: u64,
    /// 失败行数
    pub failed: u64,
    /// 失败明细
    pub errors: Vec<UserImportError>,
}

/// Excel 导入失败明细
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserImportError {
    /// 行号 (从 1 开始，含表头行时以实际 Excel 行号为准)
    pub line: u64,
    /// 列名
    pub column: String,
    /// 原始单元格值
    pub raw_value: String,
    /// 失败原因
    pub message: String,
}
