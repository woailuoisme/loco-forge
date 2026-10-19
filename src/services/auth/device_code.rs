use loco_rs::prelude::*;

use crate::{
    common::consts,
    dtos::auth::*,
    errors::{AppError, AppResult},
    models::{device_code, session, user},
};

use super::SessionService;

impl SessionService {
    pub async fn create_device_code(
        db: &DatabaseConnection,
        params: &DeviceCodeRequest,
    ) -> AppResult<DeviceCodeResource> {
        let expires_in = consts::auth::DEVICE_CODE_TTL_SECS;
        let interval = 5;
        let verification_uri = "/auth/device".to_string();

        let record = device_code::Entity::create_code(
            db,
            params.client_id.clone(),
            params.scope.clone(),
            expires_in,
            interval,
        )
        .await?;

        let verification_uri_complete = format!("/auth/device?user_code={}", record.user_code);

        Ok(DeviceCodeResource {
            device_code: record.device_code,
            user_code: record.user_code,
            verification_uri,
            verification_uri_complete: Some(verification_uri_complete),
            expires_in,
            interval: i64::from(interval),
        })
    }

    pub async fn check_device_code(
        db: &DatabaseConnection,
        user_code: &str,
    ) -> AppResult<DeviceCheckResource> {
        let Some(code) = device_code::Entity::find_by_user_code(db, user_code).await? else {
            return Err(AppError::not_found("Device code not found"));
        };

        let status = if code.is_expired() {
            "expired".to_string()
        } else {
            code.status
        };

        Ok(DeviceCheckResource {
            user_code: code.user_code,
            status,
            client_id: code.client_id,
            scope: code.scope,
        })
    }

    pub async fn approve_device_code(
        db: &DatabaseConnection,
        user_id: &str,
        user_code: &str,
    ) -> AppResult<StatusResource> {
        let Some(code) = device_code::Entity::find_by_user_code(db, user_code).await? else {
            return Err(AppError::not_found("Device code not found"));
        };

        if code.is_expired() {
            return Err(AppError::bad_request("Device code has expired"));
        }

        device_code::Entity::approve_for_user(db, user_code, user_id).await?;
        Ok(StatusResource::ok())
    }

    pub async fn deny_device_code(
        db: &DatabaseConnection,
        _user_id: &str,
        user_code: &str,
    ) -> AppResult<StatusResource> {
        let Some(code) = device_code::Entity::find_by_user_code(db, user_code).await? else {
            return Err(AppError::not_found("Device code not found"));
        };

        if code.is_expired() {
            return Err(AppError::bad_request("Device code has expired"));
        }

        device_code::Entity::deny_for_user(db, user_code).await?;
        Ok(StatusResource::ok())
    }

    pub async fn poll_device_token(
        db: &DatabaseConnection,
        params: &DeviceTokenRequest,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> AppResult<DeviceTokenResource> {
        if params.grant_type != "urn:ietf:params:oauth:grant-type:device_code" {
            return Err(AppError::bad_request("unsupported_grant_type"));
        }

        let Some(code) = device_code::Entity::find_by_device_code(db, &params.device_code).await?
        else {
            return Err(AppError::bad_request("invalid_grant"));
        };

        if code.is_expired() {
            return Err(AppError::bad_request("expired_token"));
        }

        match code.status.as_str() {
            "pending" => Err(AppError::bad_request("authorization_pending")),
            "denied" => Err(AppError::bad_request("access_denied")),
            "approved" => {
                let user_id = code
                    .user_id
                    .clone()
                    .ok_or_else(|| AppError::internal("Missing user in approved code"))?;
                let user = user::Entity::find_by_id(db, &user_id)
                    .await?
                    .ok_or_else(|| AppError::not_found("User not found"))?;

                device_code::Entity::consume_code(db, &code.id).await?;

                let session = session::Entity::create_session(db, &user.id, ip, user_agent).await?;

                Ok(DeviceTokenResource {
                    access_token: session.token.clone(),
                    token_type: "bearer".to_string(),
                    expires_in: 604800,
                    user: user.into(),
                    session: session.into(),
                })
            },
            _ => Err(AppError::bad_request("invalid_grant")),
        }
    }

    // 3. Generic OAuth & One Tap
}
