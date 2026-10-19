use loco_rs::prelude::*;

use crate::{
    dtos::auth::*,
    errors::{AppError, AppResult},
    models::{apikey, user},
};

use super::CredentialService;

impl CredentialService {
    pub async fn create_api_key(
        db: &DatabaseConnection,
        current_user: &user::Model,
        params: &CreateApiKeyRequest,
    ) -> AppResult<ApiKeyCreatedResource> {
        let permissions_str = params.permissions.as_ref().map(ToString::to_string);
        let metadata_str = params.metadata.as_ref().map(ToString::to_string);

        let (model, raw_key) = apikey::Entity::create_key(
            db,
            &current_user.id,
            params.name.clone(),
            params.prefix.clone(),
            params.expires_in,
            params.remaining,
            params.refill_interval,
            params.refill_amount,
            params.rate_limit_enabled,
            params.rate_limit_time_window,
            params.rate_limit_max,
            permissions_str,
            metadata_str,
        )
        .await?;

        Ok(ApiKeyCreatedResource {
            key: raw_key,
            api_key: model.into(),
        })
    }

    pub async fn list_api_keys(
        db: &DatabaseConnection,
        current_user: &user::Model,
    ) -> AppResult<Vec<ApiKeyResource>> {
        let keys = apikey::Entity::find_by_user(db, &current_user.id).await?;
        Ok(keys.into_iter().map(Into::into).collect())
    }

    pub async fn verify_api_key(
        db: &DatabaseConnection,
        params: &VerifyApiKeyRequest,
    ) -> AppResult<VerifyApiKeyResource> {
        let Some(key_model) = apikey::Entity::find_by_key(db, &params.key).await? else {
            return Ok(VerifyApiKeyResource {
                valid: false,
                error: Some("Invalid API key".to_string()),
                key: None,
            });
        };

        if !key_model.is_valid() {
            return Ok(VerifyApiKeyResource {
                valid: false,
                error: Some("API key is disabled, expired, or out of quota".to_string()),
                key: None,
            });
        }

        let updated = key_model.consume_request(db).await?;
        Ok(VerifyApiKeyResource {
            valid: true,
            error: None,
            key: Some(updated.into()),
        })
    }

    pub async fn delete_api_key(
        db: &DatabaseConnection,
        current_user: &user::Model,
        params: &DeleteApiKeyRequest,
    ) -> AppResult<StatusResource> {
        let deleted =
            apikey::Entity::delete_by_id_and_user(db, &params.key_id, &current_user.id).await?;
        if !deleted {
            return Err(AppError::not_found("API key not found or access denied"));
        }
        Ok(StatusResource { status: true })
    }

    pub async fn update_api_key(
        db: &DatabaseConnection,
        current_user: &user::Model,
        params: &UpdateApiKeyRequest,
    ) -> AppResult<ApiKeyResource> {
        let key_model = apikey::Entity::find_by_id(db, &params.key_id)
            .await?
            .ok_or_else(|| AppError::not_found("API key not found"))?;

        if key_model.user_id != current_user.id {
            return Err(AppError::forbidden("Access denied"));
        }

        let metadata_str = params.metadata.as_ref().map(ToString::to_string);
        let updated = key_model
            .update_metadata(
                db,
                params.name.clone(),
                params.enabled,
                None,
                None,
                None,
                None,
                metadata_str,
            )
            .await?;

        Ok(updated.into())
    }
}
