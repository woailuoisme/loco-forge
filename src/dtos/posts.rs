use sea_orm::prelude::DateTimeWithTimeZone;
use ts_rs::TS;
use utoipa::ToSchema;

/// 文章详情数据对象
#[derive(serde::Serialize, serde::Deserialize, TS, ToSchema)]
pub struct PostDto {
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

impl From<crate::models::_entities::posts::Model> for PostDto {
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

/// 创建文章请求参数
#[derive(serde::Serialize, serde::Deserialize, TS, ToSchema)]
pub struct CreatePost {
    /// 文章标题
    pub title: Option<String>,
    /// 文章内容
    pub content: Option<String>,
}

/// 更新文章请求参数
#[derive(serde::Serialize, serde::Deserialize, TS, ToSchema)]
pub struct UpdatePost {
    /// 文章标题
    pub title: Option<String>,
    /// 文章内容
    pub content: Option<String>,
}
