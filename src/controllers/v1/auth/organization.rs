use axum::{
    extract::{Query, State},
    response::Response,
};
use loco_rs::prelude::*;

use crate::{
    common::consts::openapi::TAG_ORG,
    dtos::{
        auth::*,
        common::{ApiError, ValidJson, format_res},
    },
    errors::AppResult,
    services::MembershipService,
};

// ─────────────────────────────────────────────────────────────────────────────
// Cookie 与 Token 辅助函数
// ─────────────────────────────────────────────────────────────────────────────

use super::shared::*;

// ─────────────────────────────────────────────────────────────────────────────
// 组织多租户服务 (Organization Plugin Handlers)
// ─────────────────────────────────────────────────────────────────────────────

/// 创建新组织
#[utoipa::path(
    post,
    path = "/v1/organizations",
    summary = "创建新组织",
    description = "创建组织并激活",
    request_body = CreateOrganizationRequest,
    responses(
        (status = 200, description = "创建成功", body = OrganizationResource),
        (status = 400, description = "请求参数错误", body = ApiError),
        (status = 401, description = "未登录", body = ApiError),
        (status = 409, description = "组织 slug 已存在", body = ApiError)
    ),
    tag = TAG_ORG
)]
#[debug_handler]
pub async fn create_organization(
    State(ctx): State<AppContext>,
    auth: AuthSession,
    ValidJson(params): ValidJson<CreateOrganizationRequest>,
) -> AppResult<Response> {
    let res =
        MembershipService::create_organization(&ctx.db, &auth.user, &auth.session, &params).await?;
    Ok(format_res::ok(res)?)
}

/// 更新组织信息
#[utoipa::path(
    patch,
    path = "/v1/organizations/current",
    summary = "更新组织资料",
    description = "更新组织资料",
    request_body = UpdateOrganizationRequest,
    responses(
        (status = 200, description = "更新成功", body = OrganizationResource),
        (status = 401, description = "未登录", body = ApiError),
        (status = 403, description = "无权修改组织", body = ApiError),
        (status = 404, description = "组织不存在", body = ApiError),
        (status = 409, description = "新 slug 已被占用", body = ApiError)
    ),
    tag = TAG_ORG
)]
#[debug_handler]
pub async fn update_organization(
    State(ctx): State<AppContext>,
    auth: AuthSession,
    ValidJson(params): ValidJson<UpdateOrganizationRequest>,
) -> AppResult<Response> {
    let res = MembershipService::update_organization(&ctx.db, &auth.user, &params).await?;
    Ok(format_res::ok(res)?)
}

/// 删除组织
#[utoipa::path(
    delete,
    path = "/v1/organizations/{organizationId}",
    summary = "删除组织",
    description = "删除组织及其关联",
    params(("organizationId" = String, Path, description = "组织 ID")),
    responses(
        (status = 200, description = "删除成功", body = StatusResource),
        (status = 401, description = "未登录", body = ApiError),
        (status = 403, description = "非所有者无权删除", body = ApiError)
    ),
    tag = TAG_ORG
)]
#[debug_handler]
pub async fn delete_organization(
    State(ctx): State<AppContext>,
    auth: AuthSession,
    Path(organization_id): Path<String>,
) -> AppResult<Response> {
    let res = MembershipService::delete_organization(&ctx.db, &auth.user, &organization_id).await?;
    Ok(format_res::ok(res)?)
}

/// 获取组织详情与成员列表
#[utoipa::path(
    get,
    path = "/v1/organizations/current",
    summary = "组织详情",
    description = "组织详情与成员列表",
    params(
        ("organizationId" = Option<String>, Query, description = "组织 ID"),
        ("organizationSlug" = Option<String>, Query, description = "组织 Slug 缩写")
    ),
    responses(
        (status = 200, description = "获取成功", body = FullOrganizationResource),
        (status = 401, description = "未登录", body = ApiError),
        (status = 403, description = "无权查看该组织", body = ApiError),
        (status = 404, description = "组织不存在", body = ApiError)
    ),
    tag = TAG_ORG
)]
#[debug_handler]
pub async fn get_full_organization(
    State(ctx): State<AppContext>,
    auth: AuthSession,
    Query(query): Query<GetFullOrganizationQuery>,
) -> AppResult<Response> {
    let res = MembershipService::get_full_organization(&ctx.db, &auth.user, &query).await?;
    Ok(format_res::ok(res)?)
}

/// 获取当前用户所属的所有组织
#[utoipa::path(
    get,
    path = "/v1/organizations",
    summary = "获取用户所属组织列表",
    description = "查询当前登录用户加入的所有组织集合",
    responses(
        (status = 200, description = "获取成功", body = Vec<OrganizationResource>),
        (status = 401, description = "未登录", body = ApiError)
    ),
    tag = TAG_ORG
)]
#[debug_handler]
pub async fn list_organizations(
    State(ctx): State<AppContext>,
    auth: AuthSession,
) -> AppResult<Response> {
    let res = MembershipService::list_organizations(&ctx.db, &auth.user).await?;
    Ok(format_res::ok(res)?)
}

/// 切换或清空当前会话活跃组织
#[utoipa::path(
    patch,
    path = "/v1/organizations/current/active",
    summary = "设置当前活跃组织",
    description = "切换当前活跃组织",
    request_body = SetActiveOrganizationRequest,
    responses(
        (status = 200, description = "设置成功", body = Option<OrganizationResource>),
        (status = 401, description = "未登录", body = ApiError),
        (status = 403, description = "非该组织成员", body = ApiError),
        (status = 404, description = "组织不存在", body = ApiError)
    ),
    tag = TAG_ORG
)]
#[debug_handler]
pub async fn set_active_organization(
    State(ctx): State<AppContext>,
    auth: AuthSession,
    ValidJson(params): ValidJson<SetActiveOrganizationRequest>,
) -> AppResult<Response> {
    let res =
        MembershipService::set_active_organization(&ctx.db, &auth.user, &auth.session, &params)
            .await?;
    Ok(format_res::ok(res)?)
}

/// 获取当前会话活跃组织的成员档案
#[utoipa::path(
    get,
    path = "/v1/organizations/current/members/me",
    summary = "活跃成员档案",
    description = "当前活跃成员身份",
    responses(
        (status = 200, description = "获取成功", body = ActiveMemberResource),
        (status = 400, description = "未选择活跃组织", body = ApiError),
        (status = 401, description = "未登录", body = ApiError),
        (status = 403, description = "非组织成员", body = ApiError)
    ),
    tag = TAG_ORG
)]
#[debug_handler]
pub async fn get_active_member(
    State(ctx): State<AppContext>,
    auth: AuthSession,
) -> AppResult<Response> {
    let res = MembershipService::get_active_member(&ctx.db, &auth.user, &auth.session).await?;
    Ok(format_res::ok(res)?)
}
