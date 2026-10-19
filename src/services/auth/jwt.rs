use chrono::Utc;
use loco_rs::prelude::*;

use crate::{
    dtos::auth::*,
    errors::{AppError, AppResult},
    models::{jwks, user},
};

use super::SessionService;

impl SessionService {
    pub async fn get_jwt_token(ctx: &AppContext, user_id: &str) -> AppResult<JwtTokenResource> {
        let Some(user) = user::Entity::find_by_id(&ctx.db, user_id).await? else {
            return Err(AppError::not_found("User not found"));
        };

        let fallback_secret = ctx
            .config
            .auth
            .as_ref()
            .and_then(|a| a.jwt.as_ref())
            .map_or("locos-default-jwt-secret", |a| a.secret.as_str());
        let primary_key = jwks::Entity::get_or_create_primary_key(&ctx.db, fallback_secret).await?;

        #[derive(serde::Serialize)]
        struct Claims<'a> {
            sub: &'a str,
            email: &'a str,
            name: &'a str,
            role: &'a str,
            exp: usize,
            iat: usize,
            iss: &'a str,
            aud: &'a str,
        }

        let now = Utc::now().timestamp() as usize;
        let claims = Claims {
            sub: &user.id,
            email: &user.email,
            name: &user.name,
            role: &user.role,
            exp: now + crate::common::consts::auth::JWT_EXPIRATION_SECS,
            iat: now,
            iss: "locos-auth",
            aud: "locos-client",
        };

        let mut header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::HS256);
        header.kid = Some(primary_key.id);

        let token = jsonwebtoken::encode(
            &header,
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(primary_key.private_key.as_bytes()),
        )
        .map_err(|e| {
            tracing::error!(error = %e, "JWT encoding failed");
            AppError::internal("Internal token signing error")
        })?;

        Ok(JwtTokenResource { token })
    }

    pub async fn get_jwks(ctx: &AppContext) -> AppResult<JwksResource> {
        let fallback_secret = ctx
            .config
            .auth
            .as_ref()
            .and_then(|a| a.jwt.as_ref())
            .map_or("locos-default-jwt-secret", |a| a.secret.as_str());
        jwks::Entity::get_or_create_primary_key(&ctx.db, fallback_secret).await?;

        let keys = jwks::Entity::find_active_keys(&ctx.db).await?;
        let jwk_values: Vec<serde_json::Value> =
            keys.into_iter().map(|k| k.to_jwk_value()).collect();

        Ok(JwksResource { keys: jwk_values })
    }
}
