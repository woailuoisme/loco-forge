use axum::http::StatusCode;
use loco_rs::testing::prelude::*;
use locos::{app::App, dtos::auth::*};
use serial_test::serial;
use uuid::Uuid;

#[tokio::test]
#[serial]
async fn test_organization_full_lifecycle_and_members() {
    request::<App, _, _>(|request, _ctx| async move {
        // 1. 注册 Owner 用户与普通成员用户
        let owner_email = format!("owner_{}@example.com", Uuid::new_v4().simple());
        let owner_reg = request
            .post("/v1/users")
            .add_header("x-forwarded-for", "10.30.2.1")
            .json(&serde_json::json!({
                "email": owner_email,
                "password": "strong_password_123",
                "name": "Org Owner"
            }))
            .await;
        assert_eq!(owner_reg.status_code(), StatusCode::CREATED);
        let owner_auth: AuthSessionResource = serde_json::from_str(&owner_reg.text()).unwrap();

        let member_email = format!("member_{}@example.com", Uuid::new_v4().simple());
        let member_reg = request
            .post("/v1/users")
            .add_header("x-forwarded-for", "10.30.2.2")
            .json(&serde_json::json!({
                "email": member_email,
                "password": "strong_password_123",
                "name": "Org Member"
            }))
            .await;
        assert_eq!(member_reg.status_code(), StatusCode::CREATED);
        let member_auth: AuthSessionResource = serde_json::from_str(&member_reg.text()).unwrap();

        // 2. 创建新组织
        let slug = format!("acme-{}", &Uuid::new_v4().simple().to_string()[..6]);
        let create_org_resp = request
            .post("/v1/organizations")
            .add_header("x-forwarded-for", "10.30.2.3")
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", owner_auth.token),
            )
            .json(&serde_json::json!({
                "name": "Acme Inc",
                "slug": slug
            }))
            .await;
        assert_eq!(create_org_resp.status_code(), StatusCode::OK);
        let org: OrganizationResource = serde_json::from_str(&create_org_resp.text()).unwrap();
        assert_eq!(org.name, "Acme Inc");

        // 3. 获取当前活跃成员 (创建者自动为 owner)
        let active_mbr_resp = request
            .get("/v1/organizations/current/members/me")
            .add_header("x-forwarded-for", "10.30.2.4")
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", owner_auth.token),
            )
            .await;
        assert_eq!(active_mbr_resp.status_code(), StatusCode::OK);
        let active_mbr: ActiveMemberResource =
            serde_json::from_str(&active_mbr_resp.text()).unwrap();
        assert_eq!(active_mbr.role, "owner");
        assert_eq!(active_mbr.organization_id, org.id);

        // 4. 更新组织资料
        let update_resp = request
            .patch("/v1/organizations/current")
            .add_header("x-forwarded-for", "10.30.2.5")
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", owner_auth.token),
            )
            .json(&serde_json::json!({
                "organizationId": org.id,
                "name": "Acme Global Enterprise"
            }))
            .await;
        assert_eq!(update_resp.status_code(), StatusCode::OK);
        let updated_org: OrganizationResource = serde_json::from_str(&update_resp.text()).unwrap();
        assert_eq!(updated_org.name, "Acme Global Enterprise");

        // 5. 邀请 User B 加入组织
        let invite_resp = request
            .post("/v1/organizations/invitations")
            .add_header("x-forwarded-for", "10.30.2.6")
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", owner_auth.token),
            )
            .json(&serde_json::json!({
                "organizationId": org.id,
                "email": member_email,
                "role": "member"
            }))
            .await;
        assert_eq!(invite_resp.status_code(), StatusCode::OK);
        let inv: InvitationResource = serde_json::from_str(&invite_resp.text()).unwrap();
        assert_eq!(inv.email, member_email);
        assert_eq!(inv.status, "pending");

        // 6. 获取组织详情与完整结构 (含挂起的邀请)
        let full_org_resp = request
            .get(&format!(
                "/v1/organizations/current?organizationId={}",
                org.id
            ))
            .add_header("x-forwarded-for", "10.30.2.7")
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", owner_auth.token),
            )
            .await;
        assert_eq!(full_org_resp.status_code(), StatusCode::OK);
        let full_org: FullOrganizationResource =
            serde_json::from_str(&full_org_resp.text()).unwrap();
        assert_eq!(full_org.members.len(), 1);
        assert_eq!(full_org.invitations.len(), 1);

        // 7. User B 接受邀请加入组织
        let accept_resp = request
            .post("/v1/invitation-acceptances")
            .add_header("x-forwarded-for", "10.30.2.8")
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", member_auth.token),
            )
            .json(&serde_json::json!({
                "invitationId": inv.id
            }))
            .await;
        assert_eq!(accept_resp.status_code(), StatusCode::OK);
        let new_member: MemberResource = serde_json::from_str(&accept_resp.text()).unwrap();
        assert_eq!(new_member.role, "member");
        assert_eq!(new_member.user_id, member_auth.user.id);

        // 8. User B 设置当前活跃组织
        let set_act_resp = request
            .patch("/v1/organizations/current/active")
            .add_header("x-forwarded-for", "10.30.2.9")
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", member_auth.token),
            )
            .json(&serde_json::json!({
                "organizationId": org.id
            }))
            .await;
        assert_eq!(set_act_resp.status_code(), StatusCode::OK);

        // 9. User A 修改 User B 的角色为 admin
        let update_role_resp = request
            .patch("/v1/members")
            .add_header("x-forwarded-for", "10.30.2.10")
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", owner_auth.token),
            )
            .json(&serde_json::json!({
                "organizationId": org.id,
                "memberId": new_member.id,
                "role": "admin"
            }))
            .await;
        assert_eq!(update_role_resp.status_code(), StatusCode::OK);
        let role_res: MemberResource = serde_json::from_str(&update_role_resp.text()).unwrap();
        assert_eq!(role_res.role, "admin");

        // 10. 列出用户所属组织列表
        let list_org_resp = request
            .get("/v1/organizations")
            .add_header("x-forwarded-for", "10.30.2.11")
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", member_auth.token),
            )
            .await;
        assert_eq!(list_org_resp.status_code(), StatusCode::OK);
        let orgs: Vec<OrganizationResource> = serde_json::from_str(&list_org_resp.text()).unwrap();
        assert_eq!(orgs.len(), 1);
        assert_eq!(orgs[0].id, org.id);

        // 11. User B 主动退出组织
        let leave_resp = request
            .delete(&format!("/v1/members/me?organizationId={}", org.id))
            .add_header("x-forwarded-for", "10.30.2.12")
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", member_auth.token),
            )
            .await;
        assert_eq!(leave_resp.status_code(), StatusCode::OK);

        // 12. User A 删除组织
        let del_org_resp = request
            .delete(&format!("/v1/organizations/{}", org.id))
            .add_header("x-forwarded-for", "10.30.2.13")
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", owner_auth.token),
            )
            .await;
        assert_eq!(del_org_resp.status_code(), StatusCode::OK);
    })
    .await;
}
