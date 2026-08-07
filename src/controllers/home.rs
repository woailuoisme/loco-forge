use loco_rs::prelude::*;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct DocsLocation {
    /// Scalar 交互式 API 文档链接
    pub docs: String,
    /// OpenAPI Specification JSON 描述链接
    pub openapi: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HomeResponse {
    /// 应用名称
    pub name: String,
    /// 应用版本号
    pub version: String,
    /// 服务运行状态
    pub status: String,
    /// 文档地址说明
    pub docs: DocsLocation,
}

/// 根路由信息与 API 导航入口
#[utoipa::path(
    get,
    path = "/",
    summary = "获取根路由信息",
    description = "返回应用基本信息、运行状态及交互式 API 文档入口",
    responses(
        (status = 200, description = "获取成功", body = crate::dtos::common::ApiResponse<HomeResponse>)
    ),
    tag = "home"
)]
#[debug_handler]
pub async fn index() -> Result<Response> {
    crate::dtos::common::format_res::ok(HomeResponse {
        name: env!("CARGO_CRATE_NAME").to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        status: "ok".to_string(),
        docs: DocsLocation {
            docs: "/docs".to_string(),
            openapi: "/api-docs/openapi.json".to_string(),
        },
    })
}

#[debug_handler]
pub async fn favicon() -> axum::http::StatusCode {
    axum::http::StatusCode::NO_CONTENT
}

pub fn routes() -> Routes {
    Routes::new().add("/", get(index)).add("/favicon.ico", get(favicon))
}
