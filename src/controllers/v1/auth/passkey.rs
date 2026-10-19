use axum::{
    Json,
    extract::{Query, State},
    http::{HeaderMap, StatusCode, header::SET_COOKIE},
    response::{IntoResponse, Response},
};
use loco_rs::prelude::*;

use crate::{
    common::consts::openapi::TAG_CREDENTIALS,
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

// ─────────────────────────────────────────────────────────────────────────────
// Phase 3 控制器接口处理函数 (Passkey, Device Auth, OAuth2, OneTap, JWT, JWKS)
// ─────────────────────────────────────────────────────────────────────────────

/// 生成 Passkey 认证挑战选项
#[utoipa::path(
    post,
    path = "/v1/passkey-authentication-challenges",
    summary = "生成通行密钥挑战",
    description = "生成通行密钥挑战",
    request_body = PasskeyGenerateAuthOptionsRequest,
    responses(
        (status = 200, description = "挑战选项生成成功", body = serde_json::Value)
    ),
    tag = TAG_CREDENTIALS
)]
#[debug_handler]
pub async fn passkey_generate_authenticate_options(
    State(ctx): State<AppContext>,
    ValidJson(params): ValidJson<PasskeyGenerateAuthOptionsRequest>,
) -> AppResult<Response> {
    let res = CredentialService::generate_passkey_auth_options(&ctx, &params).await?;
    Ok(format_res::ok(res)?)
}

/// 校验 Passkey 认证签名并完成登录
#[utoipa::path(
    post,
    path = "/v1/passkey-authentications",
    summary = "校验通行密钥认证",
    description = "校验通行密钥并签发会话",
    request_body = PasskeyVerifyAuthRequest,
    responses(
        (status = 200, description = "认证成功", body = AuthResource),
        (status = 401, description = "凭据无效或签名校验失败", body = ApiError)
    ),
    tag = TAG_CREDENTIALS
)]
#[debug_handler]
pub async fn passkey_verify_authentication(
    State(ctx): State<AppContext>,
    headers: HeaderMap,
    ValidJson(params): ValidJson<PasskeyVerifyAuthRequest>,
) -> AppResult<Response> {
    let (ip, ua) = extract_client_info(&headers);
    let res = CredentialService::verify_passkey_auth(&ctx, &params, ip, ua).await?;
    let mut resp_headers = HeaderMap::new();
    resp_headers.insert(
        SET_COOKIE,
        build_session_cookie(&res.token, SESSION_COOKIE_MAX_AGE_SECS),
    );
    Ok((StatusCode::OK, resp_headers, Json(res)).into_response())
}

/// 生成 Passkey 注册选项
#[utoipa::path(
    post,
    path = "/v1/passkey-registration-challenges",
    summary = "生成通行密钥选项",
    description = "生成通行密钥注册选项",
    request_body = PasskeyGenerateRegOptionsRequest,
    responses(
        (status = 200, description = "注册选项生成成功", body = serde_json::Value),
        (status = 401, description = "未授权", body = ApiError)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = TAG_CREDENTIALS
)]
#[debug_handler]
pub async fn passkey_generate_register_options(
    State(ctx): State<AppContext>,
    auth: AuthSession,
    ValidJson(_params): ValidJson<PasskeyGenerateRegOptionsRequest>,
) -> AppResult<Response> {
    let res = CredentialService::generate_passkey_reg_options(&ctx, &auth.user.id).await?;
    Ok(format_res::ok(res)?)
}

/// 校验 Passkey 注册凭据并持久化
#[utoipa::path(
    post,
    path = "/v1/passkeys",
    summary = "校验通行密钥注册",
    description = "校验并保存通行密钥",
    request_body = PasskeyVerifyRegRequest,
    responses(
        (status = 200, description = "注册成功", body = PasskeyResource),
        (status = 401, description = "未授权", body = ApiError)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = TAG_CREDENTIALS
)]
#[debug_handler]
pub async fn passkey_verify_registration(
    State(ctx): State<AppContext>,
    auth: AuthSession,
    ValidJson(params): ValidJson<PasskeyVerifyRegRequest>,
) -> AppResult<Response> {
    let res = CredentialService::verify_passkey_reg(&ctx, &auth.user.id, &params).await?;
    Ok(format_res::ok(res)?)
}

/// 获取用户 Passkey 凭据列表
#[utoipa::path(
    get,
    path = "/v1/passkeys",
    summary = "获取通行密钥列表",
    description = "列出全部通行密钥",
    responses(
        (status = 200, description = "查询成功", body = Vec<PasskeyResource>),
        (status = 401, description = "未授权", body = ApiError)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = TAG_CREDENTIALS
)]
#[debug_handler]
pub async fn passkey_list_user_passkeys(
    State(ctx): State<AppContext>,
    auth: AuthSession,
) -> AppResult<Response> {
    let res = CredentialService::list_user_passkeys(&ctx.db, &auth.user.id).await?;
    Ok(format_res::ok(res)?)
}

/// 删除指定 Passkey 凭据
#[utoipa::path(
    delete,
    path = "/v1/passkeys",
    summary = "删除通行密钥",
    description = "用户删除指定注册的 Passkey 凭据",
    params(
        ("id" = String, Query, description = "Passkey ID")
    ),
    responses(
        (status = 200, description = "删除成功", body = StatusResource),
        (status = 404, description = "凭据不存在", body = ApiError)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = TAG_CREDENTIALS
)]
#[debug_handler]
pub async fn passkey_delete_passkey(
    State(ctx): State<AppContext>,
    auth: AuthSession,
    Query(params): Query<DeletePasskeyRequest>,
) -> AppResult<Response> {
    let res = CredentialService::delete_passkey(&ctx.db, &auth.user.id, &params).await?;
    Ok(format_res::ok(res)?)
}

/// 更新 Passkey 备注名称
#[utoipa::path(
    patch,
    path = "/v1/passkeys",
    summary = "重命名通行密钥",
    description = "重命名通行密钥",
    request_body = UpdatePasskeyRequest,
    responses(
        (status = 200, description = "更新成功", body = PasskeyResource),
        (status = 404, description = "凭据不存在", body = ApiError)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = TAG_CREDENTIALS
)]
#[debug_handler]
pub async fn passkey_update_passkey(
    State(ctx): State<AppContext>,
    auth: AuthSession,
    ValidJson(params): ValidJson<UpdatePasskeyRequest>,
) -> AppResult<Response> {
    let res = CredentialService::update_passkey(&ctx.db, &auth.user.id, &params).await?;
    Ok(format_res::ok(res)?)
}
