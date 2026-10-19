//! 管理端用户接口 (Admin User Endpoints)
//!
//! 全部端点要求管理员身份 ([`AdminUser`] 提取器)，路径挂载于 `/v1/admin/users`。

use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use loco_rs::prelude::*;

use crate::{
    common::consts::openapi::TAG_ADMIN,
    common::excel::ExcelResponse,
    dtos::common::format_res,
    dtos::{
        admin::{
            AdminCreateUserRequest, AdminListUsersQuery, AdminUpdateUserRequest, AdminUserResource,
        },
        auth::{AdminBanUserRequest, AdminUnbanUserRequest},
        common::{AdminUser, ValidJson},
    },
    errors::AppResult,
    services::{AdminUserService, AuthAdminService},
};

/// 分页检索用户列表
#[utoipa::path(
    get,
    path = "/v1/admin/users",
    summary = "分页检索用户",
    description = "支持关键词搜索、角色 / 状态 / 封禁过滤与排序",
    params(AdminListUsersQuery),
    responses(
        (status = 200, description = "查询成功"),
        (status = 401, description = "未授权"),
        (status = 403, description = "需要管理员权限")
    ),
    security(("bearer_auth" = [])),
    tag = TAG_ADMIN
)]
#[debug_handler]
pub async fn list(
    State(ctx): State<AppContext>,
    _admin: AdminUser,
    Query(query): Query<AdminListUsersQuery>,
) -> AppResult<Response> {
    let res = AdminUserService::list(&ctx.db, &query).await?;
    Ok(format_res::ok(res)?)
}

/// 查询用户详情
#[utoipa::path(
    get,
    path = "/v1/admin/users/{id}",
    summary = "用户详情",
    description = "includeDeleted=true 时可查看已注销用户",
    params(
        ("id" = String, Path, description = "用户 ID"),
        ("includeDeleted" = Option<bool>, Query, description = "是否包含已注销用户")
    ),
    responses(
        (status = 200, description = "查询成功", body = AdminUserResource),
        (status = 404, description = "用户不存在")
    ),
    security(("bearer_auth" = [])),
    tag = TAG_ADMIN
)]
#[debug_handler]
pub async fn get_one(
    State(ctx): State<AppContext>,
    _admin: AdminUser,
    Path(id): Path<String>,
    Query(query): Query<IncludeDeletedQuery>,
) -> AppResult<Response> {
    let res = AdminUserService::get_one(&ctx.db, &id, query.include_deleted).await?;
    Ok(format_res::ok(res)?)
}

/// 查询串：是否包含已注销用户
#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct IncludeDeletedQuery {
    /// 是否包含已注销用户，默认 false
    #[serde(default)]
    pub include_deleted: bool,
}

/// 创建用户
#[utoipa::path(
    post,
    path = "/v1/admin/users",
    summary = "创建用户",
    description = "创建无密码用户，本人需经忘记密码或 Magic Link 首次登录",
    request_body = AdminCreateUserRequest,
    responses(
        (status = 201, description = "创建成功", body = AdminUserResource),
        (status = 409, description = "邮箱 / 用户名 / 手机号已被占用"),
        (status = 422, description = "参数校验失败")
    ),
    security(("bearer_auth" = [])),
    tag = TAG_ADMIN
)]
#[debug_handler]
pub async fn create(
    State(ctx): State<AppContext>,
    _admin: AdminUser,
    ValidJson(params): ValidJson<AdminCreateUserRequest>,
) -> AppResult<Response> {
    let res = AdminUserService::create(&ctx.db, &params).await?;
    Ok(format_res::created(res)?)
}

/// 局部更新用户
#[utoipa::path(
    patch,
    path = "/v1/admin/users/{id}",
    summary = "更新用户",
    description = "局部更新；字段缺省保持原值，显式 null 置空",
    params(("id" = String, Path, description = "用户 ID")),
    request_body = AdminUpdateUserRequest,
    responses(
        (status = 200, description = "更新成功", body = AdminUserResource),
        (status = 404, description = "用户不存在"),
        (status = 409, description = "用户名已被占用")
    ),
    security(("bearer_auth" = [])),
    tag = TAG_ADMIN
)]
#[debug_handler]
pub async fn update(
    State(ctx): State<AppContext>,
    _admin: AdminUser,
    Path(id): Path<String>,
    ValidJson(params): ValidJson<AdminUpdateUserRequest>,
) -> AppResult<Response> {
    let res = AdminUserService::update(&ctx.db, &id, &params).await?;
    Ok(format_res::ok(res)?)
}

/// 注销用户 (软删除)
#[utoipa::path(
    delete,
    path = "/v1/admin/users/{id}",
    summary = "注销用户",
    description = "软删并撤销全部会话；数据保留，邮箱 / 用户名 / 手机号随即释放",
    params(("id" = String, Path, description = "用户 ID")),
    responses(
        (status = 204, description = "注销成功"),
        (status = 400, description = "不能注销自己"),
        (status = 404, description = "用户不存在")
    ),
    security(("bearer_auth" = [])),
    tag = TAG_ADMIN
)]
#[debug_handler]
pub async fn soft_delete(
    State(ctx): State<AppContext>,
    admin: AdminUser,
    Path(id): Path<String>,
) -> AppResult<Response> {
    AdminUserService::soft_delete(&ctx.db, &admin.0.id, &id).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

/// 封禁用户
#[utoipa::path(
    post,
    path = "/v1/admin/users/{id}/bans",
    summary = "封禁用户",
    description = "封禁并立即撤销该用户全部活跃会话",
    params(("id" = String, Path, description = "用户 ID")),
    request_body = crate::dtos::admin::AdminBanUserRequest,
    responses(
        (status = 200, description = "封禁成功", body = AdminUserResource),
        (status = 400, description = "不能封禁自己"),
        (status = 404, description = "用户不存在")
    ),
    security(("bearer_auth" = [])),
    tag = TAG_ADMIN
)]
#[debug_handler]
pub async fn ban(
    State(ctx): State<AppContext>,
    admin: AdminUser,
    Path(id): Path<String>,
    ValidJson(body): ValidJson<crate::dtos::admin::AdminBanUserRequest>,
) -> AppResult<Response> {
    let params = AdminBanUserRequest {
        user_id: id,
        ban_reason: body.ban_reason,
        ban_expires_in: body.ban_expires_in,
    };
    // 复用认证域的实现执行封禁 (含撤会话)，随后按管理端完整投影重新读取，
    // 保证与 GET /v1/admin/users/{id} 的响应结构一致。
    let banned = AuthAdminService::admin_ban_user(&ctx.db, &admin.0, &params).await?;
    let res = AdminUserService::get_one(&ctx.db, &banned.id, false).await?;
    Ok(format_res::ok(res)?)
}

/// 解封用户
#[utoipa::path(
    delete,
    path = "/v1/admin/users/{id}/bans",
    summary = "解封用户",
    description = "清除用户的封禁状态与到期时间",
    params(("id" = String, Path, description = "用户 ID")),
    responses(
        (status = 200, description = "解封成功", body = AdminUserResource),
        (status = 404, description = "用户不存在")
    ),
    security(("bearer_auth" = [])),
    tag = TAG_ADMIN
)]
#[debug_handler]
pub async fn unban(
    State(ctx): State<AppContext>,
    admin: AdminUser,
    Path(id): Path<String>,
) -> AppResult<Response> {
    let params = AdminUnbanUserRequest { user_id: id };
    let unbanned = AuthAdminService::admin_unban_user(&ctx.db, &admin.0, &params).await?;
    let res = AdminUserService::get_one(&ctx.db, &unbanned.id, false).await?;
    Ok(format_res::ok(res)?)
}

/// 导出用户列表为 Excel
#[utoipa::path(
    get,
    path = "/v1/admin/users/export",
    summary = "导出用户",
    description = "流式导出为 xlsx，支持海量数据自动分 Sheet",
    params(AdminListUsersQuery),
    responses(
        (status = 200, description = "导出成功 (xlsx 文件流)"),
        (status = 401, description = "未授权"),
        (status = 403, description = "需要管理员权限")
    ),
    security(("bearer_auth" = [])),
    tag = TAG_ADMIN
)]
#[debug_handler]
pub async fn export(
    State(ctx): State<AppContext>,
    _admin: AdminUser,
    Query(query): Query<AdminListUsersQuery>,
) -> AppResult<ExcelResponse> {
    let (path, rows) = AdminUserService::export_excel(&ctx.db, &query).await?;
    tracing::info!(rows, path = %path.display(), "管理端导出用户 Excel");
    let filename = format!("users-{}.xlsx", chrono::Utc::now().format("%Y%m%d-%H%M%S"));
    // 流式回传临时文件；文件由进程退出或系统临时目录策略回收
    Ok(ExcelResponse::download_file(path, filename))
}

/// 从 Excel 批量导入用户
#[utoipa::path(
    post,
    path = "/v1/admin/users/import",
    summary = "导入用户",
    description = "multipart 上传 xlsx；仅创建，邮箱已存在等冲突计入失败行",
    responses(
        (status = 200, description = "导入完成", body = crate::dtos::admin::UserImportSummary),
        (status = 400, description = "文件缺失或格式错误")
    ),
    security(("bearer_auth" = [])),
    tag = TAG_ADMIN
)]
#[debug_handler]
pub async fn import(
    State(ctx): State<AppContext>,
    _admin: AdminUser,
    mut multipart: Multipart,
) -> AppResult<Response> {
    let mut file_bytes: Option<bytes::Bytes> = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| crate::errors::AppError::bad_request(format!("解析上传表单失败: {e}")))?
    {
        if field.name() == Some("file") {
            let data = field
                .bytes()
                .await
                .map_err(|e| crate::errors::AppError::bad_request(format!("读取文件失败: {e}")))?;
            file_bytes = Some(data);
            break;
        }
    }

    let bytes = file_bytes.ok_or_else(|| crate::errors::AppError::bad_request("缺少 file 字段"))?;
    let summary = AdminUserService::import_excel(&ctx.db, bytes).await?;
    Ok(format_res::ok(summary)?)
}
