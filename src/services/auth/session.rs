use loco_rs::prelude::*;
use secrecy::ExposeSecret;
use uuid::Uuid;

use crate::{
    common::consts,
    dtos::auth::*,
    errors::{AppError, AppResult},
    models::{account, session, user, verification},
};

use super::SessionService;

impl SessionService {
    /// 邮箱注册业务逻辑
    ///
    /// 注册新用户并自动签发初始活跃会话与凭据，返回认证载荷
    pub async fn sign_up_email(
        db: &DatabaseConnection,
        params: &SignUpEmailRequest,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> AppResult<AuthResource> {
        let email = params.email.trim().to_lowercase();

        // 唯一性校验：邮箱是否已存在
        if user::Entity::find_by_email(db, &email).await?.is_some() {
            return Err(AppError::conflict("User with this email already exists"));
        }

        // 唯一性校验：若提供了用户名，用户名是否已存在
        if let Some(ref uname) = params.username {
            let uname = uname.trim().to_lowercase();
            if user::Entity::find_by_username(db, &uname).await?.is_some() {
                return Err(AppError::conflict("Username is already in use"));
            }
        }

        // 建号 / 凭据 / 会话必须原子：任一步失败都回滚，避免留下无凭据孤儿用户
        let (user, session) = db
            .transaction_async(async |txn| {
                // 创建新用户记录（并发下唯一索引兜底 → 409）
                let user = user::Entity::create_user(
                    txn,
                    &email,
                    &params.name,
                    None,
                    params.image.clone(),
                    params.username.clone(),
                )
                .await
                .map_err(|e| {
                    if AppError::is_unique_violation(&e) {
                        AppError::conflict("User with this email already exists")
                    } else {
                        AppError::from(e)
                    }
                })?;

                // 创建密码凭据账户 (provider_id = "credential")
                account::Entity::create_credential(
                    txn,
                    &user.id,
                    &user.email,
                    params.password.expose_secret(),
                )
                .await?;

                // 自动签发初始持久化会话
                let session =
                    session::Entity::create_session(txn, &user.id, ip, user_agent).await?;

                Ok::<_, AppError>((user, session))
            })
            .await
            .map_err(|err| match err {
                sea_orm::TransactionError::Connection(db_err) => {
                    if AppError::is_unique_violation(&db_err) {
                        AppError::conflict("User with this email already exists")
                    } else {
                        AppError::from(db_err)
                    }
                },
                sea_orm::TransactionError::Transaction(app_err) => app_err,
            })?;

        // 更新最后登录记录
        let user = user.update_last_login(db, "email").await?;

        let token = session.token.clone();
        let session_res = SessionResource::from(session);
        let user_res = UserResource::from(user);

        Ok(AuthResource {
            token,
            user: user_res,
            session: Some(session_res),
        })
    }

    /// 邮箱密码登录业务逻辑 (支持 2FA 拦截与 Challenge 签发)
    pub async fn sign_in_email(
        db: &DatabaseConnection,
        params: &SignInEmailRequest,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> AppResult<(SignInResponse, Option<String>)> {
        let email = params.email.trim().to_lowercase();
        let user = user::Entity::find_by_email(db, &email).await?;
        let Some(user) = user else {
            tracing::debug!(email = %email, "sign-in attempt with non-existent email");
            return Err(AppError::unauthorized("Invalid email or password"));
        };

        // 使用 is_banned() 而非裸 banned 字段：前者判断 ban_expires 是否已到期，
        // 使「临时封禁已过期」的账号自动恢复登录能力。
        if user.is_banned() {
            let reason = user
                .ban_reason
                .clone()
                .unwrap_or_else(|| "Account is banned".to_string());
            return Err(AppError::forbidden(reason));
        }

        let account = account::Entity::find_credential_by_user(db, &user.id).await?;
        let Some(account) = account else {
            tracing::debug!(user_id = %user.id, "user has no credential account");
            return Err(AppError::unauthorized("Invalid email or password"));
        };

        if !account.verify_password(params.password.expose_secret()) {
            tracing::debug!(user_id = %user.id, "password verification failed");
            return Err(AppError::unauthorized("Invalid email or password"));
        }

        if user.two_factor_enabled {
            let challenge_token = issue_2fa_challenge(db, &user.id).await?;
            return Ok((
                SignInResponse::from(TwoFactorRedirectResource::default()),
                Some(challenge_token),
            ));
        }

        finish_sign_in(db, user, "email", ip, user_agent).await
    }

    /// 用户名密码登录业务逻辑 (支持 2FA 拦截与 Challenge 签发)
    pub async fn sign_in_username(
        db: &DatabaseConnection,
        params: &SignInUsernameRequest,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> AppResult<(SignInResponse, Option<String>)> {
        let username = params.username.trim().to_lowercase();
        let user = user::Entity::find_by_username(db, &username).await?;
        let Some(user) = user else {
            tracing::debug!(username = %username, "sign-in attempt with non-existent username");
            return Err(AppError::unauthorized("Invalid username or password"));
        };

        if user.is_banned() {
            let reason = user
                .ban_reason
                .clone()
                .unwrap_or_else(|| "Account is banned".to_string());
            return Err(AppError::forbidden(reason));
        }

        let account = account::Entity::find_credential_by_user(db, &user.id).await?;
        let Some(account) = account else {
            tracing::debug!(user_id = %user.id, "user has no credential account");
            return Err(AppError::unauthorized("Invalid username or password"));
        };

        if !account.verify_password(params.password.expose_secret()) {
            tracing::debug!(user_id = %user.id, "password verification failed");
            return Err(AppError::unauthorized("Invalid username or password"));
        }

        if user.two_factor_enabled {
            let challenge_token = issue_2fa_challenge(db, &user.id).await?;
            return Ok((
                SignInResponse::from(TwoFactorRedirectResource::default()),
                Some(challenge_token),
            ));
        }

        finish_sign_in(db, user, "username", ip, user_agent).await
    }

    /// 匿名用户签发业务逻辑
    pub async fn sign_in_anonymous(
        db: &DatabaseConnection,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> AppResult<AuthResource> {
        let user = user::Entity::create_anonymous(db, None).await?;
        let session = session::Entity::create_session(db, &user.id, ip, user_agent).await?;

        let token = session.token.clone();
        let session_res = SessionResource::from(session);
        let user_res = UserResource::from(user);

        Ok(AuthResource {
            token,
            user: user_res,
            session: Some(session_res),
        })
    }

    /// 获取当前会话上下文 (Better-Auth get-session)
    pub async fn get_session(
        db: &DatabaseConnection,
        token: &str,
    ) -> AppResult<SessionContextResource> {
        let session = session::Entity::find_valid_by_token(db, token).await?;
        let Some(session) = session else {
            return Err(AppError::unauthorized("Session expired or invalid"));
        };

        // 7天滑动窗口自动续期
        let session = session.touch_sliding_window(db).await?;

        let user = user::Entity::find_by_id(db, &session.user_id).await?;
        let Some(user) = user else {
            return Err(AppError::unauthorized("User not found"));
        };

        if user.is_banned() {
            return Err(AppError::forbidden("Account is banned"));
        }

        Ok(SessionContextResource {
            session: SessionResource::from(session),
            user: UserResource::from(user),
        })
    }

    /// 退出登录 (吊销当前会话)
    pub async fn sign_out(db: &DatabaseConnection, token: &str) -> AppResult<()> {
        session::Entity::revoke_by_token(db, token).await?;
        Ok(())
    }

    /// 查询当前用户所有活跃会话列表
    pub async fn list_sessions(
        db: &DatabaseConnection,
        user_id: &str,
    ) -> AppResult<Vec<SessionResource>> {
        let sessions = session::Entity::find_active_by_user(db, user_id).await?;
        Ok(sessions.into_iter().map(SessionResource::from).collect())
    }

    /// 吊销指定会话
    pub async fn revoke_session(
        db: &DatabaseConnection,
        user_id: &str,
        token: &str,
    ) -> AppResult<()> {
        let session = session::Entity::find_valid_by_token(db, token).await?;
        if let Some(session) = session
            && session.user_id == user_id
        {
            session::Entity::revoke_by_token(db, token).await?;
        }
        Ok(())
    }

    /// 按会话 ID 吊销指定会话（校验归属当前用户）
    pub async fn revoke_session_by_id(
        db: &DatabaseConnection,
        session_id: &str,
        user_id: &str,
    ) -> AppResult<()> {
        session::Entity::revoke_by_id_for_user(db, session_id, user_id).await?;
        Ok(())
    }

    /// 吊销当前用户除当前会话外的所有其他活跃会话
    pub async fn revoke_other_sessions(
        db: &DatabaseConnection,
        user_id: &str,
        current_token: &str,
    ) -> AppResult<u64> {
        let count = session::Entity::revoke_other_sessions(db, user_id, current_token).await?;
        Ok(count)
    }

    /// 切换并激活目标会话
    pub async fn set_active_session(
        db: &DatabaseConnection,
        user_id: &str,
        target_token: &str,
    ) -> AppResult<SessionResource> {
        let session = session::Entity::find_valid_by_token(db, target_token).await?;
        let Some(session) = session else {
            return Err(AppError::bad_request("Invalid session token"));
        };

        if session.user_id != user_id {
            return Err(AppError::forbidden("Session does not belong to this user"));
        }

        let session = session.touch_sliding_window(db).await?;
        Ok(SessionResource::from(session))
    }
}

/// 2FA Challenge 临时凭据签发（登录拦截场景复用）
async fn issue_2fa_challenge(db: &DatabaseConnection, user_id: &str) -> AppResult<String> {
    let challenge_token = Uuid::now_v7().to_string();
    let identifier = format!("2fa-challenge:{challenge_token}");
    verification::Entity::create_record(
        db,
        &identifier,
        user_id,
        consts::auth::VERIFICATION_TTL_SECS,
    )
    .await?;
    Ok(challenge_token)
}

/// 登录成功公共尾部：签发会话 + 更新最后登录 + 组装响应载荷
async fn finish_sign_in(
    db: &DatabaseConnection,
    user: user::Model,
    method: &str,
    ip: Option<String>,
    user_agent: Option<String>,
) -> AppResult<(SignInResponse, Option<String>)> {
    let session = session::Entity::create_session(db, &user.id, ip, user_agent).await?;
    let user = user.update_last_login(db, method).await?;

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
