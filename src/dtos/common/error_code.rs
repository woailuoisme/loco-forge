//! 标准业务错误码模块 (Business Error Codes)
//!
//! 定义 [`ErrorCode`] 结构体与项目全局预置的常用错误码常量，
//! 每个错误码内置 6 位数字码与默认描述消息，HTTP 状态码由前 3 位自动推导。

use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// 标准业务错误码定义结构 (Business Error Code Definition)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ErrorCode {
    /// 6 位数字错误码 (如 400001, 401002, 404001, 500001)
    pub code: u32,
    /// 默认人类可读错误描述消息
    pub message: &'static str,
}

impl ErrorCode {
    /// 构造新的业务错误码
    pub const fn new(code: u32, message: &'static str) -> Self {
        Self { code, message }
    }

    /// 获取 HTTP 状态码（根据错误码前 3 位解析，若未匹配则默认 400 BAD_REQUEST）
    pub fn status_code(&self) -> StatusCode {
        u16::try_from(self.code / 1000)
            .ok()
            .and_then(|s| StatusCode::from_u16(s).ok())
            .unwrap_or(StatusCode::BAD_REQUEST)
    }

    // =========================================================================
    // 常用预定义业务错误码常量 (Common Predefined Error Codes)
    // =========================================================================

    // 400xxx 客户端通用与请求错误 (HTTP 400)
    pub const BAD_REQUEST: Self = Self::new(400000, "客户端无效请求");
    pub const PARAM_INVALID: Self = Self::new(400001, "请求参数无效或格式错误");
    pub const JSON_PARSE_ERROR: Self = Self::new(400002, "JSON 请求体解析失败");

    // 401xxx 认证类错误 (HTTP 401)
    pub const UNAUTHORIZED: Self = Self::new(401001, "未登录或 Token 已失效");
    pub const TOKEN_EXPIRED: Self = Self::new(401002, "登录 Token 已过期");
    pub const CREDENTIALS_INVALID: Self = Self::new(401003, "用户名或密码错误");

    // 403xxx 权限与状态限制类错误 (HTTP 403)
    pub const FORBIDDEN: Self = Self::new(403001, "无权访问该资源");
    pub const ACCOUNT_DISABLED: Self = Self::new(403002, "账号已被禁用或冻结");

    // 404xxx 资源与路由未找到类错误 (HTTP 404)
    pub const NOT_FOUND: Self = Self::new(404000, "请求的目标资源未找到");
    pub const USER_NOT_FOUND: Self = Self::new(404001, "未找到目标用户");
    pub const RECORD_NOT_FOUND: Self = Self::new(404002, "目标数据记录不存在");
    pub const ROUTE_NOT_FOUND: Self = Self::new(404003, "请求的接口路径不存在");

    // 409xxx 状态冲突类错误 (HTTP 409)
    pub const CONFLICT: Self = Self::new(409000, "资源状态发生冲突");
    pub const USER_ALREADY_EXISTS: Self = Self::new(409001, "目标用户已被注册");
    pub const DUPLICATE_RECORD: Self = Self::new(409002, "数据唯一约束冲突");

    // 422xxx 数据校验类错误 (HTTP 422)
    pub const VALIDATION_ERROR: Self = Self::new(422000, "数据输入校验失败");

    // 429xxx 频率限制类错误 (HTTP 429)
    pub const TOO_MANY_REQUESTS: Self = Self::new(429000, "请求过于频繁，请稍后再试");

    // 500xxx 服务器内部错误 (HTTP 500)
    pub const INTERNAL_ERROR: Self = Self::new(500000, "服务器内部发生未捕获异常");
    pub const DATABASE_ERROR: Self = Self::new(500001, "数据库操作处理异常");

    // 503xxx 服务维护与不可用 (HTTP 503)
    pub const SERVICE_UNAVAILABLE: Self = Self::new(503000, "系统服务维护中，暂不可用");
}
