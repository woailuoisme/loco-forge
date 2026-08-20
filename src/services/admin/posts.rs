//! 管理后台文章管理业务服务模块 (Admin Posts Service)

use loco_rs::prelude::*;

use crate::{
    common::consts::message::POST_NOT_FOUND,
    dtos::{
        common::{PaginateExt, PaginatedResource, PaginationRequest},
        posts::{CreatePostRequest, PostResource, UpdatePostRequest},
    },
    errors::{AppError, AppResult},
    models::posts::{Entity as PostEntity, Model as PostModel},
};

/// 管理后台文章管理服务
pub struct AdminPostsService;

impl AdminPostsService {
    /// 分页获取文章列表
    ///
    /// # Errors
    ///
    /// 当数据库分页查询失败时返回 `AppResult`
    pub async fn list(
        ctx: &AppContext,
        params: &PaginationRequest,
    ) -> AppResult<PaginatedResource<PostResource>> {
        let (posts, total) =
            PostEntity::find_all_paginated(&ctx.db, params.page(), params.per_page()).await?;
        Ok((posts, total).into_paginated_resource(params.page(), params.per_page()))
    }

    /// 根据 ID 获取单篇文章详情
    ///
    /// # Errors
    ///
    /// 当文章不存在时返回 404 Not Found
    pub async fn get_by_id(ctx: &AppContext, id: i64) -> AppResult<PostResource> {
        let post = Self::find_or_404(&ctx.db, id).await?;
        Ok(PostResource::from(post))
    }

    /// 创建新文章
    ///
    /// # Errors
    ///
    /// 当数据库插入失败时返回 `AppResult`
    pub async fn create(ctx: &AppContext, params: &CreatePostRequest) -> AppResult<PostResource> {
        let post = PostModel::create(&ctx.db, params.title.clone(), params.content.clone()).await?;
        Ok(PostResource::from(post))
    }

    /// 根据 ID 更新文章
    ///
    /// # Errors
    ///
    /// 当文章不存在时返回 404 Not Found，更新失败返回 `AppResult`
    pub async fn update(
        ctx: &AppContext,
        id: i64,
        params: &UpdatePostRequest,
    ) -> AppResult<PostResource> {
        let post = Self::find_or_404(&ctx.db, id).await?;
        let updated = post
            .update_post(&ctx.db, params.title.clone(), params.content.clone())
            .await?;
        Ok(PostResource::from(updated))
    }

    /// 根据 ID 删除文章
    ///
    /// # Errors
    ///
    /// 当文章不存在时返回 404 Not Found，删除失败返回 `AppResult`
    pub async fn delete(ctx: &AppContext, id: i64) -> AppResult<()> {
        let post = Self::find_or_404(&ctx.db, id).await?;
        post.delete(&ctx.db).await?;
        Ok(())
    }

    /// 查找文章或返回 404 Not Found
    async fn find_or_404(db: &DatabaseConnection, id: i64) -> AppResult<PostModel> {
        PostModel::find_by_id(db, id)
            .await
            .map_err(|_| AppError::not_found(POST_NOT_FOUND))
    }
}
