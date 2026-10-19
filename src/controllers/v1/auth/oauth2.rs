use axum::{
    Json,
    extract::{Query, State},
    http::{HeaderMap, StatusCode, header::SET_COOKIE},
    response::{IntoResponse, Response},
};
use loco_rs::prelude::*;

use crate::{
    common::consts::openapi::TAG_SIGNIN,
    dtos::{
        auth::*,
        common::{ApiError, ValidJson, format_res},
    },
    errors::AppResult,
    services::CredentialService,
};

// ─────────────────────────────────────────────────────────────────────────────
// Cookie 与 Token 辅助函数
// ─────────────────────────────────────────────────────────────────────────────

use super::shared::*;

/// 通用 OAuth2 登录重定向地址获取
#[utoipa::path(
    post,
    path = "/v1/sessions/oauth2",
    summary = "OAuth2 跳转",
    description = "生成 OAuth2 跳转地址",
    request_body = SignInOAuth2Request,
    responses(
        (status = 200, description = "生成成功", body = OAuth2UrlResource)
    ),
    tag = TAG_SIGNIN
)]
#[debug_handler]
pub async fn sign_in_oauth2(
    State(ctx): State<AppContext>,
    ValidJson(params): ValidJson<SignInOAuth2Request>,
) -> AppResult<Response> {
    let res = CredentialService::sign_in_oauth2(&ctx, &params).await?;
    Ok(format_res::ok(res)?)
}

/// OAuth2 回调登录凭证校验
#[utoipa::path(
    get,
    path = "/v1/oauth2-callbacks",
    summary = "OAuth2 回调",
    description = "处理 OAuth2 回调",
    params(
        ("code" = Option<String>, Query, description = "OAuth2 授权码"),
        ("state" = Option<String>, Query, description = "OAuth2 状态校验值"),
        ("error" = Option<String>, Query, description = "OAuth2 错误码")
    ),
    responses(
        (status = 200, description = "登录成功", body = AuthResource),
        (status = 400, description = "状态无效或缺少参数", body = ApiError)
    ),
    tag = TAG_SIGNIN
)]
#[debug_handler]
pub async fn oauth2_callback(
    State(ctx): State<AppContext>,
    headers: HeaderMap,
    Query(params): Query<OAuth2CallbackRequest>,
) -> AppResult<Response> {
    let (ip, ua) = extract_client_info(&headers);
    let res = CredentialService::oauth2_callback(&ctx, &params, ip, ua).await?;
    let mut resp_headers = HeaderMap::new();
    resp_headers.insert(
        SET_COOKIE,
        build_session_cookie(&res.token, SESSION_COOKIE_MAX_AGE_SECS),
    );
    Ok((StatusCode::OK, resp_headers, Json(res)).into_response())
}

/// Google One Tap 凭证回调登录
#[utoipa::path(
    post,
    path = "/v1/one-tap-credentials",
    summary = "Google 登录",
    description = "校验 One Tap 凭据",
    request_body = OneTapCallbackRequest,
    responses(
        (status = 200, description = "登录成功", body = AuthResource),
        (status = 400, description = "凭据格式错误", body = ApiError)
    ),
    tag = TAG_SIGNIN
)]
#[debug_handler]
pub async fn one_tap_callback(
    State(ctx): State<AppContext>,
    headers: HeaderMap,
    ValidJson(params): ValidJson<OneTapCallbackRequest>,
) -> AppResult<Response> {
    let (ip, ua) = extract_client_info(&headers);
    let res = CredentialService::one_tap_callback(&ctx, &params, ip, ua).await?;
    let mut resp_headers = HeaderMap::new();
    resp_headers.insert(
        SET_COOKIE,
        build_session_cookie(&res.token, SESSION_COOKIE_MAX_AGE_SECS),
    );
    Ok((StatusCode::OK, resp_headers, Json(res)).into_response())
}
