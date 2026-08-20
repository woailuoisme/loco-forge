//! # 基础设施健康检查任务
//!
//! 提供 `cargo loco task health:check` 命令行任务，用于对数据库 (PostgreSQL)、缓存 (Redis/Valkey) 和对象存储 (S3 Garage) 进行连接与读写探针测试。

use loco_rs::prelude::*;

use crate::common::diagnostics::{ComponentHealth, DiagnosticsHub};

/// 基础设施健康检查任务
pub struct HealthCheck;

/// 格式化输出健康检查探针结果
fn report_status(label: &str, health: &ComponentHealth, has_failures: &mut bool) {
    if health.is_up() {
        tracing::info!(service = label, latency_ms = %health.latency_ms, "[OK] 探针正常");
    } else {
        *has_failures = true;
        tracing::error!(
            service = label,
            error = %health.error.as_deref().unwrap_or("unknown error"),
            "[FAIL] 探针异常"
        );
    }
}

#[async_trait]
impl Task for HealthCheck {
    /// 定义 Task 的 CLI 名称与描述信息
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "health:check".to_string(),
            detail: "校验数据库、缓存与对象存储连接健康状态。\n用法示例:\ncargo run task health:check [target:db,cache,storage]".to_string(),
        }
    }

    /// 执行健康检查流程
    async fn run(&self, ctx: &AppContext, vars: &task::Vars) -> Result<()> {
        let target = vars
            .cli_arg("target")
            .or_else(|_| vars.cli_arg("--target"))
            .unwrap_or("all")
            .to_lowercase();

        let check_db = target == "all" || target.contains("db");
        let check_cache = target == "all" || target.contains("cache") || target.contains("redis");
        let check_storage = target == "all" || target.contains("storage") || target.contains("s3");

        tracing::info!("开始并发执行基础设施健康检查探针...");

        let (db_res, cache_res, storage_res) = tokio::join!(
            async {
                if check_db {
                    Some(DiagnosticsHub::check_db(ctx).await)
                } else {
                    None
                }
            },
            async {
                if check_cache {
                    Some(DiagnosticsHub::check_cache(ctx).await)
                } else {
                    None
                }
            },
            async {
                if check_storage {
                    Some(DiagnosticsHub::check_storage(ctx).await)
                } else {
                    None
                }
            }
        );

        let mut has_failures = false;
        if let Some(ref res) = db_res {
            report_status("Database", res, &mut has_failures);
        }
        if let Some(ref res) = cache_res {
            report_status("Cache", res, &mut has_failures);
        }
        if let Some(ref res) = storage_res {
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

    #[tokio::test]
    #[serial_test::serial]
    async fn test_health_check_task_run() {
        let boot = boot_test::<crate::app::App>().await.unwrap();
        let vars = task::Vars::default();
        let res = HealthCheck.run(&boot.app_context, &vars).await;
        assert!(res.is_ok(), "health_check task failed: {res:?}");
    }
}
