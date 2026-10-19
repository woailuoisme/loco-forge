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

/// 邀请新成员加入组织
#[utoipa::path(
    post,
    path = "/v1/organizations/invitations",
    summary = "邀请组织新成员",
    description = "发送组织邀请",
    request_body = InviteMemberRequest,
    responses(
        (status = 200, description = "邀请成功", body = InvitationResource),
        (status = 401, description = "未登录", body = ApiError),
        (status = 403, description = "无邀请权限", body = ApiError),
        (status = 409, description = "用户已是组织成员", body = ApiError)
    ),
    tag = TAG_ORG
)]
#[debug_handler]
pub async fn invite_member(
    State(ctx): State<AppContext>,
    auth: AuthSession,
    ValidJson(params): ValidJson<InviteMemberRequest>,
) -> AppResult<Response> {
    let res = MembershipService::invite_member(&ctx.db, &auth.user, &params).await?;
    Ok(format_res::ok(res)?)
}

/// 接受组织加入邀请
#[utoipa::path(
    post,
    path = "/v1/invitation-acceptances",
    summary = "接受组织邀请",
    description = "接受有效未过期的组织邀请并成为正式成员",
    request_body = AcceptInvitationRequest,
    responses(
        (status = 200, description = "加入成功", body = MemberResource),
        (status = 400, description = "邀请已失效或过期", body = ApiError),
        (status = 401, description = "未登录", body = ApiError),
        (status = 403, description = "邀请邮箱不匹配", body = ApiError),
        (status = 404, description = "邀请不存在", body = ApiError)
    ),
    tag = TAG_ORG
)]
#[debug_handler]
pub async fn accept_invitation(
    State(ctx): State<AppContext>,
    auth: AuthSession,
    ValidJson(params): ValidJson<AcceptInvitationRequest>,
) -> AppResult<Response> {
    let res = MembershipService::accept_invitation(&ctx.db, &auth.user, &params).await?;
    Ok(format_res::ok(res)?)
}

/// 拒绝组织加入邀请
#[utoipa::path(
    post,
    path = "/v1/invitation-rejections",
    summary = "拒绝组织邀请",
    description = "受邀者主动拒绝加入组织",
    request_body = RejectInvitationRequest,
    responses(
        (status = 200, description = "拒绝成功", body = StatusResource),
        (status = 401, description = "未登录", body = ApiError),
        (status = 403, description = "邀请邮箱不匹配", body = ApiError),
        (status = 404, description = "邀请不存在", body = ApiError)
    ),
    tag = TAG_ORG
)]
#[debug_handler]
pub async fn reject_invitation(
    State(ctx): State<AppContext>,
    auth: AuthSession,
    ValidJson(params): ValidJson<RejectInvitationRequest>,
) -> AppResult<Response> {
    let res = MembershipService::reject_invitation(&ctx.db, &auth.user, &params).await?;
    Ok(format_res::ok(res)?)
}

/// 撤销已发出的组织邀请
#[utoipa::path(
    delete,
    path = "/v1/invitations",
    summary = "撤销组织邀请",
    description = "组织管理者或邀请发起人撤销挂起的邀请",
    params(
        ("invitationId" = String, Query, description = "邀请 ID")
    ),
    responses(
        (status = 200, description = "撤销成功", body = StatusResource),
        (status = 401, description = "未登录", body = ApiError),
        (status = 403, description = "无撤销权限", body = ApiError),
        (status = 404, description = "邀请不存在", body = ApiError)
    ),
    tag = TAG_ORG
)]
#[debug_handler]
pub async fn cancel_invitation(
    State(ctx): State<AppContext>,
    auth: AuthSession,
    Query(params): Query<CancelInvitationRequest>,
) -> AppResult<Response> {
    let res = MembershipService::cancel_invitation(&ctx.db, &auth.user, &params).await?;
    Ok(format_res::ok(res)?)
}

/// 移除组织成员
#[utoipa::path(
    delete,
    path = "/v1/members",
    summary = "移除组织成员",
    description = "移除组织成员",
    params(
        ("organizationId" = String, Query, description = "组织 ID"),
        ("memberIdOrEmail" = String, Query, description = "成员 ID 或邮箱")
    ),
    responses(
        (status = 200, description = "移除成功", body = StatusResource),
        (status = 401, description = "未登录", body = ApiError),
        (status = 403, description = "无权移除该成员", body = ApiError),
        (status = 404, description = "成员不存在", body = ApiError)
    ),
    tag = TAG_ORG
)]
#[debug_handler]
pub async fn remove_member(
    State(ctx): State<AppContext>,
    auth: AuthSession,
    Query(params): Query<RemoveMemberRequest>,
) -> AppResult<Response> {
    let res = MembershipService::remove_member(&ctx.db, &auth.user, &params).await?;
    Ok(format_res::ok(res)?)
}

/// 更新组织成员角色
#[utoipa::path(
    patch,
    path = "/v1/members",
    summary = "变更成员角色",
    description = "变更成员角色",
    request_body = UpdateMemberRoleRequest,
    responses(
        (status = 200, description = "修改成功", body = MemberResource),
        (status = 401, description = "未登录", body = ApiError),
        (status = 403, description = "无权修改该角色", body = ApiError),
        (status = 404, description = "成员不存在", body = ApiError)
    ),
    tag = TAG_ORG
)]
#[debug_handler]
pub async fn update_member_role(
    State(ctx): State<AppContext>,
    auth: AuthSession,
    ValidJson(params): ValidJson<UpdateMemberRoleRequest>,
) -> AppResult<Response> {
    let res = MembershipService::update_member_role(&ctx.db, &auth.user, &params).await?;
    Ok(format_res::ok(res)?)
}

/// 主动退出组织
#[utoipa::path(
    delete,
    path = "/v1/members/me",
    summary = "退出组织",
    description = "退出组织",
    params(
        ("organizationId" = String, Query, description = "组织 ID")
    ),
    responses(
        (status = 200, description = "退出成功", body = StatusResource),
        (status = 400, description = "唯一所有者禁止退出", body = ApiError),
        (status = 401, description = "未登录", body = ApiError),
        (status = 404, description = "非该组织成员", body = ApiError)
    ),
    tag = TAG_ORG
)]
#[debug_handler]
pub async fn leave_organization(
    State(ctx): State<AppContext>,
    auth: AuthSession,
    Query(params): Query<LeaveOrganizationRequest>,
) -> AppResult<Response> {
    let res = MembershipService::leave_organization(&ctx.db, &auth.user, &params).await?;
    Ok(format_res::ok(res)?)
}
