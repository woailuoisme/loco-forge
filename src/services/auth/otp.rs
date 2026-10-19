use axum::http::StatusCode;
use loco_rs::prelude::*;
use secrecy::ExposeSecret;
use uuid::Uuid;

use crate::{
    common::consts,
    common::utils::code::CodeUtil,
    dtos::auth::*,
    errors::{AppError, AppResult},
    models::{account, session, user, verification},
};

use super::VerificationService;

impl VerificationService {
    /// 发送邮箱验证码 OTP
    pub async fn send_email_otp(
        db: &DatabaseConnection,
        params: &SendEmailOtpRequest,
    ) -> AppResult<StatusResource> {
        let email = params.email.trim().to_lowercase();
        let otp = CodeUtil::generate_otp();
        let identifier = format!("email-otp:{}:{}", params.otp_type, email);

        verification::Entity::create_record(db, &identifier, &otp, consts::auth::OTP_TTL_SECS)
            .await?;

        // 安全约束：绝不记录 OTP 明文
        tracing::info!(email = %email, otp_type = %params.otp_type, "Email OTP generated and saved");
        Ok(StatusResource::ok())
    }

    /// 检查邮箱 OTP 验证码有效性（不消耗该验证码，但计入失败锁定）
    pub async fn check_email_otp(
        db: &DatabaseConnection,
        params: &CheckEmailOtpRequest,
    ) -> AppResult<StatusResource> {
        let email = params.email.trim().to_lowercase();
        let identifier = format!("email-otp:{}:{}", params.otp_type, email);
        ensure_otp_unlocked(db, &identifier).await?;

        if verification::Entity::peek_valid(db, &identifier, &params.otp).await? {
            Ok(StatusResource::ok())
        } else {
            verification::Entity::record_otp_failure(db, &identifier).await?;
            Err(AppError::bad_request("Invalid or expired OTP"))
        }
    }

    /// 邮箱 OTP 登录（验证码校验通过后自动创建用户或登录，并签发 Session）
    pub async fn sign_in_email_otp(
        db: &DatabaseConnection,
        params: &SignInEmailOtpRequest,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> AppResult<AuthResource> {
        let email = params.email.trim().to_lowercase();
        let identifier = format!("email-otp:sign-in:{email}");

        // 原子校验并消费 OTP，失败计入暴力破解锁定
        ensure_otp_unlocked(db, &identifier).await?;
        if !verification::Entity::consume(db, &identifier, &params.otp).await? {
            verification::Entity::record_otp_failure(db, &identifier).await?;
            return Err(AppError::bad_request("Invalid or expired OTP"));
        }

        // 查找或自动注册用户
        let user = match user::Entity::find_by_email(db, &email).await? {
            Some(u) => {
                if !u.email_verified {
                    u.set_email_verified(db, true).await?
                } else {
                    u
                }
            },
            None => {
                let display_name = params.name.as_deref().unwrap_or(&email);
                let new_user = user::Entity::create_user(
                    db,
                    &email,
                    display_name,
                    None,
                    params.image.clone(),
                    None,
                )
                .await?;
                new_user.set_email_verified(db, true).await?
            },
        };

        if user.is_banned() {
            return Err(AppError::forbidden("Account is banned"));
        }

        let session = session::Entity::create_session(db, &user.id, ip, user_agent).await?;
        let user = user.update_last_login(db, "email-otp").await?;

        let token = session.token.clone();
        let session_res = SessionResource::from(session);
        let user_res = UserResource::from(user);

        Ok(AuthResource {
            token,
            user: user_res,
            session: Some(session_res),
        })
    }

    /// 验证邮箱 (verify-email)
    pub async fn verify_email_otp(
        db: &DatabaseConnection,
        params: &VerifyEmailOtpRequest,
    ) -> AppResult<VerifyEmailOtpResource> {
        let email = params.email.trim().to_lowercase();
        let identifier = format!("email-otp:email-verification:{email}");

        ensure_otp_unlocked(db, &identifier).await?;
        if !verification::Entity::consume(db, &identifier, &params.otp).await? {
            verification::Entity::record_otp_failure(db, &identifier).await?;
            return Err(AppError::bad_request("Invalid or expired OTP"));
        }

        let user = user::Entity::find_by_email(db, &email).await?;
        let Some(user) = user else {
            return Err(AppError::not_found("User not found"));
        };

        let updated_user = user.set_email_verified(db, true).await?;
        Ok(VerifyEmailOtpResource {
            status: true,
            user: UserResource::from(updated_user),
        })
    }

    /// 重置密码 (reset-password)
    pub async fn reset_password_email_otp(
        db: &DatabaseConnection,
        params: &ResetPasswordEmailOtpRequest,
    ) -> AppResult<StatusResource> {
        let email = params.email.trim().to_lowercase();
        let identifier = format!("email-otp:forget-password:{email}");

        ensure_otp_unlocked(db, &identifier).await?;
        if !verification::Entity::consume(db, &identifier, &params.otp).await? {
            verification::Entity::record_otp_failure(db, &identifier).await?;
            return Err(AppError::bad_request("Invalid or expired OTP"));
        }

        let user = user::Entity::find_by_email(db, &email).await?;
        let Some(user) = user else {
            return Err(AppError::not_found("User not found"));
        };

        // 已有密码凭据则更新，否则创建；重置后吊销全部会话防止旧凭据残留
        if let Some(credential) = account::Entity::find_credential_by_user(db, &user.id).await? {
            credential
                .update_password(db, params.password.expose_secret())
                .await?;
        } else {
            account::Entity::create_credential(
                db,
                &user.id,
                &user.email,
                params.password.expose_secret(),
            )
            .await?;
        }
        session::Entity::revoke_all_sessions(db, &user.id).await?;

        Ok(StatusResource::ok())
    }

    /// 发送手机号短信验证码
    pub async fn send_phone_otp(
        db: &DatabaseConnection,
        params: &SendPhoneOtpRequest,
    ) -> AppResult<StatusResource> {
        let phone = params.phone_number.trim();
        let otp = CodeUtil::generate_otp();
        let identifier = format!("phone-otp:{phone}");

        verification::Entity::create_record(db, &identifier, &otp, consts::auth::OTP_TTL_SECS)
            .await?;

        // 安全约束：绝不记录 OTP 明文
        tracing::info!(phone_number = %phone, "SMS OTP generated");
        Ok(StatusResource::ok())
    }

    /// 校验手机号短信验证码 (支持登录/注册或修改手机号)
    pub async fn verify_phone_number(
        db: &DatabaseConnection,
        current_user_id: Option<&str>,
        params: &VerifyPhoneNumberRequest,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> AppResult<SignInResponse> {
        let phone = params.phone_number.trim();
        let identifier = format!("phone-otp:{phone}");

        ensure_otp_unlocked(db, &identifier).await?;
        if !verification::Entity::consume(db, &identifier, &params.code).await? {
            verification::Entity::record_otp_failure(db, &identifier).await?;
            return Err(AppError::bad_request(
                "Invalid or expired verification code",
            ));
        }

        // 修改手机号模式 (需已登录)
        if params.update_phone_number == Some(true) {
            let Some(user_id) = current_user_id else {
                return Err(AppError::unauthorized(
                    "Must be logged in to update phone number",
                ));
            };
            let user = user::Entity::find_by_id(db, user_id).await?;
            let Some(user) = user else {
                return Err(AppError::not_found("User not found"));
            };
            let updated = user.set_phone_number(db, phone, true).await?;
            return Ok(SignInResponse::from(AuthResource {
                token: String::new(),
                user: UserResource::from(updated),
                session: None,
            }));
        }

        // 登录/注册模式
        let user = match user::Entity::find_by_phone_number(db, phone).await? {
            Some(u) => u,
            None => user::Entity::create_phone_user(db, phone).await?,
        };

        if user.is_banned() {
            return Err(AppError::forbidden("Account is banned"));
        }

        // 若要求禁用自动会话，仅返回用户信息
        if params.disable_session == Some(true) {
            return Ok(SignInResponse::from(AuthResource {
                token: String::new(),
                user: UserResource::from(user),
                session: None,
            }));
        }

        let session = session::Entity::create_session(db, &user.id, ip, user_agent).await?;
        let user = user.update_last_login(db, "phone-number").await?;

        let token = session.token.clone();
        let session_res = SessionResource::from(session);
        let user_res = UserResource::from(user);

        Ok(SignInResponse::from(AuthResource {
            token,
            user: user_res,
            session: Some(session_res),
        }))
    }

    /// 手机号密码登录 (支持 2FA 拦截)
    pub async fn sign_in_phone_number(
        db: &DatabaseConnection,
        params: &SignInPhoneNumberRequest,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> AppResult<(SignInResponse, Option<String>)> {
        let phone = params.phone_number.trim();
        let user = user::Entity::find_by_phone_number(db, phone).await?;
        let Some(user) = user else {
            tracing::debug!(phone = %phone, "sign-in attempt with non-existent phone number");
            return Err(AppError::unauthorized("Invalid phone number or password"));
        };

        if user.is_banned() {
            return Err(AppError::forbidden("Account is banned"));
        }

        let account = account::Entity::find_credential_by_user(db, &user.id).await?;
        let Some(account) = account else {
            tracing::debug!(user_id = %user.id, "user has no credential account");
            return Err(AppError::unauthorized("Invalid phone number or password"));
        };

        if !account.verify_password(params.password.expose_secret()) {
            tracing::debug!(user_id = %user.id, "password verification failed");
            return Err(AppError::unauthorized("Invalid phone number or password"));
        }

        // 若用户开启了双因素认证，签发 2FA Challenge 临时凭据
        if user.two_factor_enabled {
            let challenge_token = Uuid::now_v7().to_string();
            let identifier = format!("2fa-challenge:{challenge_token}");
            verification::Entity::create_record(
                db,
                &identifier,
                &user.id,
                consts::auth::VERIFICATION_TTL_SECS,
            )
            .await?;

            return Ok((
                SignInResponse::from(TwoFactorRedirectResource::default()),
                Some(challenge_token),
            ));
        }

        let session = session::Entity::create_session(db, &user.id, ip, user_agent).await?;
        let user = user.update_last_login(db, "phone-number").await?;

        let token = session.token.clone();
        let session_res = SessionResource::from(session);
        let user_res = UserResource::from(user);

        Ok((
            SignInResponse::from(AuthResource {
                token,
                user: user_res,
                session: Some(session_res),
            }),
            None,
        ))
    }
}

/// OTP 暴力破解防护：校验失败计数未达锁定阈值才放行
async fn ensure_otp_unlocked(db: &DatabaseConnection, identifier: &str) -> AppResult<()> {
    if verification::Entity::otp_is_locked(db, identifier).await? {
        return Err(AppError::Custom(
            StatusCode::TOO_MANY_REQUESTS,
            "Too many failed attempts, please retry later".to_string(),
        ));
    }
    Ok(())
}
