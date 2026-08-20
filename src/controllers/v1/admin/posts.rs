use loco_rs::prelude::*;

use crate::{
    common::consts::openapi::TAG_POSTS,
    dtos::{
        common::{
            AdminUser, ApiError, ApiResponse, PaginatedResource, PaginationRequest, ValidJson,
            ValidQuery,
        },
        posts::{CreatePostRequest, PostResource, UpdatePostRequest},
    },
    errors::AppResult,
    services::AdminPostsService,
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
    ValidQuery(params): ValidQuery<PaginationRequest>,
) -> AppResult<ApiResponse<PaginatedResource<PostResource>>> {
    let paginated = AdminPostsService::list(&ctx, &params).await?;
    Ok(ApiResponse::ok(paginated))
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
) -> AppResult<ApiResponse<PostResource>> {
    let resource = AdminPostsService::get_by_id(&ctx, id).await?;
    Ok(ApiResponse::ok_with_message(resource, "数据获取成功"))
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
) -> AppResult<ApiResponse<PostResource>> {
    let resource = AdminPostsService::create(&ctx, &params).await?;
    Ok(ApiResponse::created(resource))
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
) -> AppResult<ApiResponse<PostResource>> {
    let updated = AdminPostsService::update(&ctx, id, &params).await?;
    Ok(ApiResponse::ok_with_message(updated, "数据更新成功"))
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
        (status = 200, description = "删除成功", body = ApiResponse<serde_json::Value>),
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
) -> AppResult<ApiResponse<serde_json::Value>> {
    AdminPostsService::delete(&ctx, id).await?;
    Ok(ApiResponse::success_empty("数据删除成功"))
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
