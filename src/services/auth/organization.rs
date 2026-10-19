use loco_rs::prelude::*;

use crate::{
    common::consts,
    dtos::auth::*,
    errors::{AppError, AppResult},
    models::{invitation, member, membership::Membership, organization, session, user},
};

use super::MembershipService;

impl MembershipService {
    pub async fn create_organization(
        db: &DatabaseConnection,
        current_user: &user::Model,
        current_session: &session::Model,
        params: &CreateOrganizationRequest,
    ) -> AppResult<OrganizationResource> {
        let slug = params.slug.trim().to_lowercase();
        if organization::Entity::find_by_slug(db, &slug)
            .await?
            .is_some()
        {
            return Err(AppError::conflict("Organization slug is already in use"));
        }
        let metadata_str = params.metadata.as_ref().map(ToString::to_string);
        // 建组织 / 写入 owner 成员 / 激活会话必须原子：避免产生无 owner 的孤儿组织
        let org = db
            .transaction_async(async |txn| {
                let org = organization::Entity::create_organization(
                    txn,
                    &params.name,
                    &slug,
                    params.logo.clone(),
                    metadata_str,
                )
                .await?;
                member::Entity::create_member(
                    txn,
                    &org.id,
                    &current_user.id,
                    consts::auth::ROLE_OWNER,
                )
                .await?;
                current_session
                    .set_active_organization(txn, Some(org.id.clone()))
                    .await?;
                Ok::<_, AppError>(org)
            })
            .await
            .map_err(|err| match err {
                sea_orm::TransactionError::Connection(db_err) => AppError::from(db_err),
                sea_orm::TransactionError::Transaction(app_err) => app_err,
            })?;
        Ok(org.into())
    }

    pub async fn update_organization(
        db: &DatabaseConnection,
        current_user: &user::Model,
        params: &UpdateOrganizationRequest,
    ) -> AppResult<OrganizationResource> {
        Membership::load(db, &params.organization_id, &current_user.id)
            .await?
            .require_owner_or_admin("You do not have permission to update this organization")?;
        let org = organization::Entity::find_by_id(db, &params.organization_id)
            .await?
            .ok_or_else(|| AppError::not_found("Organization not found"))?;

        let name = params
            .name
            .clone()
            .or_else(|| params.data.as_ref().and_then(|d| d.name.clone()));
        let slug = params
            .slug
            .clone()
            .or_else(|| params.data.as_ref().and_then(|d| d.slug.clone()));
        let logo = params
            .logo
            .clone()
            .or_else(|| params.data.as_ref().and_then(|d| d.logo.clone()));
        let metadata = params
            .metadata
            .clone()
            .or_else(|| params.data.as_ref().and_then(|d| d.metadata.clone()));
        let metadata_str = metadata.as_ref().map(ToString::to_string);

        if let Some(ref new_slug) = slug {
            let lower_slug = new_slug.trim().to_lowercase();
            if lower_slug != org.slug
                && organization::Entity::find_by_slug(db, &lower_slug)
                    .await?
                    .is_some()
            {
                return Err(AppError::conflict("Organization slug is already in use"));
            }
        }

        let updated = org.update_info(db, name, slug, logo, metadata_str).await?;
        Ok(updated.into())
    }

    pub async fn delete_organization(
        db: &DatabaseConnection,
        current_user: &user::Model,
        organization_id: &str,
    ) -> AppResult<StatusResource> {
        Membership::load(db, organization_id, &current_user.id)
            .await?
            .require_owner("Only organization owners can delete an organization")?;
        organization::Entity::delete_organization(db, organization_id).await?;
        Ok(StatusResource { status: true })
    }

    pub async fn get_full_organization(
        db: &DatabaseConnection,
        current_user: &user::Model,
        query: &GetFullOrganizationQuery,
    ) -> AppResult<FullOrganizationResource> {
        let org = if let Some(ref id) = query.organization_id {
            organization::Entity::find_by_id(db, id).await?
        } else if let Some(ref slug) = query.organization_slug {
            organization::Entity::find_by_slug(db, slug).await?
        } else {
            return Err(AppError::bad_request(
                "organizationId or organizationSlug is required",
            ));
        };
        let org = org.ok_or_else(|| AppError::not_found("Organization not found"))?;

        Membership::load(db, &org.id, &current_user.id).await?;

        let member_tuples = member::Entity::find_org_members_with_user(db, &org.id).await?;
        let members = member_tuples
            .into_iter()
            .map(|(m, u)| {
                let mut mr: MemberResource = m.into();
                mr.user = u.map(Into::into);
                mr
            })
            .collect();

        let invitations = invitation::Entity::find_pending_by_org(db, &org.id)
            .await?
            .into_iter()
            .map(Into::into)
            .collect();

        Ok(FullOrganizationResource {
            organization: org.into(),
            members,
            invitations,
        })
    }

    pub async fn list_organizations(
        db: &DatabaseConnection,
        current_user: &user::Model,
    ) -> AppResult<Vec<OrganizationResource>> {
        let orgs = organization::Entity::find_user_organizations(db, &current_user.id).await?;
        Ok(orgs.into_iter().map(Into::into).collect())
    }

    pub async fn set_active_organization(
        db: &DatabaseConnection,
        current_user: &user::Model,
        current_session: &session::Model,
        params: &SetActiveOrganizationRequest,
    ) -> AppResult<Option<OrganizationResource>> {
        if let Some(ref org_id) = params.organization_id {
            Membership::load(db, org_id, &current_user.id).await?;
            let org = organization::Entity::find_by_id(db, org_id)
                .await?
                .ok_or_else(|| AppError::not_found("Organization not found"))?;
            let _ = current_session
                .set_active_organization(db, Some(org_id.clone()))
                .await?;
            Ok(Some(org.into()))
        } else {
            let _ = current_session.set_active_organization(db, None).await?;
            Ok(None)
        }
    }

    pub async fn get_active_member(
        db: &DatabaseConnection,
        current_user: &user::Model,
        current_session: &session::Model,
    ) -> AppResult<ActiveMemberResource> {
        let org_id = current_session
            .active_organization_id
            .as_deref()
            .ok_or_else(|| AppError::bad_request("No active organization selected"))?;
        let member = Membership::load(db, org_id, &current_user.id).await?.member;
        Ok(member.into())
    }
}
