use async_trait::async_trait;
use axum::{Json, Router, routing::get};
use loco_rs::{
    Result,
    app::{AppContext, Initializer},
};
use utoipa::{
    Modify, OpenApi,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};
use utoipa_scalar::{Scalar, Servable as ScalarServable};

use crate::{controllers, dtos, models, views};

#[derive(OpenApi)]
#[openapi(
    paths(
        controllers::home::index,
        controllers::health::liveness,
        controllers::health::readiness,
        controllers::v1::auth::register,
        controllers::v1::auth::verify,
        controllers::v1::auth::login,
        controllers::v1::auth::forgot,
        controllers::v1::auth::reset,
        controllers::v1::auth::current,
        controllers::v1::auth::magic_link,
        controllers::v1::auth::magic_link_verify,
        controllers::v1::auth::resend_verification_email,
        controllers::v1::admin::posts::list,
        controllers::v1::admin::posts::get_one,
        controllers::v1::admin::posts::create,
        controllers::v1::admin::posts::update,
        controllers::v1::admin::posts::remove,
        controllers::v1::files::upload,
        controllers::v1::files::get_file,
    ),
    components(
        schemas(
            controllers::home::HomeResponse,
            controllers::home::DocsLocation,
            controllers::health::ReadinessData,
            controllers::v1::files::FileUploadResponse,
            dtos::common::ApiError,
            dtos::common::Page<dtos::posts::PostDto>,
            dtos::common::PaginationQuery,
            dtos::posts::PostDto,
            dtos::posts::CreatePost,
            dtos::posts::UpdatePost,
            controllers::v1::auth::ForgotParams,
            controllers::v1::auth::ResetParams,
            controllers::v1::auth::MagicLinkParams,
            controllers::v1::auth::ResendVerificationParams,
            models::users::LoginParams,
            models::users::RegisterParams,
            views::auth::LoginResponse,
            views::auth::CurrentResponse,
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "home", description = "应用系统入口接口"),
        (name = "health", description = "服务存活与就绪健康探针接口"),
        (name = "files", description = "文件上传与静态资源下载服务接口"),
        (name = "auth", description = "用户认证与账号管理接口"),
        (name = "posts", description = "文章管理与增删改查接口")
    ),
    info(
        title = "Loco RESTful API 文档",
        version = "1.0.0",
        description = "基于 utoipa 与 Scalar 驱动的 RESTful API 接口规范说明"
    )
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}

pub struct OpenApiInitializer;

#[async_trait]
impl Initializer for OpenApiInitializer {
    fn name(&self) -> String {
        "openapi".to_string()
    }

    async fn after_routes(&self, router: Router, _ctx: &AppContext) -> Result<Router> {
        let openapi = ApiDoc::openapi();

        let router = router
            .route(
                "/api-docs/openapi.json",
                get({
                    let openapi_json = openapi.clone();
                    move || async move { Json(openapi_json) }
                }),
            )
            .merge(Scalar::with_url("/docs", openapi));

        Ok(router)
    }
}
