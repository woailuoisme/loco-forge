use chrono::Utc;
use loco_rs::prelude::*;
use totp_rs::{Algorithm, Builder as TotpBuilder, Secret};

use crate::{
    dtos::auth::*,
    errors::{AppError, AppResult},
    models::{account, session, two_factor, user, verification},
};

use super::VerificationService;

impl VerificationService {
    /// 启用两步验证：生成 Secret、10 组备用恢复码与 TOTP URI
    pub async fn enable_two_factor(
        db: &DatabaseConnection,
        user_id: &str,
        password: &str,
    ) -> AppResult<TwoFactorEnableResource> {
        let account = account::Entity::find_credential_by_user(db, user_id).await?;
        let Some(account) = account else {
            return Err(AppError::unauthorized("Password authentication required"));
        };

        if !account.verify_password(password) {
            return Err(AppError::unauthorized("Invalid password"));
        }

        let user = user::Entity::find_by_id(db, user_id).await?;
        let Some(user) = user else {
            return Err(AppError::not_found("User not found"));
        };

        let secret = Secret::generate();
        let secret_base32 = secret.to_base32();
        let backup_codes = two_factor::Entity::generate_backup_codes(10);

        let totp = TotpBuilder::new()
            .with_algorithm(Algorithm::SHA1)
            .with_digits(6)
            .with_skew(1)
            .with_step_duration(30)
            .with_secret(secret)
            .with_account_name(user.email.as_str())
            .with_issuer(Some("Loco App"))
            .build()
            .map_err(|e| AppError::internal(format!("Failed to build TOTP: {e}")))?;

        let totp_uri = totp
            .to_url()
            .map_err(|e| AppError::internal(format!("Failed to get TOTP URL: {e}")))?;

        // 入库待激活 2FA 凭据
        two_factor::Entity::upsert_pending(db, user_id, &secret_base32, &backup_codes).await?;

        Ok(TwoFactorEnableResource {
            totp_uri,
            backup_codes,
        })
    }

    /// 校验 TOTP 动态码 (支持已登录用户首次绑定校验，或未登录用户的登录挑战校验)
    pub async fn verify_totp(
        db: &DatabaseConnection,
        authenticated_user_id: Option<&str>,
        challenge_token: Option<&str>,
        code: &str,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> AppResult<SignInResponse> {
        // 场景 A: 已登录用户激活绑定 2FA
        if let Some(user_id) = authenticated_user_id {
            let tf = two_factor::Entity::find_by_user_id(db, user_id).await?;
            let Some(tf) = tf else {
                return Err(AppError::bad_request(
                    "2FA has not been initiated for this account",
                ));
            };

            if tf.is_locked() {
                return Err(AppError::forbidden(
                    "2FA verification is locked due to too many failed attempts",
                ));
            }

            if !tf.verify_totp_code(code) {
                tf.record_failed_attempt(db).await?;
                return Err(AppError::bad_request("Invalid TOTP code"));
            }

            // 激活 2FA（绑定确认与用户标志必须原子）
            let user = user::Entity::find_by_id(db, user_id)
                .await?
                .ok_or_else(|| AppError::not_found("User not found"))?;
            let updated = db
                .transaction_async(async |txn| {
                    two_factor::Entity::confirm_enabled(txn, user_id).await?;
                    user.set_two_factor_enabled(txn, true)
                        .await
                        .map_err(AppError::from)
                })
                .await
                .map_err(|err| match err {
                    sea_orm::TransactionError::Connection(db_err) => AppError::from(db_err),
                    sea_orm::TransactionError::Transaction(app_err) => app_err,
                })?;

            return Ok(SignInResponse::from(AuthResource {
                token: String::new(),
                user: UserResource::from(updated),
                session: None,
            }));
        }

        // 场景 B: 未登录用户登录 2FA 挑战校验
        let Some(tok) = challenge_token else {
            return Err(AppError::unauthorized("Missing 2FA challenge token"));
        };

        let identifier = format!("2fa-challenge:{tok}");
        // 针对 challenge，value 存放 user_id，因此只需按 identifier 查找
        let now = Utc::now();
        let record = verification::Entity::find()
            .filter(verification::Column::Identifier.eq(&identifier))
            .filter(verification::Column::ExpiresAt.gt(now))
            .one(db)
            .await?;

        let Some(record) = record else {
            return Err(AppError::unauthorized(
                "Invalid or expired 2FA challenge token",
            ));
        };

        let user_id = record.value;
        let tf = two_factor::Entity::find_by_user_id(db, &user_id).await?;
        let Some(tf) = tf else {
            return Err(AppError::bad_request("2FA not configured for user"));
        };

        if tf.is_locked() {
            return Err(AppError::forbidden(
                "2FA verification is locked due to too many failed attempts",
            ));
        }

        if !tf.verify_totp_code(code) {
            tf.record_failed_attempt(db).await?;
            return Err(AppError::bad_request("Invalid TOTP code"));
        }

        // 校验成功，原子消费 challenge 凭据（防重放）
        verification::Entity::delete_by_id(record.id)
            .exec(db)
            .await?;

        let user = user::Entity::find_by_id(db, &user_id).await?;
        let Some(user) = user else {
            return Err(AppError::not_found("User not found"));
        };

        let session = session::Entity::create_session(db, &user.id, ip, user_agent).await?;
        let user = user.update_last_login(db, "two-factor").await?;

        let token = session.token.clone();
        let session_res = SessionResource::from(session);
        let user_res = UserResource::from(user);

        Ok(SignInResponse::from(AuthResource {
            token,
            user: user_res,
            session: Some(session_res),
        }))
    }

    /// 校验单次备用恢复码完成挑战
    pub async fn verify_backup_code(
        db: &DatabaseConnection,
        authenticated_user_id: Option<&str>,
        challenge_token: Option<&str>,
        code: &str,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> AppResult<SignInResponse> {
        let user_id = if let Some(uid) = authenticated_user_id {
            uid.to_string()
        } else if let Some(tok) = challenge_token {
            let identifier = format!("2fa-challenge:{tok}");
            let now = Utc::now();
            let record = verification::Entity::find()
                .filter(verification::Column::Identifier.eq(&identifier))
                .filter(verification::Column::ExpiresAt.gt(now))
                .one(db)
                .await?;

            let Some(record) = record else {
                return Err(AppError::unauthorized(
                    "Invalid or expired 2FA challenge token",
                ));
            };
            let uid = record.value.clone();
            verification::Entity::delete_by_id(record.id)
                .exec(db)
                .await?;
            uid
        } else {
            return Err(AppError::unauthorized(
                "Authentication or challenge token required",
            ));
        };

        let tf = two_factor::Entity::find_by_user_id(db, &user_id).await?;
        let Some(tf) = tf else {
            return Err(AppError::bad_request("2FA not configured for user"));
        };

        if tf.is_locked() {
            return Err(AppError::forbidden(
                "2FA verification is locked due to too many failed attempts",
            ));
        }

        let consumed = tf.verify_and_consume_backup_code(db, code).await?;
        if !consumed {
            tf.record_failed_attempt(db).await?;
            return Err(AppError::bad_request("Invalid backup code"));
        }

        let user = user::Entity::find_by_id(db, &user_id).await?;
        let Some(user) = user else {
            return Err(AppError::not_found("User not found"));
        };

        let session = session::Entity::create_session(db, &user.id, ip, user_agent).await?;
        let user = user.update_last_login(db, "two-factor-backup").await?;

        let token = session.token.clone();
        let session_res = SessionResource::from(session);
        let user_res = UserResource::from(user);

        Ok(SignInResponse::from(AuthResource {
            token,
            user: user_res,
            session: Some(session_res),
        }))
    }

    /// 禁用两步验证
    pub async fn disable_two_factor(
        db: &DatabaseConnection,
        user_id: &str,
        password: &str,
    ) -> AppResult<StatusResource> {
        let account = account::Entity::find_credential_by_user(db, user_id).await?;
        let Some(account) = account else {
            return Err(AppError::unauthorized("Password authentication required"));
        };

        if !account.verify_password(password) {
            return Err(AppError::unauthorized("Invalid password"));
        }

        let user = user::Entity::find_by_id(db, user_id).await?;
        let Some(user) = user else {
            return Err(AppError::not_found("User not found"));
        };

        // 解绑记录与用户标志必须原子，避免出现标志残留的不一致状态
        db.transaction_async(async |txn| {
            two_factor::Entity::disable_for_user(txn, user_id).await?;
            user.set_two_factor_enabled(txn, false).await?;
            Ok::<_, AppError>(())
        })
        .await
        .map_err(|err| match err {
            sea_orm::TransactionError::Connection(db_err) => AppError::from(db_err),
            sea_orm::TransactionError::Transaction(app_err) => app_err,
        })?;

        Ok(StatusResource::ok())
    }

    /// 重新生成 10 组备用恢复码 (覆盖原恢复码)
    pub async fn generate_backup_codes(
        db: &DatabaseConnection,
        user_id: &str,
        password: &str,
    ) -> AppResult<TwoFactorBackupCodesResource> {
        let account = account::Entity::find_credential_by_user(db, user_id).await?;
        let Some(account) = account else {
            return Err(AppError::unauthorized("Password authentication required"));
        };

        if !account.verify_password(password) {
            return Err(AppError::unauthorized("Invalid password"));
        }

        let new_codes = two_factor::Entity::generate_backup_codes(10);
        two_factor::Entity::update_backup_codes(db, user_id, &new_codes).await?;

        Ok(TwoFactorBackupCodesResource {
            backup_codes: new_codes,
        })
    }

    // 1. Passkey / WebAuthn
}
