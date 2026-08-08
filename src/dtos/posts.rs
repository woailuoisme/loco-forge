//! # 文章模块 DTO 定义
//!
//! 本文件包含文章模块的数据传输对象 (DTO)，实现请求解析与响应解耦：
//! - **Response DTO**：用于将数据库 Model 转换为前端展示对象 [`PostResource`]。
//! - **Request DTO**：用于解析客户端提交的 Payload [`CreatePostRequest`], [`UpdatePostRequest`]。

use sea_orm::prelude::DateTimeWithTimeZone;
use ts_rs::TS;
use utoipa::ToSchema;
use validator::Validate;

// ============================================================================
// Response DTOs (响应数据对象)
// ============================================================================

/// 文章响应数据传输对象 [Response DTO]
///
/// 用于将数据库 SeaORM 实体模型解耦并格式化为标准 JSON 响应。
#[derive(serde::Serialize, serde::Deserialize, TS, ToSchema)]
pub struct PostResource {
    /// 文章 ID
    #[ts(type = "number")]
    pub id: i64,
    /// 文章标题
    pub title: Option<String>,
    /// 文章内容
    pub content: Option<String>,
    /// 创建时间
    #[ts(type = "string")]
    #[schema(value_type = String)]
    pub created_at: DateTimeWithTimeZone,
    /// 更新时间
    #[ts(type = "string")]
    #[schema(value_type = String)]
    pub updated_at: DateTimeWithTimeZone,
}

impl From<crate::models::_entities::posts::Model> for PostResource {
    fn from(m: crate::models::_entities::posts::Model) -> Self {
        Self {
            id: m.id,
            title: m.title,
            content: m.content,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

// ============================================================================
// Request DTOs (请求数据对象)
// ============================================================================

/// 创建文章请求参数 [Request DTO]
///
/// 用于接收和反序列化客户端创建文章时提交的 JSON Payload。
#[derive(serde::Serialize, serde::Deserialize, Validate, TS, ToSchema)]
pub struct CreatePostRequest {
    /// 文章标题
    #[validate(length(min = 1, message = "文章标题不能为空"))]
    pub title: Option<String>,
    /// 文章内容
    pub content: Option<String>,
}

/// 更新文章请求参数 [Request DTO]
///
/// 用于接收和反序列化客户端更新文章时提交的 JSON Payload。
#[derive(serde::Serialize, serde::Deserialize, Validate, TS, ToSchema)]
pub struct UpdatePostRequest {
    /// 文章标题
    #[validate(length(min = 1, message = "文章标题不能为空"))]
    pub title: Option<String>,
    /// 文章内容
    pub content: Option<String>,
}
