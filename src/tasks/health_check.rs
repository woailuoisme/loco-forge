//! # 基础设施健康检查任务
//!
//! 提供 `cargo loco task health:check` 命令行任务，用于对数据库 (PostgreSQL)、缓存 (Redis/Valkey) 和对象存储 (S3 Garage) 进行连接与读写探针测试。

use axum::body::Bytes;
use loco_rs::prelude::*;
use std::{
    path::Path,
    time::{Duration, Instant},
};

/// 基础设施健康检查任务
pub struct HealthCheck;

/// 健康检查目标过滤器
#[derive(Debug, PartialEq, Eq)]
struct TargetFilter {
    db: bool,
    cache: bool,
    storage: bool,
}

impl TargetFilter {
    /// 从命令行参数中解析检测目标筛选条件
    fn from_vars(vars: &task::Vars) -> Self {
        let raw = vars
            .cli_arg("target")
            .or_else(|_| vars.cli_arg("--target"))
            .or_else(|_| vars.cli_arg("only"))
            .or_else(|_| vars.cli_arg("--only"))
            .ok();

        let mut filter = Self {
            db: false,
            cache: false,
            storage: false,
        };

        if let Some(target_str) = raw {
            let cleaned = target_str.trim_start_matches("--target=").trim_start_matches("--only=");
            for item in cleaned.split(',') {
                match item.trim().to_lowercase().as_str() {
                    "db" | "database" => filter.db = true,
                    "cache" | "redis" | "valkey" => filter.cache = true,
                    "storage" | "s3" => filter.storage = true,
                    "all" => return Self::all(),
                    _ => {}
                }
            }
        }

        if !filter.db && !filter.cache && !filter.storage {
            Self::all()
        } else {
            filter
        }
    }

    fn all() -> Self {
        Self {
            db: true,
            cache: true,
            storage: true,
        }
    }
}

/// 数据库 (PostgreSQL) ping 探针
async fn check_db(ctx: &AppContext) -> std::result::Result<Duration, String> {
    let start = Instant::now();
    ctx.db.ping().await.map_err(|e| e.to_string())?;
    Ok(start.elapsed())
}

/// 缓存 (Redis/Valkey/Memory) 读写与删除探针（动态生成唯一 Key 避免并发竞态）
async fn check_cache(ctx: &AppContext) -> std::result::Result<Duration, String> {
    let start = Instant::now();
    let probe_id = uuid::Uuid::new_v4().simple();
    let key = format!("_health_check_probe:{probe_id}");
    let val = "ping";

    ctx.cache
        .insert(&key, &val)
        .await
        .map_err(|e| format!("写入失败: {e}"))?;
    let res: Option<String> = ctx.cache.get(&key).await.map_err(|e| format!("读取失败: {e}"))?;
    let _ = ctx.cache.remove(&key).await;

    if res.as_deref() != Some(val) {
        return Err("缓存读取值不匹配".to_string());
    }

    Ok(start.elapsed())
}

/// 对象存储 (S3 Garage/Local) 上传与删除探针（动态生成唯一文件名避免并发竞态）
async fn check_storage(ctx: &AppContext) -> std::result::Result<Duration, String> {
    let start = Instant::now();
    let probe_id = uuid::Uuid::new_v4().simple();
    let filename = format!("_health_check_probe_{probe_id}.txt");
    let path = Path::new(&filename);
    let bytes = Bytes::from_static(b"ping");

    ctx.storage
        .upload(path, &bytes)
        .await
        .map_err(|e| format!("上传失败: {e}"))?;
    let _ = ctx.storage.delete(path).await;

    Ok(start.elapsed())
}

/// 格式化输出健康检查探针结果
fn report_status(label: &str, res: std::result::Result<Duration, String>, has_failures: &mut bool) {
    match res {
        Ok(duration) => {
            tracing::info!(service = label, latency_ms = %duration.as_millis(), "[OK] 探针正常");
        }
        Err(err) => {
            *has_failures = true;
            tracing::error!(service = label, error = %err, "[FAIL] 探针异常");
        }
    }
}

#[async_trait]
impl Task for HealthCheck {
    /// 定义 Task 的 CLI 名称与描述信息
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "health:check".to_string(),
            detail: "校验数据库 (PostgreSQL)、缓存 (Redis) 和对象存储 (S3 Garage) 连接健康状态。\n用法示例:\ncargo run task health:check [target:db,cache|only:storage]".to_string(),
        }
    }

    /// 执行健康检查流程
    async fn run(&self, ctx: &AppContext, vars: &task::Vars) -> Result<()> {
        let targets = TargetFilter::from_vars(vars);
        tracing::info!("开始并发执行基础设施健康检查探针...");

        let (db_res, cache_res, storage_res) = tokio::join!(
            async { if targets.db { Some(check_db(ctx).await) } else { None } },
            async {
                if targets.cache {
                    Some(check_cache(ctx).await)
                } else {
                    None
                }
            },
            async {
                if targets.storage {
                    Some(check_storage(ctx).await)
                } else {
                    None
                }
            }
        );

        let mut has_failures = false;
        if let Some(res) = db_res {
            report_status("Database", res, &mut has_failures);
        }
        if let Some(res) = cache_res {
            report_status("Cache", res, &mut has_failures);
        }
        if let Some(res) = storage_res {
            report_status("Storage", res, &mut has_failures);
        }

        if has_failures {
            tracing::error!("部分基础设施健康检查未通过！");
            Err(Error::string("部分基础设施健康检查未通过！"))
        } else {
            tracing::info!("[SUCCESS] 所选基础设施探针检测全数通过！");
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_filter_parse() {
        let mut vars = task::Vars::default();
        assert_eq!(TargetFilter::from_vars(&vars), TargetFilter::all());

        vars.cli.insert("target".to_string(), "db,cache".to_string());
        assert_eq!(
            TargetFilter::from_vars(&vars),
            TargetFilter {
                db: true,
                cache: true,
                storage: false,
            }
        );
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_health_check_task_run() {
        let boot = boot_test::<crate::app::App>().await.unwrap();
        let vars = task::Vars::default();
        let res = HealthCheck.run(&boot.app_context, &vars).await;
        assert!(res.is_ok(), "health_check task failed: {:?}", res);
    }
}
