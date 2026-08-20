use loco_rs::prelude::*;

use crate::{
    common::consts::{message::POST_NOT_FOUND, openapi::TAG_POSTS},
    dtos::{
        common::{
            AdminUser, ApiError, ApiResponse, PaginatedResource, PaginationRequest, ValidJson,
            format_res,
        },
        posts::{CreatePostRequest, PostResource, UpdatePostRequest},
    },
    models::posts::{Entity as PostEntity, Model as PostModel},
};

/// 获取文章列表
#[utoipa::path(
    get,
    path = "/api/v1/admin/posts",
    summary = "获取文章列表",
    description = "分页查询文章列表",
    params(PaginationRequest),
    responses(
        (status = 200, description = "获取成功", body = ApiResponse<PaginatedResource<PostResource>>),
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = TAG_POSTS
)]
#[debug_handler]
pub async fn list(
    _admin: AdminUser,
    State(ctx): State<AppContext>,
    Query(params): Query<PaginationRequest>,
) -> Result<Response> {
    let (posts, total) =
        PostEntity::find_all_paginated(&ctx.db, params.page(), params.per_page()).await?;
    let dtos: Vec<PostResource> = posts.into_iter().map(PostResource::from).collect();
    format_res::page(dtos, params.page(), params.per_page(), total)
}

/// 获取指定文章
#[utoipa::path(
    get,
    path = "/api/v1/admin/posts/{id}",
    summary = "获取指定文章",
    description = "根据文章 ID 查询文章详情",
    params(
        ("id" = i64, Path, description = "文章 ID")
    ),
    responses(
        (status = 200, description = "获取成功", body = ApiResponse<PostResource>),
        (status = 404, description = "文章不存在", body = ApiError)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = TAG_POSTS
)]
#[debug_handler]
pub async fn get_one(
    _admin: AdminUser,
    State(ctx): State<AppContext>,
    Path(id): Path<i64>,
) -> Result<Response> {
    let Ok(post) = PostModel::find_by_id(&ctx.db, id).await else {
        return format_res::not_found(POST_NOT_FOUND);
    };
    format_res::retrieved(PostResource::from(post))
}

/// 创建新文章
#[utoipa::path(
    post,
    path = "/api/v1/admin/posts",
    summary = "创建新文章",
    description = "提交标题与内容创建新文章",
    request_body = CreatePostRequest,
    responses(
        (status = 201, description = "创建成功", body = ApiResponse<PostResource>)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = TAG_POSTS
)]
#[debug_handler]
pub async fn create(
    _admin: AdminUser,
    State(ctx): State<AppContext>,
    ValidJson(params): ValidJson<CreatePostRequest>,
) -> Result<Response> {
    let post = PostModel::create(&ctx.db, params.title, params.content).await?;
    format_res::created(PostResource::from(post))
}

/// 更新文章
#[utoipa::path(
    put,
    path = "/api/v1/admin/posts/{id}",
    summary = "更新文章",
    description = "根据文章 ID 更新标题与内容",
    params(
        ("id" = i64, Path, description = "文章 ID")
    ),
    request_body = UpdatePostRequest,
    responses(
        (status = 200, description = "更新成功", body = ApiResponse<PostResource>),
        (status = 404, description = "文章不存在", body = ApiError)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = TAG_POSTS
)]
#[debug_handler]
pub async fn update(
    _admin: AdminUser,
    State(ctx): State<AppContext>,
    Path(id): Path<i64>,
    ValidJson(params): ValidJson<UpdatePostRequest>,
) -> Result<Response> {
    let Ok(post) = PostModel::find_by_id(&ctx.db, id).await else {
        return format_res::not_found(POST_NOT_FOUND);
    };

    let updated = post
        .update_post(&ctx.db, params.title, params.content)
        .await?;
    format_res::updated(PostResource::from(updated))
}

/// 删除文章
#[utoipa::path(
    delete,
    path = "/api/v1/admin/posts/{id}",
    summary = "删除文章",
    description = "根据文章 ID 删除对应文章",
    params(
        ("id" = i64, Path, description = "文章 ID")
    ),
    responses(
        (status = 200, description = "删除成功", body = ApiError),
        (status = 404, description = "文章不存在", body = ApiError)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = TAG_POSTS
)]
#[debug_handler]
pub async fn remove(
    _admin: AdminUser,
    State(ctx): State<AppContext>,
    Path(id): Path<i64>,
) -> Result<Response> {
    let Ok(post) = PostModel::find_by_id(&ctx.db, id).await else {
        return format_res::not_found(POST_NOT_FOUND);
    };

    post.delete(&ctx.db).await?;
    format_res::deleted()
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/v1/admin/posts")
        .add("/", get(list))
        .add("/", post(create))
        .add("/{id}", get(get_one))
        .add("/{id}", put(update))
        .add("/{id}", delete(remove))
}
