use async_trait::async_trait;
use axum::{routing::get, Json, Router};
use loco_rs::{
    app::{AppContext, Initializer},
    Result,
};
use utoipa::{
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_scalar::{Scalar, Servable as ScalarServable};

use crate::{controllers, dtos, models, views};

#[derive(OpenApi)]
#[openapi(
    paths(
        controllers::home::index,
        controllers::auth::register,
        controllers::auth::verify,
        controllers::auth::login,
        controllers::auth::forgot,
        controllers::auth::reset,
        controllers::auth::current,
        controllers::auth::magic_link,
        controllers::auth::magic_link_verify,
        controllers::auth::resend_verification_email,
        controllers::posts::list,
        controllers::posts::get_one,
        controllers::posts::create,
        controllers::posts::update,
        controllers::posts::remove,
    ),
    components(
        schemas(
            controllers::home::HomeResponse,
            controllers::home::DocsLocation,
            dtos::common::ApiError,
            dtos::common::Page<dtos::posts::PostDto>,
            dtos::posts::PostDto,
            dtos::posts::CreatePost,
            dtos::posts::UpdatePost,
            controllers::posts::ListParams,
            controllers::auth::ForgotParams,
            controllers::auth::ResetParams,
            controllers::auth::MagicLinkParams,
            controllers::auth::ResendVerificationParams,
            models::users::LoginParams,
            models::users::RegisterParams,
            views::auth::LoginResponse,
            views::auth::CurrentResponse,
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "home", description = "应用系统入口与健康状态接口"),
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
