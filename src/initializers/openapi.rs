use async_trait::async_trait;
use axum::{Json, Router, routing::get};
use loco_rs::{
    Result,
    app::{AppContext, Initializer},
};
use utoipa::{
    Modify, OpenApi,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};
use utoipa_redoc::{Redoc, Servable as RedocServable};
use utoipa_scalar::{Scalar, Servable as ScalarServable};

use crate::{
    common::consts::openapi::{
        TAG_ADMIN, TAG_CREDENTIALS, TAG_FILES, TAG_HOME, TAG_ORG, TAG_SESSION, TAG_SIGNIN,
    },
    controllers, dtos,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        controllers::v1::auth::sign_up_email,
        controllers::v1::auth::sign_in_email,
        controllers::v1::auth::sign_in_username,
        controllers::v1::auth::sign_in_anonymous,
        controllers::v1::auth::get_session,
        controllers::v1::auth::sign_out,
        controllers::v1::auth::list_sessions,
        controllers::v1::auth::revoke_session,
        controllers::v1::auth::revoke_other_sessions,
        controllers::v1::auth::set_active_session,
        controllers::v1::auth::send_verification_otp,
        controllers::v1::auth::check_verification_otp,
        controllers::v1::auth::sign_in_email_otp,
        controllers::v1::auth::verify_email,
        controllers::v1::auth::reset_password,
        controllers::v1::auth::sign_in_magic_link,
        controllers::v1::auth::verify_magic_link,
        controllers::v1::auth::send_phone_otp,
        controllers::v1::auth::verify_phone_number,
        controllers::v1::auth::sign_in_phone_number,
        controllers::v1::auth::two_factor_enable,
        controllers::v1::auth::two_factor_verify_totp,
        controllers::v1::auth::two_factor_verify_backup_code,
        controllers::v1::auth::two_factor_disable,
        controllers::v1::auth::two_factor_generate_backup_codes,
        controllers::v1::auth::passkey_generate_authenticate_options,
        controllers::v1::auth::passkey_verify_authentication,
        controllers::v1::auth::passkey_generate_register_options,
        controllers::v1::auth::passkey_verify_registration,
        controllers::v1::auth::passkey_list_user_passkeys,
        controllers::v1::auth::passkey_delete_passkey,
        controllers::v1::auth::passkey_update_passkey,
        controllers::v1::auth::device_code,
        controllers::v1::auth::device_check,
        controllers::v1::auth::device_approve,
        controllers::v1::auth::device_deny,
        controllers::v1::auth::device_token,
        controllers::v1::auth::sign_in_oauth2,
        controllers::v1::auth::oauth2_callback,
        controllers::v1::auth::one_tap_callback,
        controllers::v1::auth::get_jwt_token,
        controllers::v1::auth::get_jwks,
        controllers::v1::auth::create_organization,
        controllers::v1::auth::update_organization,
        controllers::v1::auth::delete_organization,
        controllers::v1::auth::get_full_organization,
        controllers::v1::auth::list_organizations,
        controllers::v1::auth::set_active_organization,
        controllers::v1::auth::get_active_member,
        controllers::v1::auth::invite_member,
        controllers::v1::auth::accept_invitation,
        controllers::v1::auth::reject_invitation,
        controllers::v1::auth::cancel_invitation,
        controllers::v1::auth::remove_member,
        controllers::v1::auth::update_member_role,
        controllers::v1::auth::leave_organization,
        controllers::v1::auth::admin_set_role,
        controllers::v1::auth::admin_ban_user,
        controllers::v1::auth::admin_unban_user,
        controllers::v1::auth::admin_impersonate_user,
        controllers::v1::auth::admin_stop_impersonating,
        controllers::v1::auth::admin_list_users,
        controllers::v1::auth::admin_remove_user,
        controllers::v1::auth::create_api_key,
        controllers::v1::auth::list_api_keys,
        controllers::v1::auth::verify_api_key,
        controllers::v1::auth::delete_api_key,
        controllers::v1::auth::update_api_key,
        controllers::v1::files::upload,
        controllers::v1::files::get_file,
        controllers::home::index,
        controllers::v1::admin::auth::login,
        controllers::v1::admin::auth::logout,
        controllers::v1::admin::auth::me,
        controllers::v1::admin::user::list,
        controllers::v1::admin::user::get_one,
        controllers::v1::admin::user::create,
        controllers::v1::admin::user::update,
        controllers::v1::admin::user::soft_delete,
        controllers::v1::admin::user::ban,
        controllers::v1::admin::user::unban,
        controllers::v1::admin::user::export,
        controllers::v1::admin::user::import,
    ),
    components(
        schemas(
            dtos::files::FileUploadResource,
            dtos::common::ApiError,
            controllers::home::HomeResource,
            controllers::home::DocsLocation,
            dtos::admin::AdminListUsersQuery,
            dtos::admin::AdminCreateUserRequest,
            dtos::admin::AdminUpdateUserRequest,
            dtos::admin::AdminBanUserRequest,
            dtos::admin::NullableString,
            dtos::admin::AdminMeResource,
            dtos::admin::AdminUserResource,
            dtos::admin::UserImportRow,
            dtos::admin::UserImportSummary,
            dtos::admin::UserImportError,
            dtos::auth::SignUpEmailRequest,
            dtos::auth::SignInEmailRequest,
            dtos::auth::SignInUsernameRequest,
            dtos::auth::SignInAnonymousRequest,
            dtos::auth::RevokeSessionRequest,
            dtos::auth::SetActiveSessionRequest,
            dtos::auth::SendEmailOtpRequest,
            dtos::auth::CheckEmailOtpRequest,
            dtos::auth::SignInEmailOtpRequest,
            dtos::auth::VerifyEmailOtpRequest,
            dtos::auth::ResetPasswordEmailOtpRequest,
            dtos::auth::SignInMagicLinkRequest,
            dtos::auth::SendPhoneOtpRequest,
            dtos::auth::VerifyPhoneNumberRequest,
            dtos::auth::SignInPhoneNumberRequest,
            dtos::auth::TwoFactorPasswordRequest,
            dtos::auth::VerifyTotpRequest,
            dtos::auth::VerifyBackupCodeRequest,
            dtos::auth::UserResource,
            dtos::auth::SessionResource,
            dtos::auth::AuthResource,
            dtos::auth::SessionContextResource,
            dtos::auth::StatusResource,
            dtos::auth::VerifyEmailOtpResource,
            dtos::auth::TwoFactorEnableResource,
            dtos::auth::TwoFactorBackupCodesResource,
            dtos::auth::TwoFactorRedirectResource,
            dtos::auth::SignInResponse,
            dtos::auth::PasskeyGenerateAuthOptionsRequest,
            dtos::auth::PasskeyVerifyAuthRequest,
            dtos::auth::PasskeyGenerateRegOptionsRequest,
            dtos::auth::PasskeyVerifyRegRequest,
            dtos::auth::PasskeyResource,
            dtos::auth::DeletePasskeyRequest,
            dtos::auth::UpdatePasskeyRequest,
            dtos::auth::DeviceCodeRequest,
            dtos::auth::DeviceCodeResource,
            dtos::auth::DeviceCheckRequest,
            dtos::auth::DeviceCheckResource,
            dtos::auth::DeviceApproveRequest,
            dtos::auth::DeviceDenyRequest,
            dtos::auth::DeviceTokenRequest,
            dtos::auth::DeviceTokenResource,
            dtos::auth::SignInOAuth2Request,
            dtos::auth::OAuth2UrlResource,
            dtos::auth::OAuth2CallbackRequest,
            dtos::auth::OneTapCallbackRequest,
            dtos::auth::JwtTokenResource,
            dtos::auth::JwksResource,
            dtos::auth::CreateOrganizationRequest,
            dtos::auth::UpdateOrganizationRequest,
            dtos::auth::UpdateOrganizationDataRequest,
            dtos::auth::SetActiveOrganizationRequest,
            dtos::auth::InviteMemberRequest,
            dtos::auth::AcceptInvitationRequest,
            dtos::auth::RejectInvitationRequest,
            dtos::auth::CancelInvitationRequest,
            dtos::auth::RemoveMemberRequest,
            dtos::auth::UpdateMemberRoleRequest,
            dtos::auth::LeaveOrganizationRequest,
            dtos::auth::OrganizationResource,
            dtos::auth::MemberResource,
            dtos::auth::ActiveMemberResource,
            dtos::auth::InvitationResource,
            dtos::auth::FullOrganizationResource,
            dtos::auth::AdminSetRoleRequest,
            dtos::auth::AdminBanUserRequest,
            dtos::auth::AdminUnbanUserRequest,
            dtos::auth::AdminImpersonateUserRequest,
            dtos::auth::AdminRemoveUserRequest,
            dtos::auth::CreateApiKeyRequest,
            dtos::auth::VerifyApiKeyRequest,
            dtos::auth::DeleteApiKeyRequest,
            dtos::auth::UpdateApiKeyRequest,
            dtos::auth::ApiKeyResource,
            dtos::auth::ApiKeyCreatedResource,
            dtos::auth::VerifyApiKeyResource,
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = TAG_SESSION, description = "注册、登录与多会话管理"),
        (name = TAG_SIGNIN, description = "验证码、魔法链接与第三方登录"),
        (name = TAG_CREDENTIALS, description = "两步验证、通行密钥与 API 密钥"),
        (name = TAG_ORG, description = "多租户组织与成员邀请"),
        (name = TAG_ADMIN, description = "管理员用户治理操作"),
        (name = TAG_FILES, description = "文件上传与下载"),
        (name = TAG_HOME, description = "API 导航与版本信息"),
    ),
    info(
        title = "Loco RESTful API 文档",
        version = "1.0.0",
        description = "基于 utoipa 与 Scalar 驱动的 RESTful API 接口规范说明"
    )
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}

pub struct OpenApiInitializer;

#[async_trait]
impl Initializer for OpenApiInitializer {
    fn name(&self) -> String {
        "openapi".to_string()
    }

    async fn after_routes(&self, router: Router, _ctx: &AppContext) -> Result<Router> {
        let openapi = ApiDoc::openapi();

        let router = router
            .route(
                "/api-docs/openapi.json",
                get({
                    let openapi_json = openapi.clone();
                    move || async move { Json(openapi_json) }
                }),
            )
            .merge(Scalar::with_url("/docs", openapi.clone()).custom_html(
                r#"<!doctype html>
<html>
<head>
    <title>Loco RESTful API 文档</title>
    <meta charset="utf-8"/>
    <meta name="viewport" content="width=device-width, initial-scale=1"/>
</head>
<body>

<script
        id="api-reference"
        data-configuration='{"hideModels": true}'
        type="application/json">
    $spec
</script>
<script src="https://cdn.jsdelivr.net/npm/@scalar/api-reference"></script>
</body>
</html>"#,
            ))
            .merge(Redoc::with_url("/redoc", openapi));

        Ok(router)
    }
}
