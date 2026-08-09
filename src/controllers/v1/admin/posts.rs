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
    services::admin::PostService,
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
    _auth: auth::JWT,
    State(ctx): State<AppContext>,
    Query(params): Query<PaginationRequest>,
) -> Result<Response> {
    let (dtos, page, per_page, total) = PostService::list(&ctx, &params).await?;
    format_res::page(dtos, page, per_page, total)
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
    _auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(id): Path<i64>,
) -> Result<Response> {
    let Some(dto) = PostService::get_one(&ctx, id).await? else {
        return format_res::not_found(POST_NOT_FOUND);
    };
    format_res::retrieved(dto)
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
    _auth: auth::JWT,
    State(ctx): State<AppContext>,
    ValidJson(params): ValidJson<CreatePostRequest>,
) -> Result<Response> {
    let dto = PostService::create(&ctx, params).await?;
    format_res::created(dto)
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
    _auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(id): Path<i64>,
    ValidJson(params): ValidJson<UpdatePostRequest>,
) -> Result<Response> {
    let Some(dto) = PostService::update(&ctx, id, params).await? else {
        return format_res::not_found(POST_NOT_FOUND);
    };

    format_res::updated(dto)
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
    if !PostService::remove(&ctx, id).await? {
        return format_res::not_found(POST_NOT_FOUND);
    }

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
