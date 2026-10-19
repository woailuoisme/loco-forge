use std::sync::OnceLock;
use std::time::{Duration, Instant};

use jsonwebtoken::{DecodingKey, Validation, decode, decode_header, jwk::JwkSet};
use loco_rs::prelude::*;
use serde::Deserialize;

use crate::{
    common::utils::code::CodeUtil,
    dtos::auth::*,
    errors::{AppError, AppResult},
    models::{account, session, user, verification},
};

use super::CredentialService;

/// Google OAuth2/OIDC 默认端点（可被 settings.oauth2.{provider} 覆盖）
const GOOGLE_ISSUER: &str = "https://accounts.google.com";
const GOOGLE_AUTHORIZE_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_JWKS_URL: &str = "https://www.googleapis.com/oauth2/v3/certs";
const OAUTH_SCOPE: &str = "openid email profile";
const HTTP_TIMEOUT_SECS: u64 = 10;
const JWKS_CACHE_TTL_SECS: u64 = 3600;

/// OIDC 标准端点集合
struct Endpoints {
    authorize_url: String,
    token_url: String,
    jwks_url: String,
    issuer: String,
}

/// settings.oauth2.{provider} 的凭据配置
#[derive(Debug, Deserialize)]
struct ProviderConfig {
    client_id: String,
    client_secret: String,
    authorize_url: Option<String>,
    token_url: Option<String>,
    jwks_url: Option<String>,
    issuer: Option<String>,
}

/// Token endpoint 响应
#[derive(Deserialize)]
struct TokenResponse {
    id_token: Option<String>,
    access_token: Option<String>,
}

/// id_token 标准声明
#[derive(Deserialize)]
struct IdClaims {
    email: Option<String>,
    sub: String,
    name: Option<String>,
}

/// state 记录负载（关联 provider 与 redirect_uri，保证 token 交换一致）
#[derive(serde::Serialize, serde::Deserialize)]
struct StatePayload {
    provider_id: String,
    redirect_uri: String,
}

/// 从应用配置读取 provider 凭据 (settings.oauth2.{provider_id})
fn provider_config(ctx: &AppContext, provider_id: &str) -> AppResult<ProviderConfig> {
    ctx.config
        .settings
        .as_ref()
        .and_then(|s| s.get("oauth2"))
        .and_then(|o| o.get(provider_id))
        .map_or_else(
            || serde_json::from_value(serde_json::Value::Null),
            |v| serde_json::from_value(v.clone()),
        )
        .map_err(|_| {
            AppError::bad_request(format!("OAuth provider `{provider_id}` is not configured"))
        })
}

/// 解析 provider 的 OIDC 端点（配置覆盖 > Google 默认值）
fn endpoints(provider: &ProviderConfig) -> Endpoints {
    Endpoints {
        authorize_url: provider
            .authorize_url
            .clone()
            .unwrap_or_else(|| GOOGLE_AUTHORIZE_URL.to_string()),
        token_url: provider
            .token_url
            .clone()
            .unwrap_or_else(|| GOOGLE_TOKEN_URL.to_string()),
        jwks_url: provider
            .jwks_url
            .clone()
            .unwrap_or_else(|| GOOGLE_JWKS_URL.to_string()),
        issuer: provider
            .issuer
            .clone()
            .unwrap_or_else(|| GOOGLE_ISSUER.to_string()),
    }
}

/// 拉取并缓存 provider JWKS（内存缓存 1 小时，避免每请求拉取）
async fn fetch_jwks(jwks_url: &str) -> AppResult<JwkSet> {
    static CACHE: OnceLock<tokio::sync::Mutex<Option<(String, JwkSet, Instant)>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| tokio::sync::Mutex::new(None));
    let mut guard = cache.lock().await;
    if let Some((url, jwks, at)) = guard.as_ref()
        && url == jwks_url
        && at.elapsed() < Duration::from_secs(JWKS_CACHE_TTL_SECS)
    {
        return Ok(jwks.clone());
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(HTTP_TIMEOUT_SECS))
        .build()
        .map_err(|e| AppError::internal(format!("HTTP client build failed: {e}")))?;
    let jwks: JwkSet = client
        .get(jwks_url)
        .send()
        .await
        .map_err(|e| AppError::internal(format!("Failed to fetch JWKS: {e}")))?
        .error_for_status()
        .map_err(|e| AppError::internal(format!("Failed to fetch JWKS: {e}")))?
        .json()
        .await
        .map_err(|e| AppError::internal(format!("Failed to parse JWKS: {e}")))?;

    *guard = Some((jwks_url.to_string(), jwks.clone(), Instant::now()));
    Ok(jwks)
}

/// 验证 OIDC id_token：签名（JWKS）+ aud + iss + exp
async fn verify_id_token(
    token: &str,
    endpoints: &Endpoints,
    client_id: &str,
) -> AppResult<IdClaims> {
    let header = decode_header(token).map_err(|_| AppError::unauthorized("Malformed id_token"))?;
    let jwks = fetch_jwks(&endpoints.jwks_url).await?;
    let kid = header
        .kid
        .as_deref()
        .ok_or_else(|| AppError::unauthorized("id_token missing kid"))?;
    let jwk = jwks
        .find(kid)
        .ok_or_else(|| AppError::unauthorized("Unknown signing key"))?;
    let key =
        DecodingKey::from_jwk(jwk).map_err(|e| AppError::internal(format!("Invalid JWK: {e}")))?;

    let mut validation = Validation::new(header.alg);
    validation.set_audience(&[client_id]);
    validation.set_issuer(&[&endpoints.issuer]);

    decode::<IdClaims>(token, &key, &validation)
        .map(|d| d.claims)
        .map_err(|e| {
            tracing::warn!(error = %e, "id_token verification failed");
            AppError::unauthorized("id_token verification failed")
        })
}

/// 通过 code 向 provider token endpoint 换取 id_token/access_token
async fn exchange_code(
    token_url: &str,
    code: &str,
    provider: &ProviderConfig,
    redirect_uri: &str,
) -> AppResult<TokenResponse> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(HTTP_TIMEOUT_SECS))
        .build()
        .map_err(|e| AppError::internal(format!("HTTP client build failed: {e}")))?;
    let resp = client
        .post(token_url)
        .form(&[
            ("code", code),
            ("client_id", provider.client_id.as_str()),
            ("client_secret", provider.client_secret.as_str()),
            ("redirect_uri", redirect_uri),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await
        .map_err(|e| AppError::internal(format!("Token endpoint unreachable: {e}")))?;

    if !resp.status().is_success() {
        tracing::warn!(status = %resp.status(), "OAuth token exchange rejected");
        return Err(AppError::unauthorized("OAuth authorization failed"));
    }
    resp.json()
        .await
        .map_err(|e| AppError::internal(format!("Invalid token endpoint response: {e}")))
}

impl CredentialService {
    pub async fn sign_in_oauth2(
        ctx: &AppContext,
        params: &SignInOAuth2Request,
    ) -> AppResult<OAuth2UrlResource> {
        let provider = provider_config(ctx, &params.provider_id)?;
        let endpoints = endpoints(&provider);

        let state = CodeUtil::generate_alphanumeric(32);
        let callback = params
            .callback_url
            .clone()
            .unwrap_or_else(|| "http://localhost:5150/api/auth/oauth2/callback".to_string());
        let payload = serde_json::to_string(&StatePayload {
            provider_id: params.provider_id.clone(),
            redirect_uri: callback.clone(),
        })
        .map_err(|e| AppError::internal(format!("Serialize oauth state failed: {e}")))?;
        verification::Entity::create_record(
            &ctx.db,
            &format!("oauth2:state:{state}"),
            &payload,
            900,
        )
        .await?;

        let url = format!(
            "{}?response_type=code&client_id={}&state={state}&redirect_uri={callback}&scope={OAUTH_SCOPE}",
            endpoints.authorize_url, provider.client_id
        );

        Ok(OAuth2UrlResource {
            url,
            redirect: params.redirect.unwrap_or(true),
        })
    }

    pub async fn oauth2_callback(
        ctx: &AppContext,
        params: &OAuth2CallbackRequest,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> AppResult<AuthResource> {
        if let Some(ref err) = params.error {
            return Err(AppError::unauthorized(format!("OAuth error: {err}")));
        }
        let state = params
            .state
            .as_deref()
            .ok_or_else(|| AppError::bad_request("Missing state"))?;
        let code = params
            .code
            .as_deref()
            .ok_or_else(|| AppError::bad_request("Missing code"))?;

        let Some(ver) = verification::Entity::find_valid_by_identifier(
            &ctx.db,
            &format!("oauth2:state:{state}"),
        )
        .await?
        else {
            return Err(AppError::bad_request("Invalid or expired OAuth state"));
        };
        // 校验通过后消费 state，防止重放
        let payload: StatePayload = serde_json::from_str(&ver.value)
            .map_err(|e| AppError::internal(format!("Corrupted oauth state: {e}")))?;
        ver.consume(&ctx.db).await?;

        let provider = provider_config(ctx, &payload.provider_id)?;
        let endpoints = endpoints(&provider);
        let tokens =
            exchange_code(&endpoints.token_url, code, &provider, &payload.redirect_uri).await?;
        let id_token = tokens
            .id_token
            .as_deref()
            .ok_or_else(|| AppError::unauthorized("Provider did not return id_token"))?;
        let claims = verify_id_token(id_token, &endpoints, &provider.client_id).await?;
        let email = claims
            .email
            .ok_or_else(|| AppError::unauthorized("id_token missing email claim"))?;

        // 按 sub（provider 侧稳定 ID）关联账号，其次按 email 关联用户
        let user = if let Some(account) = account::Entity::find_by_provider_and_account(
            &ctx.db,
            &payload.provider_id,
            &claims.sub,
        )
        .await?
        {
            user::Entity::find_by_id(&ctx.db, &account.user_id)
                .await?
                .ok_or_else(|| AppError::internal("OAuth account has no linked user"))?
        } else {
            let user = match user::Entity::find_by_email(&ctx.db, &email).await? {
                Some(u) => u,
                None => {
                    user::Entity::create_user(
                        &ctx.db,
                        &email,
                        claims.name.as_deref().unwrap_or("OAuth User"),
                        None,
                        None,
                        None,
                    )
                    .await?
                },
            };
            account::Entity::create_oauth_account(
                &ctx.db,
                &user.id,
                &payload.provider_id,
                &claims.sub,
                tokens.access_token.as_deref(),
                Some(id_token),
            )
            .await?;
            user
        };

        let user = user
            .update_last_login(&ctx.db, &payload.provider_id)
            .await?;
        let session = session::Entity::create_session(&ctx.db, &user.id, ip, user_agent).await?;

        Ok(AuthResource {
            token: session.token.clone(),
            user: user.into(),
            session: Some(session.into()),
        })
    }

    pub async fn one_tap_callback(
        ctx: &AppContext,
        params: &OneTapCallbackRequest,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> AppResult<AuthResource> {
        let provider = provider_config(ctx, "google")?;
        let endpoints = endpoints(&provider);
        // One Tap 凭据为 Google 签发的 id_token：必须完整验签并校验 aud/iss/exp
        let client_id = params.client_id.as_deref().unwrap_or(&provider.client_id);
        let claims = verify_id_token(&params.credential, &endpoints, client_id).await?;
        let email = claims
            .email
            .ok_or_else(|| AppError::unauthorized("id_token missing email claim"))?;

        let user = if let Some(account) =
            account::Entity::find_by_provider_and_account(&ctx.db, "google", &claims.sub).await?
        {
            user::Entity::find_by_id(&ctx.db, &account.user_id)
                .await?
                .ok_or_else(|| AppError::internal("OAuth account has no linked user"))?
        } else {
            let user = match user::Entity::find_by_email(&ctx.db, &email).await? {
                Some(u) => u,
                None => {
                    user::Entity::create_user(
                        &ctx.db,
                        &email,
                        claims.name.as_deref().unwrap_or("OneTap User"),
                        None,
                        None,
                        None,
                    )
                    .await?
                },
            };
            account::Entity::create_oauth_account(
                &ctx.db,
                &user.id,
                "google",
                &claims.sub,
                None,
                Some(&params.credential),
            )
            .await?;
            user
        };

        let user = user.update_last_login(&ctx.db, "google").await?;
        let session = session::Entity::create_session(&ctx.db, &user.id, ip, user_agent).await?;

        Ok(AuthResource {
            token: session.token.clone(),
            user: user.into(),
            session: Some(session.into()),
        })
    }
}
