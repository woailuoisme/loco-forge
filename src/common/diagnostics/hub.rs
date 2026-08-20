//! # 基础设施健康诊断与探针门面 (Diagnostics Hub)

use std::{
    path::Path,
    time::{Duration, Instant},
};

use axum::body::Bytes;
use loco_rs::{environment::Environment, prelude::*};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// 单个服务/组件探针指标 (Terminus 风格)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ComponentHealth {
    /// 组件驱动类型 ("pg" | "mysql" | "sqlite" | "redis" | "memory" | "s3" | "local")
    pub kind: String,
    /// 组件状态 ("up" | "down")
    pub status: String,
    /// 探针响应延迟时间 (毫秒 ms)
    pub latency_ms: f64,
    /// 异常提示说明 (健康时为 None)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl ComponentHealth {
    #[must_use]
    pub fn is_up(&self) -> bool {
        self.status == "up"
    }
}

/// 各基础设施核心组件健康探针集合
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HealthChecks {
    /// 数据库健康探针 (PostgreSQL / MySQL / SQLite)
    pub db: ComponentHealth,
    /// 缓存健康探针 (Redis / Valkey / In-Memory)
    pub cache: ComponentHealth,
    /// 对象存储健康探针 (S3 Garage / Local / Memory)
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

impl ReadinessData {
    #[must_use]
    pub fn is_all_up(&self) -> bool {
        self.checks.db.is_up() && self.checks.cache.is_up() && self.checks.storage.is_up()
    }
}

/// 统一健康诊断中心
pub struct DiagnosticsHub;

impl DiagnosticsHub {
    /// 探针: 执行数据库 ping 存活性与延迟检测
    pub async fn check_db(ctx: &AppContext) -> ComponentHealth {
        let start = Instant::now();
        let kind = if ctx.config.database.uri.starts_with("postgres")
            || ctx.config.database.uri.starts_with("postgresql")
        {
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

    /// 探针: 执行缓存写入、读取与删除检测（使用 UUID 隔离 Key 防止并发碰撞）
    pub async fn check_cache(ctx: &AppContext) -> ComponentHealth {
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

    /// 探针: 执行对象存储上传与删除检测（使用 UUID 隔离文件名防止并发碰撞）
    pub async fn check_storage(ctx: &AppContext) -> ComponentHealth {
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

    /// 并发检查所有就绪探针并汇总
    pub async fn check_readiness(ctx: &AppContext) -> ReadinessData {
        let (db, cache, storage) = tokio::join!(
            Self::check_db(ctx),
            Self::check_cache(ctx),
            Self::check_storage(ctx)
        );

        let is_all_up = db.is_up() && cache.is_up() && storage.is_up();
        let status = if is_all_up { "up" } else { "down" };

        ReadinessData {
            status: status.to_string(),
            checks: HealthChecks { db, cache, storage },
        }
    }
}
