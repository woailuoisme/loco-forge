#![allow(clippy::unused_async)]
use axum::http::StatusCode;
use loco_rs::prelude::*;
use sea_orm::{PaginatorTrait, QueryOrder};
use serde::Deserialize;

use crate::{
    dtos::{
        common::{ApiError, Page},
        posts::{CreatePost, PostDto, UpdatePost},
    },
    models::_entities::posts::{ActiveModel, Column, Entity},
};

const fn default_page() -> u64 {
    1
}

const fn default_per_page() -> u64 {
    25
}

#[derive(Debug, Deserialize, utoipa::IntoParams, utoipa::ToSchema)]
pub struct ListParams {
    /// 页码（从 1 开始，默认 1）
    #[serde(default = "default_page")]
    pub page: u64,
    /// 每页数量（默认 25）
    #[serde(default = "default_per_page")]
    pub per_page: u64,
}

/// 构建包含 [`ApiError`] 规范的 404 错误响应
fn not_found(message: &str) -> Response {
    let error = ApiError {
        code: "not_found".to_string(),
        message: message.to_string(),
        details: None,
    };
    (StatusCode::NOT_FOUND, Json(error)).into_response()
}

/// 获取文章列表
#[utoipa::path(
    get,
    path = "/api/posts",
    summary = "获取文章列表",
    description = "分页查询文章列表",
    params(ListParams),
    responses(
        (status = 200, description = "获取成功", body = Page<PostDto>),
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "posts"
)]
#[debug_handler]
pub async fn list(
    _auth: auth::JWT,
    State(ctx): State<AppContext>,
    Query(params): Query<ListParams>,
) -> Result<Json<Page<PostDto>>> {
    let page = params.page.max(1);
    let per_page = params.per_page.max(1);

    let query = Entity::find().order_by_asc(Column::Id);

    let paginator = query.paginate(&ctx.db, per_page);
    let total = paginator.num_items_and_pages().await?.number_of_items;
    let items = paginator.fetch_page(page - 1).await?;

    Ok(Json(Page {
        items: items.into_iter().map(PostDto::from).collect(),
        total: i64::try_from(total).unwrap_or(i64::MAX),
        page: i64::try_from(page).unwrap_or(i64::MAX),
        per_page: i64::try_from(per_page).unwrap_or(i64::MAX),
    }))
}

/// 获取指定文章
#[utoipa::path(
    get,
    path = "/api/posts/{id}",
    summary = "获取指定文章",
    description = "根据文章 ID 查询文章详情",
    params(
        ("id" = i64, Path, description = "文章 ID")
    ),
    responses(
        (status = 200, description = "获取成功", body = PostDto),
        (status = 404, description = "文章不存在", body = ApiError)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "posts"
)]
#[debug_handler]
pub async fn get_one(_auth: auth::JWT, State(ctx): State<AppContext>, Path(id): Path<i64>) -> Result<Response> {
    let Some(model) = Entity::find_by_id(id).one(&ctx.db).await? else {
        return Ok(not_found("post not found"));
    };
    Ok(Json(PostDto::from(model)).into_response())
}

/// 创建新文章
#[utoipa::path(
    post,
    path = "/api/posts",
    summary = "创建新文章",
    description = "提交标题与内容创建新文章",
    request_body = CreatePost,
    responses(
        (status = 201, description = "创建成功", body = PostDto)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "posts"
)]
#[debug_handler]
pub async fn create(
    _auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(params): Json<CreatePost>,
) -> Result<Response> {
    let item = ActiveModel {
        title: Set(params.title),
        content: Set(params.content),
        ..Default::default()
    };
    let item = item.insert(&ctx.db).await?;

    Ok((StatusCode::CREATED, Json(PostDto::from(item))).into_response())
}

/// 更新文章
#[utoipa::path(
    put,
    path = "/api/posts/{id}",
    summary = "更新文章",
    description = "根据文章 ID 更新标题与内容",
    params(
        ("id" = i64, Path, description = "文章 ID")
    ),
    request_body = UpdatePost,
    responses(
        (status = 200, description = "更新成功", body = PostDto),
        (status = 404, description = "文章不存在", body = ApiError)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "posts"
)]
#[debug_handler]
pub async fn update(
    _auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(id): Path<i64>,
    Json(params): Json<UpdatePost>,
) -> Result<Response> {
    let Some(model) = Entity::find_by_id(id).one(&ctx.db).await? else {
        return Ok(not_found("post not found"));
    };

    let mut item = model.into_active_model();
    item.title = Set(params.title);
    item.content = Set(params.content);
    let item = item.update(&ctx.db).await?;

    Ok(Json(PostDto::from(item)).into_response())
}

/// 删除文章
#[utoipa::path(
    delete,
    path = "/api/posts/{id}",
    summary = "删除文章",
    description = "根据文章 ID 删除对应文章",
    params(
        ("id" = i64, Path, description = "文章 ID")
    ),
    responses(
        (status = 204, description = "删除成功"),
        (status = 404, description = "文章不存在", body = ApiError)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "posts"
)]
#[debug_handler]
pub async fn remove(_auth: auth::JWT, State(ctx): State<AppContext>, Path(id): Path<i64>) -> Result<Response> {
    let Some(model) = Entity::find_by_id(id).one(&ctx.db).await? else {
        return Ok(not_found("post not found"));
    };

    model.into_active_model().delete(&ctx.db).await?;

    Ok(StatusCode::NO_CONTENT.into_response())
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/posts")
        .add("/", get(list))
        .add("/", post(create))
        .add("/{id}", get(get_one))
        .add("/{id}", put(update))
        .add("/{id}", delete(remove))
}
