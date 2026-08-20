//! # 系统健康检查控制器 (Health Check Controller)
//!
//! 提供 Kubernetes Liveness 存活探针与 Readiness 就绪探针接口。

use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use loco_rs::prelude::*;

pub use crate::common::diagnostics::{
    ComponentHealth, DiagnosticsHub, HealthChecks, ReadinessData,
};
use crate::{common::consts::openapi::TAG_HEALTH, dtos::common::ApiResponse, errors::AppResult};

/// Liveness 存活探针
#[utoipa::path(
    get,
    path = "/health",
    summary = "Liveness 存活探针",
    description = "用于 Kubernetes livenessProbe 检查服务进程是否处于存活状态",
    responses(
        (status = 200, description = "服务存活", body = ApiResponse<serde_json::Value>)
    ),
    tag = TAG_HEALTH
)]
#[debug_handler]
pub async fn liveness() -> AppResult<ApiResponse<serde_json::Value>> {
    Ok(ApiResponse::success_empty("ok"))
}

/// Readiness 就绪探针
#[utoipa::path(
    get,
    path = "/health/ready",
    summary = "Readiness 就绪探针",
    description = "用于 Kubernetes readinessProbe 检测数据库、缓存、存储等基础设施就绪状态",
    responses(
        (status = 200, description = "服务就绪", body = ApiResponse<ReadinessData>),
        (status = 503, description = "服务不可用", body = ApiResponse<ReadinessData>)
    ),
    tag = TAG_HEALTH
)]
#[debug_handler]
pub async fn readiness(State(ctx): State<AppContext>) -> AppResult<Response> {
    let data = DiagnosticsHub::check_readiness(&ctx).await;

    if data.is_all_up() {
        Ok((
            StatusCode::OK,
            Json(ApiResponse::ok_with_message(data, "ok")),
        )
            .into_response())
    } else {
        Ok((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ApiResponse {
                success: false,
                code: 503,
                message: "service unavailable: one or more health probes failed".to_string(),
                data: Some(data),
                errors: None,
                trace: None,
            }),
        )
            .into_response())
    }
}

pub fn routes() -> Routes {
    Routes::new()
        .add("/health", get(liveness))
        .add("/health/ready", get(readiness))
}
