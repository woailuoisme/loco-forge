use loco_rs::prelude::*;
use uuid::Uuid;

use crate::{
    dtos::auth::*,
    errors::{AppError, AppResult},
    models::{session, user, verification},
};

use super::VerificationService;

impl VerificationService {
    /// 发送 Magic Link 登录链接
    pub async fn send_magic_link(
        db: &DatabaseConnection,
        params: &SignInMagicLinkRequest,
    ) -> AppResult<StatusResource> {
        let email = params.email.trim().to_lowercase();
        let token = Uuid::now_v7().to_string();
        let identifier = format!("magic-link:{email}");

        // 有效期 15 分钟 (900 秒)
        verification::Entity::create_record(db, &identifier, &token, 900).await?;

        // 安全约束：token 即登录凭据，绝不写入日志
        tracing::info!(email = %email, "Magic link generated");
        Ok(StatusResource::ok())
    }

    /// 校验 Magic Link Token 并自动登入
    pub async fn verify_magic_link(
        db: &DatabaseConnection,
        token: &str,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> AppResult<AuthResource> {
        let record = verification::Entity::find_valid_by_prefix_and_value(db, "magic-link:", token)
            .await?
            .ok_or_else(|| AppError::bad_request("Invalid or expired magic link token"))?;

        // 从 identifier 提取 email（前缀 + 邮箱），格式异常直接报错而非静默兜底
        let email = record
            .identifier
            .strip_prefix("magic-link:")
            .filter(|e| e.contains('@'))
            .ok_or_else(|| AppError::bad_request("Invalid magic link identifier"))?
            .to_string();

        // 原子消费 token，防止重放
        if !verification::Entity::consume(db, &record.identifier, token).await? {
            return Err(AppError::bad_request("Magic link already used"));
        }

        let user = match user::Entity::find_by_email(db, &email).await? {
            Some(u) => {
                if !u.email_verified {
                    u.set_email_verified(db, true).await?
                } else {
                    u
                }
            },
            None => {
                let new_user =
                    user::Entity::create_user(db, &email, &email, None, None, None).await?;
                new_user.set_email_verified(db, true).await?
            },
        };

        if user.is_banned() {
            return Err(AppError::forbidden("Account is banned"));
        }

        let session = session::Entity::create_session(db, &user.id, ip, user_agent).await?;
        let user = user.update_last_login(db, "magic-link").await?;

        let token = session.token.clone();
        let session_res = SessionResource::from(session);
        let user_res = UserResource::from(user);

        Ok(AuthResource {
            token,
            user: user_res,
            session: Some(session_res),
        })
    }
}
