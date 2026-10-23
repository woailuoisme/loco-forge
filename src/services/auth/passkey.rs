use loco_rs::prelude::*;
use webauthn_rs::prelude::{
    Passkey as WPasskey, PasskeyAuthentication, PasskeyRegistration, PublicKeyCredential,
    RegisterPublicKeyCredential, Webauthn, WebauthnBuilder,
};

use crate::{
    common::consts::auth::{PASSKEY_ORIGIN, PASSKEY_RP_ID, PASSKEY_RP_NAME, VERIFICATION_TTL_SECS},
    dtos::auth::*,
    errors::{AppError, AppResult},
    models::{passkey, session, user, verification},
};

use super::CredentialService;

/// Passkey challenge 状态在 verification 表中的 identifier 键名
const STATE_KEY_AUTH: &str = "passkey:webauthn:auth:";
const STATE_KEY_REG: &str = "passkey:webauthn:reg:";

/// 从应用配置构建 WebAuthn 实例 (settings.passkey.rp_id/origin，支持环境变量覆盖)
fn webauthn_instance(ctx: &AppContext) -> AppResult<Webauthn> {
    let settings = ctx.config.settings.as_ref().and_then(|s| s.get("passkey"));
    let read = |key: &str| -> Option<String> {
        settings
            .and_then(|p| p.get(key))
            .and_then(|v| v.as_str())
            .map(String::from)
    };
    let rp_id = read("rp_id")
        .or_else(|| std::env::var("PASSKEY_RP_ID").ok())
        .unwrap_or_else(|| PASSKEY_RP_ID.to_string());
    let origin = read("origin")
        .or_else(|| std::env::var("PASSKEY_ORIGIN").ok())
        .unwrap_or_else(|| PASSKEY_ORIGIN.to_string());

    let origin_url = url::Url::parse(&origin)
        .map_err(|e| AppError::internal(format!("Invalid passkey origin in config: {e}")))?;
    WebauthnBuilder::new(&rp_id, &origin_url)
        .and_then(|b| b.rp_name(PASSKEY_RP_NAME).build())
        .map_err(|e| {
            tracing::error!(error = %e, "Invalid webauthn configuration");
            AppError::internal("Invalid webauthn configuration")
        })
}

/// 读取并反序列化 challenge 状态记录
async fn load_state(db: &DatabaseConnection, identifier: &str) -> AppResult<verification::Model> {
    verification::Entity::find_valid_by_identifier(db, identifier)
        .await?
        .ok_or_else(|| AppError::bad_request("Passkey challenge expired or not found"))
}

impl CredentialService {
    /// 生成 Passkey 认证挑战（基于用户已注册凭据发起 WebAuthn 断言）
    pub async fn generate_passkey_auth_options(
        ctx: &AppContext,
        params: &PasskeyGenerateAuthOptionsRequest,
    ) -> AppResult<serde_json::Value> {
        let webauthn = webauthn_instance(ctx)?;
        let email = params.email.trim().to_lowercase();

        let Some(user) = user::Entity::find_by_email(&ctx.db, &email).await? else {
            // 统一文案，避免泄露用户是否存在
            return Err(AppError::not_found("No passkey credentials found"));
        };
        let models = passkey::Entity::find_by_user_id(&ctx.db, &user.id).await?;

        // 单条损坏记录不阻断登录选项生成（跳过并告警），但全部损坏则拒绝
        let creds: Vec<WPasskey> = models
            .iter()
            .filter_map(|m| match serde_json::from_str::<WPasskey>(&m.public_key) {
                Ok(pk) => Some(pk),
                Err(e) => {
                    tracing::warn!(passkey_id = %m.id, error = %e, "Skipping corrupted passkey state");
                    None
                },
            })
            .collect();
        if creds.is_empty() {
            return Err(AppError::not_found("No passkey credentials found"));
        }

        let (rcr, state) = webauthn.start_passkey_authentication(&creds).map_err(|e| {
            tracing::error!(error = %e, "Failed to start passkey authentication");
            AppError::internal("Failed to start passkey authentication")
        })?;

        verification::Entity::create_record(
            &ctx.db,
            &format!("{STATE_KEY_AUTH}{}", user.id),
            &serde_json::to_string(&state)
                .map_err(|e| AppError::internal(format!("Serialize passkey state failed: {e}")))?,
            VERIFICATION_TTL_SECS,
        )
        .await?;

        serde_json::to_value(rcr)
            .map_err(|e| AppError::internal(format!("Serialize challenge failed: {e}")))
    }

    /// 校验 Passkey 认证签名（完整 WebAuthn assertion 验证）并签发会话
    pub async fn verify_passkey_auth(
        ctx: &AppContext,
        params: &PasskeyVerifyAuthRequest,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> AppResult<AuthResource> {
        let webauthn = webauthn_instance(ctx)?;
        let cred_id = params
            .response
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::bad_request("Missing credential ID"))?;

        let pk = passkey::Entity::find_by_credential_id(&ctx.db, cred_id)
            .await?
            .ok_or_else(|| AppError::unauthorized("Passkey credential not recognized"))?;

        let record = load_state(&ctx.db, &format!("{STATE_KEY_AUTH}{}", pk.user_id)).await?;
        let state: PasskeyAuthentication = serde_json::from_str(&record.value)
            .map_err(|e| AppError::internal(format!("Corrupted passkey state: {e}")))?;

        let assertion: PublicKeyCredential = serde_json::from_value(params.response.clone())
            .map_err(|e| {
                AppError::bad_request(format!("Invalid WebAuthn assertion format: {e}"))
            })?;

        let result = webauthn
            .finish_passkey_authentication(&assertion, &state)
            .map_err(|e| {
                tracing::warn!(error = %e, "Passkey assertion verification failed");
                AppError::unauthorized("Passkey verification failed")
            })?;

        // 校验通过后消费 challenge，防止重放
        record.consume(&ctx.db).await?;

        // 按 WebAuthn 规范维护签名计数器；需要时同步凭据属性（备份位等）
        let mut credential: WPasskey = serde_json::from_str(&pk.public_key)
            .map_err(|e| AppError::internal(format!("Corrupted passkey state: {e}")))?;
        let refreshed = (credential.update_credential(&result) == Some(true))
            .then(|| serde_json::to_string(&credential).ok())
            .flatten();
        pk.apply_authentication_result(&ctx.db, i64::from(result.counter()), refreshed)
            .await?;

        let user = user::Entity::find_by_id(&ctx.db, &pk.user_id)
            .await?
            .ok_or_else(|| AppError::not_found("User not found"))?;
        let user = user.update_last_login(&ctx.db, "passkey").await?;

        let session = session::Entity::create_session(&ctx.db, &user.id, ip, user_agent).await?;

        Ok(AuthResource {
            token: session.token.clone(),
            user: user.into(),
            session: Some(session.into()),
        })
    }

    /// 生成 Passkey 注册挑战（WebAuthn attestation 选项）
    pub async fn generate_passkey_reg_options(
        ctx: &AppContext,
        user_id: &str,
    ) -> AppResult<serde_json::Value> {
        let webauthn = webauthn_instance(ctx)?;
        let user = user::Entity::find_by_id(&ctx.db, user_id)
            .await?
            .ok_or_else(|| AppError::not_found("User not found"))?;

        let existing = passkey::Entity::find_by_user_id(&ctx.db, user_id).await?;
        let exclude = existing
            .iter()
            .map(|m| serde_json::from_str::<WPasskey>(&m.public_key))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::internal(format!("Corrupted passkey state: {e}")))?;

        let user_unique_id = uuid::Uuid::parse_str(&user.id)
            .map_err(|e| AppError::internal(format!("User id is not a valid UUID: {e}")))?;

        let (ccr, state) = webauthn
            .start_passkey_registration(
                user_unique_id,
                &user.email,
                &user.name,
                (!exclude.is_empty())
                    .then(|| exclude.iter().map(|p| p.cred_id().clone()).collect()),
            )
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to start passkey registration");
                AppError::internal("Failed to start passkey registration")
            })?;

        verification::Entity::create_record(
            &ctx.db,
            &format!("{STATE_KEY_REG}{user_id}"),
            &serde_json::to_string(&state)
                .map_err(|e| AppError::internal(format!("Serialize passkey state failed: {e}")))?,
            VERIFICATION_TTL_SECS,
        )
        .await?;

        serde_json::to_value(ccr)
            .map_err(|e| AppError::internal(format!("Serialize challenge failed: {e}")))
    }

    /// 校验 Passkey 注册（完整 attestation 验证）并持久化凭据
    pub async fn verify_passkey_reg(
        ctx: &AppContext,
        user_id: &str,
        params: &PasskeyVerifyRegRequest,
    ) -> AppResult<PasskeyResource> {
        let webauthn = webauthn_instance(ctx)?;

        let reg: RegisterPublicKeyCredential = serde_json::from_value(params.response.clone())
            .map_err(|e| {
                AppError::bad_request(format!("Invalid WebAuthn registration response: {e}"))
            })?;

        let record = load_state(&ctx.db, &format!("{STATE_KEY_REG}{user_id}")).await?;
        let state: PasskeyRegistration = serde_json::from_str(&record.value)
            .map_err(|e| AppError::internal(format!("Corrupted passkey state: {e}")))?;

        let registered = webauthn
            .finish_passkey_registration(&reg, &state)
            .map_err(|e| {
                tracing::warn!(error = %e, "Passkey registration verification failed");
                AppError::bad_request("Passkey registration verification failed")
            })?;

        // 校验通过后消费 challenge，防止重放
        record.consume(&ctx.db).await?;

        // 同一 credential 不得注册到多个账号
        if passkey::Entity::find_by_credential_id(&ctx.db, &reg.id)
            .await?
            .is_some()
        {
            return Err(AppError::conflict("Credential already registered"));
        }

        let state_json = serde_json::to_string(&registered)
            .map_err(|e| AppError::internal(format!("Serialize passkey failed: {e}")))?;
        let pk = passkey::Entity::create_passkey(
            &ctx.db,
            user_id,
            &reg.id,
            &state_json,
            0,
            params.name.clone(),
            Some("platform".to_string()),
            false,
            Some("internal".to_string()),
            None,
        )
        .await?;

        Ok(pk.into())
    }

    pub async fn list_user_passkeys(
        db: &DatabaseConnection,
        user_id: &str,
    ) -> AppResult<Vec<PasskeyResource>> {
        let passkeys = passkey::Entity::find_by_user_id(db, user_id).await?;
        Ok(passkeys.into_iter().map(Into::into).collect())
    }

    pub async fn delete_passkey(
        db: &DatabaseConnection,
        user_id: &str,
        params: &DeletePasskeyRequest,
    ) -> AppResult<StatusResource> {
        let deleted = passkey::Entity::delete_by_id_for_user(db, &params.id, user_id).await?;
        if !deleted {
            return Err(AppError::not_found("Passkey not found"));
        }
        Ok(StatusResource::ok())
    }

    pub async fn update_passkey(
        db: &DatabaseConnection,
        user_id: &str,
        params: &UpdatePasskeyRequest,
    ) -> AppResult<PasskeyResource> {
        let updated =
            passkey::Entity::update_name_for_user(db, &params.id, user_id, &params.name).await?;
        if !updated {
            return Err(AppError::not_found("Passkey not found"));
        }
        let Some(pk) = passkey::Entity::find()
            .filter(passkey::Column::Id.eq(&params.id))
            .filter(passkey::Column::UserId.eq(user_id))
            .one(db)
            .await?
        else {
            return Err(AppError::not_found("Passkey not found"));
        };
        Ok(pk.into())
    }
}
