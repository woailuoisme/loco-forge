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
use utoipa_redoc::{Redoc, Servable as RedocServable};
use utoipa_scalar::{Scalar, Servable as ScalarServable};

use crate::{
    common::consts::openapi::{TAG_AUTH, TAG_FILES, TAG_POSTS},
    controllers, dtos, models,
};

#[derive(OpenApi)]
#[openapi(
    paths(
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
            dtos::files::FileUploadResource,
            dtos::common::ApiError,
            // 无载荷成功响应 (注册/验证/重置密码/魔术链接等)
            dtos::common::ApiResponse,
            // 文章接口具化响应
            dtos::common::ApiResponse<dtos::posts::PostResource>,
            dtos::common::ApiResponse<dtos::common::PaginatedResource<dtos::posts::PostResource>>,
            // 认证接口具化响应
            dtos::common::ApiResponse<dtos::auth::LoginResource>,
            dtos::common::ApiResponse<dtos::auth::CurrentUserResource>,
            // 文件上传具化响应
            dtos::common::ApiResponse<Vec<dtos::files::FileUploadResource>>,
            // 分页与通用类型
            dtos::common::Page<dtos::posts::PostResource>,
            dtos::common::PaginationRequest,
            dtos::posts::PostResource,
            dtos::posts::CreatePostRequest,
            dtos::posts::UpdatePostRequest,
            dtos::auth::ForgotPasswordRequest,
            dtos::auth::ResetPasswordRequest,
            dtos::auth::MagicLinkRequest,
            dtos::auth::ResendVerificationRequest,
            models::users::LoginRequest,
            models::users::RegisterRequest,
            dtos::auth::LoginResource,
            dtos::auth::CurrentUserResource,
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = TAG_AUTH, description = "用户认证与账号管理接口"),
        (name = TAG_POSTS, description = "文章管理与增删改查接口"),
        (name = TAG_FILES, description = "文件上传与静态资源下载服务接口"),
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
            .merge(Scalar::with_url("/docs", openapi.clone()).custom_html(
                r#"<!doctype html>
<html>
<head>
    <title>Loco RESTful API 文档</title>
    <meta charset="utf-8"/>
    <meta name="viewport" content="width=device-width, initial-scale=1"/>
</head>
<body>

<script
        id="api-reference"
        data-configuration='{"hideModels": true}'
        type="application/json">
    $spec
</script>
<script src="https://cdn.jsdelivr.net/npm/@scalar/api-reference"></script>
</body>
</html>"#,
            ))
            .merge(Redoc::with_url("/redoc", openapi));

        Ok(router)
    }
}
