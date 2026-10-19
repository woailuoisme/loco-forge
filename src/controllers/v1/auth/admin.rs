use axum::{
    Json,
    extract::{Query, State},
    http::{HeaderMap, StatusCode, header::SET_COOKIE},
    response::{IntoResponse, Response},
};
use loco_rs::prelude::*;

use crate::{
    common::consts::openapi::TAG_ADMIN,
    dtos::{
        auth::*,
        common::{ApiError, PaginatedResource, ValidJson, format_res},
    },
    errors::AppResult,
    services::AuthAdminService,
};

// ─────────────────────────────────────────────────────────────────────────────
// Cookie 与 Token 辅助函数
// ─────────────────────────────────────────────────────────────────────────────

use super::shared::*;

// ─────────────────────────────────────────────────────────────────────────────
// 超级管理员服务 (Admin Plugin Handlers)
// ─────────────────────────────────────────────────────────────────────────────

/// 管理员设置用户平台角色
#[utoipa::path(
    patch,
    path = "/v1/admin/user-roles",
    summary = "设置用户全局角色",
    description = "修改用户全局角色",
    request_body = AdminSetRoleRequest,
    responses(
        (status = 200, description = "修改成功", body = UserResource),
        (status = 401, description = "未登录", body = ApiError),
        (status = 403, description = "非管理员权限拒绝", body = ApiError),
        (status = 404, description = "目标用户不存在", body = ApiError)
    ),
    tag = TAG_ADMIN
)]
#[debug_handler]
pub async fn admin_set_role(
    State(ctx): State<AppContext>,
    auth: AuthSession,
    ValidJson(params): ValidJson<AdminSetRoleRequest>,
) -> AppResult<Response> {
    let res = AuthAdminService::admin_set_role(&ctx.db, &auth.user, &params).await?;
    Ok(format_res::ok(res)?)
}

/// 管理员封禁用户
#[utoipa::path(
    patch,
    path = "/v1/admin/user-bans",
    summary = "封禁指定用户",
    description = "封禁用户并撤销其会话",
    request_body = AdminBanUserRequest,
    responses(
        (status = 200, description = "封禁成功", body = UserResource),
        (status = 400, description = "禁止封禁自己", body = ApiError),
        (status = 401, description = "未登录", body = ApiError),
        (status = 403, description = "非管理员权限拒绝", body = ApiError),
        (status = 404, description = "目标用户不存在", body = ApiError)
    ),
    tag = TAG_ADMIN
)]
#[debug_handler]
pub async fn admin_ban_user(
    State(ctx): State<AppContext>,
    auth: AuthSession,
    ValidJson(params): ValidJson<AdminBanUserRequest>,
) -> AppResult<Response> {
    let res = AuthAdminService::admin_ban_user(&ctx.db, &auth.user, &params).await?;
    Ok(format_res::ok(res)?)
}

/// 管理员解除用户封禁
#[utoipa::path(
    delete,
    path = "/v1/admin/user-bans",
    summary = "解封指定用户",
    description = "解除用户封禁",
    request_body = AdminUnbanUserRequest,
    responses(
        (status = 200, description = "解封成功", body = UserResource),
        (status = 401, description = "未登录", body = ApiError),
        (status = 403, description = "非管理员权限拒绝", body = ApiError),
        (status = 404, description = "目标用户不存在", body = ApiError)
    ),
    tag = TAG_ADMIN
)]
#[debug_handler]
pub async fn admin_unban_user(
    State(ctx): State<AppContext>,
    auth: AuthSession,
    ValidJson(params): ValidJson<AdminUnbanUserRequest>,
) -> AppResult<Response> {
    let res = AuthAdminService::admin_unban_user(&ctx.db, &auth.user, &params).await?;
    Ok(format_res::ok(res)?)
}

/// 管理员模拟目标用户登录 (Impersonation)
#[utoipa::path(
    post,
    path = "/v1/admin/impersonation-sessions",
    summary = "模拟用户身份登录",
    description = "创建模拟身份会话",
    request_body = AdminImpersonateUserRequest,
    responses(
        (status = 200, description = "模拟成功", body = AuthResource),
        (status = 400, description = "禁止模拟管理员用户", body = ApiError),
        (status = 401, description = "未登录", body = ApiError),
        (status = 403, description = "非管理员权限拒绝", body = ApiError),
        (status = 404, description = "目标用户不存在", body = ApiError)
    ),
    tag = TAG_ADMIN
)]
#[debug_handler]
pub async fn admin_impersonate_user(
    State(ctx): State<AppContext>,
    auth: AuthSession,
    headers: HeaderMap,
    ValidJson(params): ValidJson<AdminImpersonateUserRequest>,
) -> AppResult<Response> {
    let (ip, ua) = extract_client_info(&headers);
    let res =
        AuthAdminService::admin_impersonate_user(&ctx.db, &auth.user, ip, ua, &params).await?;
    let mut resp_headers = HeaderMap::new();
    resp_headers.insert(
        SET_COOKIE,
        build_session_cookie(&res.token, SESSION_COOKIE_MAX_AGE_SECS),
    );
    Ok((StatusCode::OK, resp_headers, Json(res)).into_response())
}

/// 退出模拟身份并恢复管理员会话
#[utoipa::path(
    delete,
    path = "/v1/admin/impersonation-sessions",
    summary = "停止模拟并恢复原身份",
    description = "停止模拟身份",
    responses(
        (status = 200, description = "恢复成功", body = AuthResource),
        (status = 400, description = "当前会话非模拟会话", body = ApiError),
        (status = 401, description = "未登录", body = ApiError),
        (status = 404, description = "原管理员用户不存在", body = ApiError)
    ),
    tag = TAG_ADMIN
)]
#[debug_handler]
pub async fn admin_stop_impersonating(
    State(ctx): State<AppContext>,
    auth: AuthSession,
) -> AppResult<Response> {
    let res = AuthAdminService::admin_stop_impersonating(&ctx.db, &auth.session).await?;
    let mut resp_headers = HeaderMap::new();
    resp_headers.insert(
        SET_COOKIE,
        build_session_cookie(&res.token, SESSION_COOKIE_MAX_AGE_SECS),
    );
    Ok((StatusCode::OK, resp_headers, Json(res)).into_response())
}

/// 管理员分页获取用户列表
#[utoipa::path(
    get,
    path = "/v1/admin/accounts",
    summary = "获取全平台用户列表",
    description = "分页查询所有用户",
    params(
        ("limit" = Option<u64>, Query, description = "每页数量"),
        ("offset" = Option<u64>, Query, description = "偏移量"),
        ("searchValue" = Option<String>, Query, description = "搜索关键词")
    ),
    responses(
        (status = 200, description = "获取成功", body = PaginatedResource<UserResource>),
        (status = 401, description = "未登录", body = ApiError),
        (status = 403, description = "非管理员权限拒绝", body = ApiError)
    ),
    tag = TAG_ADMIN
)]
#[debug_handler]
pub async fn admin_list_users(
    State(ctx): State<AppContext>,
    auth: AuthSession,
    Query(query): Query<AdminListUsersQuery>,
) -> AppResult<Response> {
    let res = AuthAdminService::admin_list_users(&ctx.db, &auth.user, &query).await?;
    Ok(format_res::ok(res)?)
}

/// 管理员物理删除用户
#[utoipa::path(
    delete,
    path = "/v1/admin/accounts",
    summary = "彻底删除用户",
    description = "级联删除用户",
    request_body = AdminRemoveUserRequest,
    responses(
        (status = 200, description = "删除成功", body = StatusResource),
        (status = 400, description = "禁止删除自己", body = ApiError),
        (status = 401, description = "未登录", body = ApiError),
        (status = 403, description = "非管理员权限拒绝", body = ApiError),
        (status = 404, description = "目标用户不存在", body = ApiError)
    ),
    tag = TAG_ADMIN
)]
#[debug_handler]
pub async fn admin_remove_user(
    State(ctx): State<AppContext>,
    auth: AuthSession,
    ValidJson(params): ValidJson<AdminRemoveUserRequest>,
) -> AppResult<Response> {
    let res = AuthAdminService::admin_remove_user(&ctx.db, &auth.user, &params).await?;
    Ok(format_res::ok(res)?)
}
