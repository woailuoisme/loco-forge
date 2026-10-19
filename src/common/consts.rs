//! # 全局常量统一定义模块 (Global Constants)
//!
//! 提供系统应用、日志与存储、分页限制等全局常量的统一声明与管理。

/// 应用与环境标识常量
pub mod app {
    /// 应用默认名称
    pub const APP_NAME: &str = "locos";
    /// 开发环境标识
    pub const ENV_DEVELOPMENT: &str = "development";
    /// 生产环境标识
    pub const ENV_PRODUCTION: &str = "production";
    /// 测试环境标识
    pub const ENV_TEST: &str = "test";
}

/// HTTP API 版本化路径常量
pub mod api {
    /// 当前 API 主版本前缀 (`"/v1"`)
    ///
    /// 所有业务资源路由统一挂载于该前缀之下 (如 `/v1/auth/*`、`/v1/files/*`)。
    /// 基础设施入口 (`/`、`/livez`、`/readyz`、`/docs` 等) 不纳入版本管理。
    pub const API_PREFIX: &str = "/v1";
}

/// 存储与文件路径常量
pub mod storage {
    /// 默认存储根目录 (`"./storages"`)
    pub const DEFAULT_STORAGE_DIR: &str = "./storages";
    /// 默认单文件字节限制上限 (10 MB)
    pub const DEFAULT_MAX_FILE_SIZE: usize = 10 * 1024 * 1024;
    /// 默认单次 Multipart 总上传字节限制上限 (50 MB)
    pub const DEFAULT_MAX_TOTAL_UPLOAD_SIZE: usize = 50 * 1024 * 1024;
}

/// HTTP 接口通用分页限制常量
pub mod pagination {
    /// 默认页码 (从 1 开始)
    pub const DEFAULT_PAGE: u64 = 1;
    /// 默认单页记录条数
    pub const DEFAULT_PAGE_SIZE: u64 = 20;
    /// 单页允许查询的最大记录条数
    pub const MAX_PAGE_SIZE: u64 = 100;
}

/// OpenAPI 与 Scalar 交互式文档 Tag 标签常量
///
/// 分组边界对齐 Better-Auth 插件体系：主认证流程 / 附加登录方式 /
/// 二次验证与凭证 / 多租户组织 / 后台治理 / 文件存储 各自独立成组。
pub mod openapi {
    /// 认证会话 Tag ("认证会话")：注册、登录、会话查询与吊销
    pub const TAG_SESSION: &str = "认证会话";
    /// 登录方式 Tag ("登录方式")：验证码、魔法链接、第三方与设备授权
    pub const TAG_SIGNIN: &str = "登录方式";
    /// 多因素与密钥 Tag ("多因素与密钥")：两步验证、通行密钥与 API 密钥
    pub const TAG_CREDENTIALS: &str = "多因素与密钥";
    /// 组织与成员 Tag ("组织与成员")：多租户组织与成员邀请
    pub const TAG_ORG: &str = "组织与成员";
    /// 系统管理 Tag ("系统管理")：管理员用户治理操作
    pub const TAG_ADMIN: &str = "系统管理";
    /// 文件存储与下载服务接口 Tag ("文件存储")
    pub const TAG_FILES: &str = "文件存储";
    /// 根路由信息与 API 导航接口 Tag ("服务信息")
    pub const TAG_HOME: &str = "服务信息";
}

/// 用户认证与账号业务常量
pub mod auth {
    /// 允许接收验证码/魔术链接的邮箱域名正则 (匹配 @example.com 或 @gmail.com 结尾)
    pub const ALLOW_EMAIL_DOMAIN_PATTERN: &str = r"@example\.com$|@gmail\.com$";
    /// 管理员角色标识
    pub const ROLE_ADMIN: &str = "admin";
    /// 组织所有者角色标识
    pub const ROLE_OWNER: &str = "owner";
    /// JWT 访问令牌有效期 (24 小时)
    pub const JWT_EXPIRATION_SECS: usize = 60 * 60 * 24;
    /// 组织邀请有效期 (7 天)
    pub const INVITATION_EXPIRES_SECS: i64 = 7 * 24 * 3600;
    /// OTP 6 位数字验证码有效分钟数 (10 分钟)
    pub const OTP_EXPIRATION_MIN: i64 = 10;
    /// 验证码记录有效期秒数 (10 分钟)
    pub const OTP_TTL_SECS: i64 = 600;
    /// 邮箱验证 / 密码重置 Token 有效期秒数 (5 分钟)
    pub const VERIFICATION_TTL_SECS: i64 = 300;
    /// 设备授权码有效期秒数 (10 分钟)
    pub const DEVICE_CODE_TTL_SECS: i64 = 600;
    /// OTP 验证码字符长度 (6 位)
    pub const OTP_CODE_LENGTH: usize = 6;
    /// 密码重置 Token 有效分钟数 (30 分钟)
    pub const RESET_TOKEN_EXPIRATION_MIN: i64 = 30;
    /// Passkey 默认 Relying Party ID (可被 config settings.auth.passkey.rp_id 覆盖)
    pub const PASSKEY_RP_ID: &str = "localhost";
    /// Passkey 默认 Origin (可被 config settings.auth.passkey.origin 覆盖)
    pub const PASSKEY_ORIGIN: &str = "http://localhost:5150";
    /// Passkey Relying Party 展示名称
    pub const PASSKEY_RP_NAME: &str = "Locos App";
    /// OTP 校验最大连续失败次数，超过即锁定
    pub const OTP_MAX_ATTEMPTS: i32 = 5;
    /// OTP 锁定时长秒数 (15 分钟)
    pub const OTP_LOCKOUT_SECS: i64 = 900;
}

/// Excel 导入导出处理常量
pub mod excel {
    /// 默认分批处理行数（导入与导出通用，内存友好）
    pub const DEFAULT_CHUNK_SIZE: usize = 1000;
    /// 单个 Sheet 允许写入的最大数据行数（超过自动创建新 Sheet）
    pub const MAX_ROWS_PER_SHEET: usize = 1_000_000;
    /// 导入过程允许收集的最大错误条数（达到阈值即熔断提前停止解析）
    pub const MAX_IMPORT_ERRORS: usize = 100;
}

/// Telegram Bot 通知渠道常量
pub mod telegram {
    /// Telegram HTTP API 请求超时秒数
    pub const REQUEST_TIMEOUT_SECS: u64 = 10;
}

/// 图片处理与水印常量
pub mod media {
    /// 图片最大边长安全限制（像素），超出拒绝加载防止内存耗尽
    pub const MAX_DIMENSION: u32 = 8000;
    /// 水印相对底图的最大尺寸占比
    pub const WATERMARK_MAX_RATIO: f32 = 0.35;
    /// 水印距边缘的默认内边距（像素）
    pub const WATERMARK_PADDING: u32 = 10;
    /// WebP 无损编码质量阈值（质量高于该值启用 lossless 编码）
    pub const WEBP_LOSSLESS_QUALITY: u8 = 90;
}

/// 二维码生成常量
pub mod qrcode {
    /// 默认边长（像素）
    pub const DEFAULT_SIZE: u32 = 256;
    /// 允许设置的最小边长（像素）
    pub const MIN_SIZE: u32 = 32;
    /// 默认白边/留白边距格数
    pub const DEFAULT_MARGIN: u32 = 4;
    /// Logo 默认缩放比例（相对二维码边长）
    pub const LOGO_DEFAULT_SCALE: f32 = 0.20;
    /// Logo 缩放比例下限
    pub const LOGO_MIN_SCALE: f32 = 0.10;
    /// Logo 缩放比例上限
    pub const LOGO_MAX_SCALE: f32 = 0.35;
    /// Logo 外围白色对比遮罩默认边距（像素）
    pub const DEFAULT_BORDER_PADDING: u32 = 4;
}

/// 标准业务逻辑错误码常量 (范围 600 ~ 999)
pub mod biz_code {
    /// 最小业务错误码下限 (600)
    pub const MIN_BIZ_CODE: u32 = 600;
    /// 最大业务错误码上限 (999)
    pub const MAX_BIZ_CODE: u32 = 999;

    /// 通用参数非法 / 校验失败 (600)
    pub const PARAM_INVALID: u32 = 600;
    /// 账号已被冻结 (601)
    pub const USER_FROZEN: u32 = 601;
    /// 余额不足 (602)
    pub const INSUFFICIENT_BALANCE: u32 = 602;
    /// 验证码校验失败 (603)
    pub const CAPTCHA_INVALID: u32 = 603;
    /// 业务状态不允许操作 (604)
    pub const INVALID_STATE: u32 = 604;
    /// 频次超限 / 操作受限 (605)
    pub const OPERATION_RESTRICTED: u32 = 605;
}

// 编译期不变量：TTL 必须为正且顺序合理（避免运行期断言被 clippy 判为常量断言）
const _: () = assert!(auth::VERIFICATION_TTL_SECS > 0);
const _: () = assert!(auth::OTP_TTL_SECS > 0);
const _: () = assert!(auth::DEVICE_CODE_TTL_SECS > 0);
const _: () = assert!(auth::VERIFICATION_TTL_SECS <= auth::OTP_TTL_SECS);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_biz_codes_within_documented_range() {
        for code in [
            biz_code::PARAM_INVALID,
            biz_code::USER_FROZEN,
            biz_code::INSUFFICIENT_BALANCE,
            biz_code::CAPTCHA_INVALID,
            biz_code::INVALID_STATE,
            biz_code::OPERATION_RESTRICTED,
        ] {
            assert!(
                (biz_code::MIN_BIZ_CODE..=biz_code::MAX_BIZ_CODE).contains(&code),
                "业务错误码 {code} 超出 600~999 区间"
            );
        }
    }

    #[test]
    fn test_auth_roles_are_lowercase_tokens() {
        // api.md §2.1：枚举值全小写，多词用下划线
        for role in [auth::ROLE_ADMIN, auth::ROLE_OWNER] {
            assert_eq!(role, role.to_lowercase());
            assert!(!role.contains(char::is_whitespace));
        }
    }
}
