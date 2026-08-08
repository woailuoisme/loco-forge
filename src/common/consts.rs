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
pub mod openapi {
    /// 用户认证与账号管理接口 Tag ("用户认证")
    pub const TAG_AUTH: &str = "用户认证";
    /// 文章管理接口 Tag ("文章管理")
    pub const TAG_POSTS: &str = "文章管理";
    /// 文件存储与下载服务接口 Tag ("文件存储")
    pub const TAG_FILES: &str = "文件存储";
    /// 健康检查探针接口 Tag ("health")
    pub const TAG_HEALTH: &str = "health";
    /// 根路由信息与 API 导航接口 Tag ("home")
    pub const TAG_HOME: &str = "home";
}

/// 用户认证与账号业务常量
pub mod auth {
    /// 允许接收魔术链接的邮箱域名正则 (匹配 @example.com 或 @gmail.com 结尾)
    pub const ALLOW_EMAIL_DOMAIN_PATTERN: &str = r"@example\.com$|@gmail\.com$";
    /// 管理员角色标识
    pub const ROLE_ADMIN: &str = "admin";
}

/// 通用业务提示消息常量
pub mod message {
    /// 文章不存在提示消息
    pub const POST_NOT_FOUND: &str = "post not found";
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
