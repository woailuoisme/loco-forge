use loco_rs::prelude::*;

use crate::{
    common::consts,
    dtos::auth::*,
    errors::{AppError, AppResult},
    models::{invitation, member, membership::Membership, user},
};

use super::MembershipService;

impl MembershipService {
    pub async fn invite_member(
        db: &DatabaseConnection,
        current_user: &user::Model,
        params: &InviteMemberRequest,
    ) -> AppResult<InvitationResource> {
        Membership::load(db, &params.organization_id, &current_user.id)
            .await?
            .require_owner_or_admin("Only owners and admins can invite new members")?;

        let email = params.email.trim().to_lowercase();
        if member::Entity::find_by_org_and_user_or_email(db, &params.organization_id, &email)
            .await?
            .is_some()
        {
            return Err(AppError::conflict(
                "User is already a member of this organization",
            ));
        }

        let inv = invitation::Entity::create_invitation(
            db,
            &params.organization_id,
            &email,
            &params.role,
            &current_user.id,
            crate::common::consts::auth::INVITATION_EXPIRES_SECS,
        )
        .await?;
        Ok(inv.into())
    }

    pub async fn accept_invitation(
        db: &DatabaseConnection,
        current_user: &user::Model,
        params: &AcceptInvitationRequest,
    ) -> AppResult<MemberResource> {
        let inv = invitation::Entity::find_by_id(db, &params.invitation_id)
            .await?
            .ok_or_else(|| AppError::not_found("Invitation not found"))?;

        if inv.status != "pending" {
            return Err(AppError::bad_request("Invitation is no longer pending"));
        }
        if inv.is_expired() {
            return Err(AppError::bad_request("Invitation has expired"));
        }
        if inv.email.to_lowercase() != current_user.email.to_lowercase() {
            return Err(AppError::forbidden(
                "This invitation was sent to a different email address",
            ));
        }

        let member = if let Some(existing) =
            member::Entity::find_by_org_and_user(db, &inv.organization_id, &current_user.id).await?
        {
            existing
        } else {
            member::Entity::create_member(db, &inv.organization_id, &current_user.id, &inv.role)
                .await?
        };

        let _ = inv.set_status(db, "accepted").await?;
        Ok(member.into())
    }

    pub async fn reject_invitation(
        db: &DatabaseConnection,
        current_user: &user::Model,
        params: &RejectInvitationRequest,
    ) -> AppResult<StatusResource> {
        let inv = invitation::Entity::find_by_id(db, &params.invitation_id)
            .await?
            .ok_or_else(|| AppError::not_found("Invitation not found"))?;

        if inv.email.to_lowercase() != current_user.email.to_lowercase() {
            return Err(AppError::forbidden(
                "This invitation was sent to a different email address",
            ));
        }

        let _ = inv.set_status(db, "rejected").await?;
        Ok(StatusResource { status: true })
    }

    pub async fn cancel_invitation(
        db: &DatabaseConnection,
        current_user: &user::Model,
        params: &CancelInvitationRequest,
    ) -> AppResult<StatusResource> {
        let inv = invitation::Entity::find_by_id(db, &params.invitation_id)
            .await?
            .ok_or_else(|| AppError::not_found("Invitation not found"))?;

        let current_member = Membership::load(db, &inv.organization_id, &current_user.id)
            .await?
            .member;
        if !current_member.role.eq(consts::auth::ROLE_OWNER)
            && !current_member.role.eq(consts::auth::ROLE_ADMIN)
            && inv.inviter_id != current_user.id
        {
            return Err(AppError::forbidden(
                "You do not have permission to cancel this invitation",
            ));
        }

        let _ = inv.set_status(db, "canceled").await?;
        Ok(StatusResource { status: true })
    }

    pub async fn remove_member(
        db: &DatabaseConnection,
        current_user: &user::Model,
        params: &RemoveMemberRequest,
    ) -> AppResult<StatusResource> {
        let current_member = Membership::load(db, &params.organization_id, &current_user.id)
            .await?
            .require_owner_or_admin("Only owners and admins can remove members")?
            .member;

        let target_member = member::Entity::find_by_org_and_user_or_email(
            db,
            &params.organization_id,
            &params.member_id_or_email,
        )
        .await?
        .ok_or_else(|| AppError::not_found("Member not found"))?;

        if target_member.role == consts::auth::ROLE_OWNER
            && current_member.role != consts::auth::ROLE_OWNER
        {
            return Err(AppError::forbidden(
                "Admins cannot remove organization owners",
            ));
        }

        member::Entity::delete_member(db, &target_member.id).await?;
        Ok(StatusResource { status: true })
    }

    pub async fn update_member_role(
        db: &DatabaseConnection,
        current_user: &user::Model,
        params: &UpdateMemberRoleRequest,
    ) -> AppResult<MemberResource> {
        let current_member = Membership::load(db, &params.organization_id, &current_user.id)
            .await?
            .require_owner_or_admin("Only owners and admins can update member roles")?
            .member;

        let target_member = member::Entity::find_by_org_and_user_or_email(
            db,
            &params.organization_id,
            &params.member_id,
        )
        .await?
        .ok_or_else(|| AppError::not_found("Member not found"))?;

        if target_member.role == consts::auth::ROLE_OWNER
            && current_member.role != consts::auth::ROLE_OWNER
        {
            return Err(AppError::forbidden("Admins cannot update owner roles"));
        }

        let updated = target_member.update_role(db, &params.role).await?;
        Ok(updated.into())
    }

    pub async fn leave_organization(
        db: &DatabaseConnection,
        current_user: &user::Model,
        params: &LeaveOrganizationRequest,
    ) -> AppResult<StatusResource> {
        let target_member =
            member::Entity::find_by_org_and_user(db, &params.organization_id, &current_user.id)
                .await?
                .ok_or_else(|| AppError::not_found("You are not a member of this organization"))?;

        // owner 余额校验与成员删除放在同一事务内，防止并发下删空 owner
        db.transaction_async(async |txn| {
            if target_member.role == consts::auth::ROLE_OWNER {
                let members = member::Entity::find_org_members_with_user(
                    txn,
                    &params.organization_id,
                )
                .await?;
                let owner_count = members
                    .iter()
                    .filter(|(m, _)| m.role == consts::auth::ROLE_OWNER)
                    .count();
                if owner_count <= 1 {
                    return Err(AppError::bad_request(
                        "You are the sole owner of this organization. Transfer ownership or delete the organization first.",
                    ));
                }
            }

            member::Entity::delete_member(txn, &target_member.id).await?;
            Ok::<_, AppError>(())
        })
        .await
        .map_err(|err| match err {
            sea_orm::TransactionError::Connection(db_err) => AppError::from(db_err),
            sea_orm::TransactionError::Transaction(app_err) => app_err,
        })?;
        Ok(StatusResource { status: true })
    }
}
