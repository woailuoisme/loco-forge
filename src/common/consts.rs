//! # 全局常量统一定义模块 (Global Constants)
//!
//! 提供系统应用、日志与存储、分页限制等全局常量的统一声明与管理。

/// 应用与环境标识常量
pub mod app {
    /// 应用默认名称
    pub const APP_NAME: &str = "loco-starter";
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
}

/// HTTP 接口通用分页限制常量
pub mod pagination {
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
}
