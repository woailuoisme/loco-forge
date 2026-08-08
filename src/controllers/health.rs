use std::{
    path::Path,
    time::{Duration, Instant},
};

use axum::body::Bytes;
use loco_rs::{environment::Environment, prelude::*};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::common::consts::openapi::TAG_HEALTH;

/// 单个服务/组件探针指标 (Terminus 风格)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ComponentHealth {
    /// 组件驱动类型 ("pg" | "redis" | "s3" | "memory" 等)
    pub kind: String,
    /// 组件状态 ("up" | "down")
    pub status: String,
    /// 探针响应延迟时间 (毫秒 ms)
    pub latency_ms: f64,
    /// 异常提示说明 (健康时为 None)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// 各基础设施核心组件健康探针集合
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HealthChecks {
    /// 数据库健康探针 (PostgreSQL)
    pub db: ComponentHealth,
    /// 缓存健康探针 (Redis / Valkey / In-Memory)
    pub cache: ComponentHealth,
    /// 对象存储健康探针 (S3 Garage / Local)
    pub storage: ComponentHealth,
}

/// 就绪探针总体响应结构
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ReadinessData {
    /// 总体服务就绪状态 ("up" | "down")
    pub status: String,
    /// 基础设施各组件详细探针指标
    pub checks: HealthChecks,
}

/// 探针: 执行数据库 (PostgreSQL) ping 存活性与延迟检测
async fn check_db(ctx: &AppContext) -> ComponentHealth {
    let start = Instant::now();
    let kind = if ctx.config.database.uri.starts_with("postgres") || ctx.config.database.uri.starts_with("postgresql") {
        "pg"
    } else if ctx.config.database.uri.starts_with("mysql") {
        "mysql"
    } else {
        "sqlite"
    };

    match tokio::time::timeout(Duration::from_secs(3), ctx.db.ping()).await {
        Ok(Ok(_)) => ComponentHealth {
            kind: kind.to_string(),
            status: "up".to_string(),
            latency_ms: (start.elapsed().as_secs_f64() * 100_000.0).round() / 100.0,
            error: None,
        },
        Ok(Err(err)) => ComponentHealth {
            kind: kind.to_string(),
            status: "down".to_string(),
            latency_ms: (start.elapsed().as_secs_f64() * 100_000.0).round() / 100.0,
            error: Some(format!("db ping failed: {err}")),
        },
        Err(_) => ComponentHealth {
            kind: kind.to_string(),
            status: "down".to_string(),
            latency_ms: (start.elapsed().as_secs_f64() * 100_000.0).round() / 100.0,
            error: Some("db ping timed out (3s)".to_string()),
        },
    }
}

/// 探针: 执行缓存 (Redis/Valkey/Memory) 写入、读取与删除探针检测（使用 UUID 隔离 Key 防止并发碰撞）
async fn check_cache(ctx: &AppContext) -> ComponentHealth {
    let start = Instant::now();
    let kind = if ctx.environment == Environment::Test {
        "memory"
    } else {
        "redis"
    };
    let probe_id = uuid::Uuid::new_v4().simple();
    let test_key = format!("_health_ready_probe:{probe_id}");
    let test_val = "ping";

    let probe = async {
        ctx.cache
            .insert(&test_key, &test_val)
            .await
            .map_err(|e| format!("cache write failed: {e}"))?;
        let res: Option<String> = ctx
            .cache
            .get(&test_key)
            .await
            .map_err(|e| format!("cache read failed: {e}"))?;
        let _ = ctx.cache.remove(&test_key).await;
        if res.as_deref() != Some(test_val) {
            return Err("cache value mismatch".to_string());
        }
        Ok(())
    };

    match tokio::time::timeout(Duration::from_secs(3), probe).await {
        Ok(Ok(_)) => ComponentHealth {
            kind: kind.to_string(),
            status: "up".to_string(),
            latency_ms: (start.elapsed().as_secs_f64() * 1000.0 * 100.0).round() / 100.0,
            error: None,
        },
        Ok(Err(err)) => ComponentHealth {
            kind: kind.to_string(),
            status: "down".to_string(),
            latency_ms: (start.elapsed().as_secs_f64() * 1000.0 * 100.0).round() / 100.0,
            error: Some(err),
        },
        Err(_) => ComponentHealth {
            kind: kind.to_string(),
            status: "down".to_string(),
            latency_ms: (start.elapsed().as_secs_f64() * 1000.0 * 100.0).round() / 100.0,
            error: Some("cache probe timed out (3s)".to_string()),
        },
    }
}

/// 探针: 执行对象存储 (S3 Garage / Local) 上传与删除探针检测（使用 UUID 隔离文件名防止并发碰撞）
async fn check_storage(ctx: &AppContext) -> ComponentHealth {
    let start = Instant::now();
    let kind = if ctx.environment == Environment::Test {
        "memory"
    } else {
        "s3"
    };
    let probe_id = uuid::Uuid::new_v4().simple();
    let filename = format!("_health_ready_probe_{probe_id}.txt");
    let test_path = Path::new(&filename);
    let test_bytes = Bytes::from_static(b"ping");

    let probe = async {
        let _ = tokio::fs::create_dir_all("uploads").await;
        ctx.storage
            .upload(test_path, &test_bytes)
            .await
            .map_err(|e| format!("storage upload failed: {e}"))?;
        let _ = ctx.storage.delete(test_path).await;
        Ok(())
    };

    match tokio::time::timeout(Duration::from_secs(3), probe).await {
        Ok(Ok(_)) => ComponentHealth {
            kind: kind.to_string(),
            status: "up".to_string(),
            latency_ms: (start.elapsed().as_secs_f64() * 1000.0 * 100.0).round() / 100.0,
            error: None,
        },
        Ok(Err(err)) => ComponentHealth {
            kind: kind.to_string(),
            status: "down".to_string(),
            latency_ms: (start.elapsed().as_secs_f64() * 1000.0 * 100.0).round() / 100.0,
            error: Some(err),
        },
        Err(_) => ComponentHealth {
            kind: kind.to_string(),
            status: "down".to_string(),
            latency_ms: (start.elapsed().as_secs_f64() * 1000.0 * 100.0).round() / 100.0,
            error: Some("storage probe timed out (3s)".to_string()),
        },
    }
}

/// Liveness 存活探针
#[utoipa::path(
    get,
    path = "/health",
    summary = "Liveness 存活探针",
    description = "用于 Kubernetes livenessProbe 检查服务进程是否处于存活状态",
    responses(
        (status = 200, description = "服务存活", body = crate::dtos::common::ApiResponse<serde_json::Value>)
    ),
    tag = TAG_HEALTH
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
    description = "用于 Kubernetes readinessProbe 类似 Terminus 指标检测数据库、缓存、存储依赖就绪状态",
    responses(
        (status = 200, description = "服务就绪", body = crate::dtos::common::ApiResponse<ReadinessData>),
        (status = 503, description = "服务不可用", body = crate::dtos::common::ApiResponse<ReadinessData>)
    ),
    tag = TAG_HEALTH
)]
#[debug_handler]
pub async fn readiness(State(ctx): State<AppContext>) -> Result<Response> {
    let (db, cache, storage) = tokio::join!(check_db(&ctx), check_cache(&ctx), check_storage(&ctx));

    let is_all_up = db.status == "up" && cache.status == "up" && storage.status == "up";
    let status = if is_all_up { "up" } else { "down" };

    let data = ReadinessData {
        status: status.to_string(),
        checks: HealthChecks { db, cache, storage },
    };

    if is_all_up {
        crate::dtos::common::format_res::ok_msg(data, "ok")
    } else {
        crate::dtos::common::format_res::service_unavailable_data(
            data,
            "service unavailable: one or more health probes failed",
        )
    }
}

pub fn routes() -> Routes {
    Routes::new()
        .add("/health", get(liveness))
        .add("/health/ready", get(readiness))
}
