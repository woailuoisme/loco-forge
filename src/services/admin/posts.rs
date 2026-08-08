use loco_rs::prelude::*;
use sea_orm::{PaginatorTrait, QueryOrder};

use crate::{
    dtos::{
        common::PaginationRequest,
        posts::{CreatePostRequest, PostResource, UpdatePostRequest},
    },
    models::_entities::posts::{ActiveModel, Column, Entity},
};

pub struct PostService;

impl PostService {
    /// 分页查询文章列表
    pub async fn list(ctx: &AppContext, params: &PaginationRequest) -> Result<(Vec<PostResource>, u64, u64, u64)> {
        let page = params.page();
        let per_page = params.per_page();

        let query = Entity::find().order_by_asc(Column::Id);
        let paginator = query.paginate(&ctx.db, per_page);
        let total = paginator.num_items_and_pages().await?.number_of_items;
        let items = paginator.fetch_page(page - 1).await?;

        let dtos: Vec<PostResource> = items.into_iter().map(PostResource::from).collect();
        Ok((dtos, page, per_page, total))
    }

    /// 根据 ID 查询文章详情
    pub async fn get_one(ctx: &AppContext, id: i64) -> Result<Option<PostResource>> {
        let model = Entity::find_by_id(id).one(&ctx.db).await?;
        Ok(model.map(PostResource::from))
    }

    /// 创建新文章
    pub async fn create(ctx: &AppContext, params: CreatePostRequest) -> Result<PostResource> {
        let item = ActiveModel {
            title: Set(params.title),
            content: Set(params.content),
            ..Default::default()
        };
        let item = item.insert(&ctx.db).await?;
        Ok(PostResource::from(item))
    }

    /// 更新文章
    pub async fn update(ctx: &AppContext, id: i64, params: UpdatePostRequest) -> Result<Option<PostResource>> {
        let Some(model) = Entity::find_by_id(id).one(&ctx.db).await? else {
            return Ok(None);
        };

        let mut item = model.into_active_model();
        item.title = Set(params.title);
        item.content = Set(params.content);
        let item = item.update(&ctx.db).await?;

        Ok(Some(PostResource::from(item)))
    }

    /// 删除文章
    pub async fn remove(ctx: &AppContext, id: i64) -> Result<bool> {
        let Some(model) = Entity::find_by_id(id).one(&ctx.db).await? else {
            return Ok(false);
        };

        model.into_active_model().delete(&ctx.db).await?;
        Ok(true)
    }
}
