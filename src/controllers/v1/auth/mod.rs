//! # Better-Auth 兼容认证控制器 (Authentication Controller)
//!
//! 提供 Better-Auth 核心认证标准端点规范实现：
//! - 邮箱注册与登录 (`/api/auth/sign-up/email`, `/api/auth/sign-in/email`)
//! - 用户名登录 (`/api/auth/sign-in/username`)
//! - 匿名会话登录 (`/api/auth/sign-in/anonymous`)
//! - 会话上下文获取 (`/api/auth/get-session`)
//! - 退出登录 (`/api/auth/sign-out`)
//! - 会话列表与吊销管理 (`/api/auth/list-sessions`, `/api/auth/revoke-session`, `/api/auth/revoke-other-sessions`)
//! - 多会话切换 (`/api/auth/set-active-session`)

use loco_rs::prelude::*;

use crate::common::consts::api::API_PREFIX;

mod shared;
pub use shared::*;

mod session;
pub use session::*;
mod otp;
pub use otp::*;
mod magic_link;
pub use magic_link::*;
mod two_factor;
pub use two_factor::*;
mod passkey;
pub use passkey::*;
mod device_code;
pub use device_code::*;
mod oauth2;
pub use oauth2::*;
mod jwt;
pub use jwt::*;
mod organization;
pub use organization::*;
mod member;
pub use member::*;
mod admin;
pub use admin::*;
mod api_key;
pub use api_key::*;

// ─────────────────────────────────────────────────────────────────────────────
// 路由注册 (Routes)
// ─────────────────────────────────────────────────────────────────────────────

pub fn routes() -> Routes {
    Routes::new()
        .prefix(API_PREFIX)
        // ── 账户与会话 (RESTful 资源化) ────────────────────────────────────
        .add("/users", post(sign_up_email))
        .add("/sessions", get(list_sessions).delete(revoke_other_sessions))
        .add(
            "/sessions/current",
            get(get_session)
                .patch(set_active_session)
                .delete(sign_out),
        )
        .add("/sessions/{id}", delete(revoke_session))
        .add("/sessions/email", post(sign_in_email))
        .add("/sessions/username", post(sign_in_username))
        .add("/sessions/anonymous", post(sign_in_anonymous))
        .add("/access-tokens", post(get_jwt_token))
        .add("/jwks", get(get_jwks))
        // ── 以下子域待资源化（暂留旧路径，按批次迁移）──────────────────────
        // Phase 2: Email OTP
        .add("/verification-codes", post(send_verification_otp))
        .add("/verification-code-checks", post(check_verification_otp))
        .add("/sessions/email-otp", post(sign_in_email_otp))
        .add("/email-verifications", post(verify_email))
        .add("/users/me/password", patch(reset_password))
        // Phase 2: Magic Link
        .add("/sessions/magic-link", post(sign_in_magic_link))
        .add("/magic-link-tokens/{token}", get(verify_magic_link))
        // Phase 2: Phone Number
        .add("/phone-verification-codes", post(send_phone_otp))
        .add("/phone-verifications", post(verify_phone_number))
        .add("/sessions/phone-number", post(sign_in_phone_number))
        // 二次验证 (2FA)
        .add(
            "/users/me/two-factor",
            post(two_factor_enable).delete(two_factor_disable),
        )
        .add("/totp-checks", post(two_factor_verify_totp))
        .add("/backup-code-checks", post(two_factor_verify_backup_code))
        .add("/backup-codes", post(two_factor_generate_backup_codes))
        // Passkey
        .add(
            "/passkeys",
            post(passkey_verify_registration)
                .get(passkey_list_user_passkeys)
                .delete(passkey_delete_passkey)
                .patch(passkey_update_passkey),
        )
        .add(
            "/passkey-registration-challenges",
            post(passkey_generate_register_options),
        )
        .add(
            "/passkey-authentication-challenges",
            post(passkey_generate_authenticate_options),
        )
        .add(
            "/passkey-authentications",
            post(passkey_verify_authentication),
        )
        // Device Authorization (RFC 8628)
        .add("/device-authorizations", post(device_code))
        .add("/device-authorizations/{user_code}", get(device_check))
        .add("/device-authorization-approvals", post(device_approve))
        .add("/device-authorization-denials", post(device_deny))
        .add("/device-authorization-tokens", post(device_token))
        // OAuth2 与 One Tap
        .add("/sessions/oauth2", post(sign_in_oauth2))
        .add("/oauth2-callbacks", get(oauth2_callback))
        .add("/one-tap-credentials", post(one_tap_callback))
        // Organization / Member / Invitation
        .add("/organizations", post(create_organization).get(list_organizations))
        .add("/organizations/current", get(get_full_organization).patch(update_organization))
        .add("/organizations/{organization_id}", delete(delete_organization))
        .add("/organizations/current/active", patch(set_active_organization))
        .add("/organizations/current/members/me", get(get_active_member))
        .add("/organizations/invitations", post(invite_member))
        .add("/invitation-acceptances", post(accept_invitation))
        .add("/invitation-rejections", post(reject_invitation))
        .add("/invitations", delete(cancel_invitation))
        .add("/members", delete(remove_member).patch(update_member_role))
        .add("/members/me", delete(leave_organization))
        // Admin 治理
        .add("/admin/accounts", get(admin_list_users).delete(admin_remove_user))
        .add("/admin/user-roles", patch(admin_set_role))
        .add("/admin/user-bans", patch(admin_ban_user).delete(admin_unban_user))
        .add(
            "/admin/impersonation-sessions",
            post(admin_impersonate_user).delete(admin_stop_impersonating),
        )
        // API Key
        .add(
            "/api-keys",
            post(create_api_key)
                .get(list_api_keys)
                .delete(delete_api_key)
                .patch(update_api_key),
        )
        .add("/api-key-checks", post(verify_api_key))
        .layer(crate::setup::middleware::rate_limit::auth_rate_limit_layer())
}
