use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header::SET_COOKIE},
    response::{IntoResponse, Response},
};
use loco_rs::prelude::*;

use crate::{
    common::consts::openapi::TAG_SESSION,
    dtos::{
        auth::*,
        common::{ApiError, ValidJson, format_res},
    },
    errors::{AppError, AppResult},
    services::SessionService,
};

// ─────────────────────────────────────────────────────────────────────────────
// Cookie 与 Token 辅助函数
// ─────────────────────────────────────────────────────────────────────────────

use super::shared::*;

// ─────────────────────────────────────────────────────────────────────────────
// 控制器接口处理函数 (Handlers)
// ─────────────────────────────────────────────────────────────────────────────

/// 邮箱密码注册
#[utoipa::path(
    post,
    path = "/v1/users",
    summary = "邮箱密码注册",
    description = "注册并签发会话",
    request_body = SignUpEmailRequest,
    responses(
        (status = 201, description = "注册成功并返回认证会话", body = AuthResource),
        (status = 409, description = "邮箱或用户名已存在", body = ApiError),
        (status = 422, description = "参数校验失败", body = ApiError)
    ),
    tag = TAG_SESSION
)]
#[debug_handler]
pub async fn sign_up_email(
    State(ctx): State<AppContext>,
    headers: HeaderMap,
    ValidJson(params): ValidJson<SignUpEmailRequest>,
) -> AppResult<Response> {
    let (ip, ua) = extract_client_info(&headers);
    let res = SessionService::sign_up_email(&ctx.db, &params, ip, ua).await?;

    let mut resp_headers = HeaderMap::new();
    resp_headers.insert(
        SET_COOKIE,
        build_session_cookie(&res.token, SESSION_COOKIE_MAX_AGE_SECS),
    );
    Ok((StatusCode::CREATED, resp_headers, Json(res)).into_response())
}

/// 邮箱密码登录
#[utoipa::path(
    post,
    path = "/v1/sessions/email",
    summary = "邮箱密码登录",
    description = "邮箱密码登录",
    request_body = SignInEmailRequest,
    responses(
        (status = 200, description = "登录成功", body = SignInResponse),
        (status = 401, description = "邮箱或密码错误", body = ApiError),
        (status = 403, description = "账号已被封禁", body = ApiError)
    ),
    tag = TAG_SESSION
)]
#[debug_handler]
pub async fn sign_in_email(
    State(ctx): State<AppContext>,
    headers: HeaderMap,
    ValidJson(params): ValidJson<SignInEmailRequest>,
) -> AppResult<Response> {
    let (ip, ua) = extract_client_info(&headers);
    let res = SessionService::sign_in_email(&ctx.db, &params, ip, ua).await?;
    Ok(format_sign_in_result(res))
}

/// 用户名密码登录
#[utoipa::path(
    post,
    path = "/v1/sessions/username",
    summary = "用户名密码登录",
    description = "用户名密码登录",
    request_body = SignInUsernameRequest,
    responses(
        (status = 200, description = "登录成功", body = SignInResponse),
        (status = 401, description = "用户名或密码错误", body = ApiError),
        (status = 403, description = "账号已被封禁", body = ApiError)
    ),
    tag = TAG_SESSION
)]
#[debug_handler]
pub async fn sign_in_username(
    State(ctx): State<AppContext>,
    headers: HeaderMap,
    ValidJson(params): ValidJson<SignInUsernameRequest>,
) -> AppResult<Response> {
    let (ip, ua) = extract_client_info(&headers);
    let res = SessionService::sign_in_username(&ctx.db, &params, ip, ua).await?;
    Ok(format_sign_in_result(res))
}

/// 匿名用户登录
#[utoipa::path(
    post,
    path = "/v1/sessions/anonymous",
    summary = "匿名用户登录",
    description = "自动创建临时匿名用户并签发活跃持久化会话",
    responses(
        (status = 200, description = "匿名登录成功", body = AuthResource)
    ),
    tag = TAG_SESSION
)]
#[debug_handler]
pub async fn sign_in_anonymous(
    State(ctx): State<AppContext>,
    headers: HeaderMap,
) -> AppResult<Response> {
    let (ip, ua) = extract_client_info(&headers);
    let res = SessionService::sign_in_anonymous(&ctx.db, ip, ua).await?;

    let mut resp_headers = HeaderMap::new();
    resp_headers.insert(
        SET_COOKIE,
        build_session_cookie(&res.token, SESSION_COOKIE_MAX_AGE_SECS),
    );
    Ok((StatusCode::OK, resp_headers, Json(res)).into_response())
}

/// 获取当前会话信息
#[utoipa::path(
    get,
    path = "/v1/sessions/current",
    summary = "获取当前会话",
    description = "获取当前会话与用户",
    responses(
        (status = 200, description = "获取成功", body = SessionContextResource),
        (status = 401, description = "未携带或令牌无效", body = ApiError)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = TAG_SESSION
)]
#[debug_handler]
pub async fn get_session(State(ctx): State<AppContext>, headers: HeaderMap) -> AppResult<Response> {
    let Some(token) = extract_token_from_headers(&headers) else {
        return Err(AppError::unauthorized("缺少访问令牌"));
    };

    let session_context = SessionService::get_session(&ctx.db, &token).await?;
    format_res::ok(session_context).map_err(Into::into)
}

/// 退出登录
#[utoipa::path(
    post,
    path = "/v1/sessions/current",
    summary = "退出登录",
    description = "退出并清除会话",
    responses(
        (status = 204, description = "退出成功，无响应体")
    ),
    tag = TAG_SESSION
)]
#[debug_handler]
pub async fn sign_out(State(ctx): State<AppContext>, headers: HeaderMap) -> AppResult<Response> {
    if let Some(token) = extract_token_from_headers(&headers) {
        let _ = SessionService::sign_out(&ctx.db, &token).await;
    }

    let mut resp_headers = HeaderMap::new();
    resp_headers.insert(SET_COOKIE, build_clear_session_cookie());
    Ok((StatusCode::NO_CONTENT, resp_headers).into_response())
}

/// 查询当前用户所有活跃会话
#[utoipa::path(
    get,
    path = "/v1/sessions",
    summary = "查询用户所有活跃会话",
    description = "列出所有活跃会话",
    responses(
        (status = 200, description = "获取会话列表成功", body = Vec<SessionResource>),
        (status = 401, description = "未登录或会话已过期", body = ApiError)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = TAG_SESSION
)]
#[debug_handler]
pub async fn list_sessions(
    State(ctx): State<AppContext>,
    auth: AuthSession,
) -> AppResult<Response> {
    let sessions = SessionService::list_sessions(&ctx.db, &auth.user.id).await?;
    Ok(format_res::ok(sessions)?)
}

/// 吊销指定会话
#[utoipa::path(
    delete,
    path = "/v1/sessions/{id}",
    summary = "吊销指定会话",
    description = "按会话 ID 吊销，幂等",
    params(("id" = String, Path, description = "会话 ID")),
    responses(
        (status = 204, description = "会话已吊销，无响应体"),
        (status = 401, description = "未授权", body = ApiError)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = TAG_SESSION
)]
#[debug_handler]
pub async fn revoke_session(
    State(ctx): State<AppContext>,
    auth: AuthSession,
    Path(id): Path<String>,
) -> AppResult<Response> {
    SessionService::revoke_session_by_id(&ctx.db, &id, &auth.user.id).await?;
    format_res::no_content().map_err(Into::into)
}

/// 吊销其他所有活跃会话
#[utoipa::path(
    delete,
    path = "/v1/sessions",
    summary = "吊销其他所有活跃会话",
    description = "吊销其他所有会话",
    responses(
        (status = 204, description = "其他会话已全部吊销，无响应体"),
        (status = 401, description = "未授权", body = ApiError)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = TAG_SESSION
)]
#[debug_handler]
pub async fn revoke_other_sessions(
    State(ctx): State<AppContext>,
    auth: AuthSession,
) -> AppResult<Response> {
    SessionService::revoke_other_sessions(&ctx.db, &auth.user.id, &auth.token).await?;
    format_res::no_content().map_err(Into::into)
}

/// 切换并激活目标会话 (Multi-session)
#[utoipa::path(
    patch,
    path = "/v1/sessions/current",
    summary = "激活切换目标会话",
    description = "切换活跃会话",
    request_body = SetActiveSessionRequest,
    responses(
        (status = 200, description = "激活会话成功", body = SessionResource),
        (status = 400, description = "目标会话无效", body = ApiError),
        (status = 401, description = "未授权", body = ApiError),
        (status = 403, description = "目标会话不属于当前用户", body = ApiError)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = TAG_SESSION
)]
#[debug_handler]
pub async fn set_active_session(
    State(ctx): State<AppContext>,
    auth: AuthSession,
    ValidJson(params): ValidJson<SetActiveSessionRequest>,
) -> AppResult<Response> {
    let session_res =
        SessionService::set_active_session(&ctx.db, &auth.user.id, &params.session_token).await?;

    // 切换会话即以目标会话 token 重建 Cookie
    let mut resp_headers = HeaderMap::new();
    resp_headers.insert(
        SET_COOKIE,
        build_session_cookie(&params.session_token, SESSION_COOKIE_MAX_AGE_SECS),
    );
    Ok((StatusCode::OK, resp_headers, Json(session_res)).into_response())
}
