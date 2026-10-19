use chrono::Utc;
use loco_rs::prelude::*;

use crate::{
    common::consts,
    dtos::auth::*,
    dtos::common::{PageMeta, PaginatedResource},
    errors::{AppError, AppResult},
    models::{account, session, user},
};

use super::AuthAdminService;

impl AuthAdminService {
    pub async fn admin_set_role(
        db: &DatabaseConnection,
        current_user: &user::Model,
        params: &AdminSetRoleRequest,
    ) -> AppResult<UserResource> {
        current_user.require_admin()?;
        let target_user = user::Entity::find_by_id(db, &params.user_id)
            .await?
            .ok_or_else(|| AppError::not_found("User not found"))?;
        let updated = target_user.set_role(db, &params.role).await?;
        Ok(updated.into())
    }

    pub async fn admin_ban_user(
        db: &DatabaseConnection,
        current_user: &user::Model,
        params: &AdminBanUserRequest,
    ) -> AppResult<UserResource> {
        current_user.require_admin()?;
        if params.user_id == current_user.id {
            return Err(AppError::bad_request("Cannot ban yourself"));
        }
        let target_user = user::Entity::find_by_id(db, &params.user_id)
            .await?
            .ok_or_else(|| AppError::not_found("User not found"))?;

        let expires = params
            .ban_expires_in
            .map(|secs| (Utc::now() + chrono::Duration::seconds(secs)).into());
        let updated = target_user
            .set_banned(db, true, params.ban_reason.as_deref(), expires)
            .await?;

        // 撤销该用户所有活跃会话
        session::Entity::revoke_all_sessions(db, &target_user.id).await?;

        Ok(updated.into())
    }

    pub async fn admin_unban_user(
        db: &DatabaseConnection,
        current_user: &user::Model,
        params: &AdminUnbanUserRequest,
    ) -> AppResult<UserResource> {
        current_user.require_admin()?;
        let target_user = user::Entity::find_by_id(db, &params.user_id)
            .await?
            .ok_or_else(|| AppError::not_found("User not found"))?;
        let updated = target_user.set_banned(db, false, None, None).await?;
        Ok(updated.into())
    }

    pub async fn admin_impersonate_user(
        db: &DatabaseConnection,
        current_user: &user::Model,
        ip: Option<String>,
        user_agent: Option<String>,
        params: &AdminImpersonateUserRequest,
    ) -> AppResult<AuthResource> {
        current_user.require_admin()?;
        let target_user = user::Entity::find_by_id(db, &params.user_id)
            .await?
            .ok_or_else(|| AppError::not_found("Target user not found"))?;
        if target_user.role == consts::auth::ROLE_ADMIN {
            return Err(AppError::bad_request(
                "Cannot impersonate another admin user",
            ));
        }

        let new_session = session::Entity::create_impersonated_session(
            db,
            &target_user.id,
            &current_user.id,
            ip,
            user_agent,
        )
        .await?;

        Ok(AuthResource {
            token: new_session.token.clone(),
            user: target_user.into(),
            session: Some(new_session.into()),
        })
    }

    pub async fn admin_stop_impersonating(
        db: &DatabaseConnection,
        current_session: &session::Model,
    ) -> AppResult<AuthResource> {
        let admin_id = current_session
            .impersonated_by
            .as_deref()
            .ok_or_else(|| AppError::bad_request("Current session is not impersonating anyone"))?;
        let admin_user = user::Entity::find_by_id(db, admin_id)
            .await?
            .ok_or_else(|| AppError::not_found("Admin user not found"))?;

        session::Entity::revoke_by_token(db, &current_session.token).await?;
        let restored_session = session::Entity::create_session(
            db,
            &admin_user.id,
            current_session.ip_address.clone(),
            current_session.user_agent.clone(),
        )
        .await?;

        Ok(AuthResource {
            token: restored_session.token.clone(),
            user: admin_user.into(),
            session: Some(restored_session.into()),
        })
    }

    pub async fn admin_list_users(
        db: &DatabaseConnection,
        current_user: &user::Model,
        query: &AdminListUsersQuery,
    ) -> AppResult<PaginatedResource<UserResource>> {
        current_user.require_admin()?;
        // limit 上限收敛，防止超大分页拖垮查询
        let limit = query
            .limit
            .unwrap_or(50)
            .clamp(1, consts::pagination::MAX_PAGE_SIZE);
        let offset = query.offset.unwrap_or(0);
        let search_val = query.search_value.as_deref();

        let (users, total) =
            user::Entity::find_all_paginated(db, limit, offset, search_val).await?;
        Ok(PaginatedResource {
            items: users.into_iter().map(Into::into).collect(),
            // limit/offset 入参换算为页码信封：page = offset / limit + 1
            meta: PageMeta::new(offset / limit + 1, limit, total),
        })
    }

    pub async fn admin_remove_user(
        db: &DatabaseConnection,
        current_user: &user::Model,
        params: &AdminRemoveUserRequest,
    ) -> AppResult<StatusResource> {
        current_user.require_admin()?;
        if params.user_id == current_user.id {
            return Err(AppError::bad_request("Cannot remove yourself"));
        }
        let target_user = user::Entity::find_by_id(db, &params.user_id)
            .await?
            .ok_or_else(|| AppError::not_found("User not found"))?;

        // 撤会话 / 删凭证 / 软删账号必须原子：任一步失败都回滚，避免账号与凭证不一致
        db.transaction_async(async |txn| {
            session::Entity::revoke_all_sessions(txn, &target_user.id).await?;
            account::Entity::delete_by_user(txn, &target_user.id).await?;
            // 管理端移除用户走软删，保留审计轨迹（api.md §2.3 语义）
            target_user.soft_delete(txn).await?;
            Ok::<_, AppError>(())
        })
        .await
        .map_err(|err| match err {
            sea_orm::TransactionError::Connection(db_err) => {
                tracing::error!(error = %db_err, "移除用户事务连接失败");
                AppError::from(db_err)
            },
            sea_orm::TransactionError::Transaction(app_err) => {
                tracing::error!(error = %app_err, "移除用户事务执行失败");
                app_err
            },
        })?;

        Ok(StatusResource { status: true })
    }
}
