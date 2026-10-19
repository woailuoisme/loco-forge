//! 认证业务逻辑，按概念边界拆分为 5 个 service（原来是一个 `AuthService` 吞掉全部）。
//!
//! | Service | 覆盖子领域 | 源文件 |
//! |---|---|---|
//! | [`SessionService`] | 注册 / 登录 / 会话 / JWT / 设备码 | `session` `jwt` `device_code` |
//! | [`VerificationService`] | 邮箱与短信验证码 / 2FA / 魔法链接 | `otp` `two_factor` `magic_link` |
//! | [`MembershipService`] | 组织 / 成员 / 邀请 | `organization` `member` |
//! | [`CredentialService`] | API Key / Passkey / OAuth2 | `api_key` `passkey` `oauth2` |
//! | [`AuthAdminService`] | 后台治理（角色、封禁、冒充、删除用户） | `admin` |
//!
//! 各文件通过 `impl <Type>` 扩展对应类型，方法名在全部文件间唯一（无重名冲突）。

/// 会话与令牌：注册、登录、会话管理、JWT 签发、设备授权流
pub struct SessionService;

/// 验证与二次验证：验证码、邮箱/手机验证、2FA、魔法链接
pub struct VerificationService;

/// 组织与成员：组织 CRUD、成员与邀请
pub struct MembershipService;

/// 凭据：API Key、Passkey、OAuth2 / One-Tap
pub struct CredentialService;

/// 后台治理：角色设置、封禁、冒充、移除用户
pub struct AuthAdminService;

pub mod admin;
pub mod api_key;
pub mod device_code;
pub mod jwt;
pub mod magic_link;
pub mod member;
pub mod oauth2;
pub mod organization;
pub mod otp;
pub mod passkey;
pub mod session;
pub mod two_factor;
