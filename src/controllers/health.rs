use loco_rs::prelude::*;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct ReadinessData {
    /// 数据库连接状态 ("ok" 或 错误描述)
    pub db: String,
}

/// Liveness 存活探针
#[utoipa::path(
    get,
    path = "/health",
    summary = "Liveness 存活探针",
    description = "用于 Kubernetes livenessProbe 检查服务进程是否处于存活状态",
    responses(
        (status = 200, description = "服务存活", body = crate::dtos::common::ApiError)
    ),
    tag = "health"
)]
#[debug_handler]
pub async fn liveness() -> Result<Response> {
    crate::dtos::common::format_res::empty_msg("ok")
}

/// Readiness 就绪探针
#[utoipa::path(
    get,
    path = "/health/ready",
    summary = "Readiness 就绪探针",
    description = "用于 Kubernetes readinessProbe 检查数据库等下游依赖是否就绪",
    responses(
        (status = 200, description = "服务已就绪", body = crate::dtos::common::ApiResponse<ReadinessData>),
        (status = 503, description = "服务不可用", body = crate::dtos::common::ApiError)
    ),
    tag = "health"
)]
#[debug_handler]
pub async fn readiness(State(ctx): State<AppContext>) -> Result<Response> {
    match ctx.db.ping().await {
        Ok(_) => crate::dtos::common::format_res::ok_msg(ReadinessData { db: "ok".to_string() }, "ok"),
        Err(err) => crate::dtos::common::format_res::service_unavailable(format!(
            "service unavailable: db ping failed: {}",
            err
        )),
    }
}

pub fn routes() -> Routes {
    Routes::new()
        .add("/health", get(liveness))
        .add("/health/ready", get(readiness))
}
