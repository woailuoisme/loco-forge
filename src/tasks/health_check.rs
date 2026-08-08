//! # 基础设施健康检查任务
//!
//! 提供 `cargo loco task health:check` 命令行任务，用于对数据库 (PostgreSQL)、缓存 (Redis/Valkey) 和对象存储 (S3 Garage) 进行连接与读写探针测试。

use axum::body::Bytes;
use loco_rs::prelude::*;
use std::path::Path;
use std::time::{Duration, Instant};

/// 基础设施健康检查任务
pub struct HealthCheck;

/// 健康检查目标过滤器
#[derive(Debug, Default)]
struct TargetFilter {
    db: bool,
    cache: bool,
    storage: bool,
}

/// 解析命令行参数中的目标服务筛选条件
fn parse_targets(vars: &task::Vars) -> TargetFilter {
    let raw = vars
        .cli_arg("target")
        .or_else(|_| vars.cli_arg("--target"))
        .or_else(|_| vars.cli_arg("only"))
        .or_else(|_| vars.cli_arg("--only"))
        .ok();

    let mut filter = TargetFilter::default();

    if let Some(target_str) = raw {
        let cleaned = target_str.trim_start_matches("--target=").trim_start_matches("--only=");

        for item in cleaned.split(',') {
            match item.trim().to_lowercase().as_str() {
                "db" | "database" => filter.db = true,
                "cache" | "redis" => filter.cache = true,
                "storage" | "s3" => filter.storage = true,
                "all" => {
                    filter.db = true;
                    filter.cache = true;
                    filter.storage = true;
                }
                _ => {}
            }
        }
    }

    if !filter.db && !filter.cache && !filter.storage {
        filter.db = true;
        filter.cache = true;
        filter.storage = true;
    }

    filter
}

/// 步骤子函数: 执行数据库 (PostgreSQL) 连接探针检测
async fn check_db(ctx: &AppContext) -> std::result::Result<Duration, String> {
    let start = Instant::now();
    ctx.db.ping().await.map_err(|e| e.to_string())?;
    Ok(start.elapsed())
}

/// 步骤子函数: 执行缓存 (Redis/Valkey) 写入、读取与清理探针检测
async fn check_cache(ctx: &AppContext) -> std::result::Result<Duration, String> {
    let start = Instant::now();
    let test_key = "health_check_test_key";
    let test_val = "ping";

    // 1. 测试缓存写入
    ctx.cache
        .insert(test_key, &test_val)
        .await
        .map_err(|e| format!("写入失败: {}", e))?;

    // 2. 测试缓存读取与值匹配
    let res: Option<String> = ctx.cache.get(test_key).await.map_err(|e| format!("读取失败: {}", e))?;

    if res.as_deref() != Some(test_val) {
        return Err(format!("缓存值校验不匹配: 期望 '{}', 实际得到 {:?}", test_val, res));
    }

    // 3. 测试后清理探针键
    let _ = ctx.cache.remove(test_key).await;
    Ok(start.elapsed())
}

/// 步骤子函数: 执行对象存储 (S3 Garage) 上传与删除探针检测
async fn check_storage(ctx: &AppContext) -> std::result::Result<Duration, String> {
    let start = Instant::now();
    let test_path = Path::new("health_check_test.txt");
    let test_bytes = Bytes::from_static(b"health_check_content");

    // 1. 优先使用 AppContext 默认对象存储驱动测试
    if ctx.storage.upload(test_path, &test_bytes).await.is_ok() {
        let _ = ctx.storage.delete(test_path).await;
        return Ok(start.elapsed());
    }

    // 2. 降级备用: 从环境变量构建 S3 Storage 驱动进行二次探针检测
    let bucket = std::env::var("STORAGE_S3_BUCKET").unwrap_or_else(|_| "default".to_string());
    let region = std::env::var("STORAGE_S3_REGION").unwrap_or_else(|_| "garage".to_string());
    let endpoint = std::env::var("STORAGE_S3_ENDPOINT").unwrap_or_else(|_| "http://127.0.0.1:3900".to_string());
    let key_id = std::env::var("STORAGE_S3_ACCESS_KEY")
        .map_err(|_| "S3 存储密钥未配置 (需设置 STORAGE_S3_ACCESS_KEY 环境变量)".to_string())?;
    let secret_key = std::env::var("STORAGE_S3_SECRET_KEY")
        .map_err(|_| "S3 存储密钥未配置 (需设置 STORAGE_S3_SECRET_KEY 环境变量)".to_string())?;

    let credential = loco_rs::storage::drivers::aws::Credential {
        key_id,
        secret_key,
        token: None,
    };

    let driver = loco_rs::storage::drivers::aws::with_credentials_and_endpoint(&bucket, &region, &endpoint, credential)
        .map_err(|e| format!("S3 驱动创建失败: {}", e))?;

    let storage = loco_rs::storage::Storage::single(driver);

    storage
        .upload(test_path, &test_bytes)
        .await
        .map_err(|e| format!("上传文件失败: {}", e))?;

    storage
        .delete(test_path)
        .await
        .map_err(|e| format!("删除测试文件失败: {}", e))?;

    Ok(start.elapsed())
}

/// 格式化输出健康检查结果状态
fn report_status(label: &str, res: std::result::Result<Duration, String>, has_failures: &mut bool) {
    match res {
        Ok(duration) => tracing::info!(service = label, duration = ?duration, "[OK] 探针正常"),
        Err(err) => {
            *has_failures = true;
            tracing::error!(service = label, error = %err, "[FAIL] 探针异常");
        }
    }
}

#[async_trait]
impl Task for HealthCheck {
    /// 定义 Task 的 CLI 名称与帮助说明
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "health:check".to_string(),
            detail: "校验数据库 (PostgreSQL)、缓存 (Redis) 和对象存储 (S3 Garage) 连接健康状态。\n用法示例:\ncargo run task health:check [target:db,cache|only:storage]".to_string(),
        }
    }

    /// 执行健康检查流程
    async fn run(&self, ctx: &AppContext, vars: &task::Vars) -> Result<()> {
        // 步骤 1: 解析命令行参数中的检测目标服务
        let targets = parse_targets(vars);

        tracing::info!("开始执行基础设施健康检查探针...");
        let mut has_failures = false;

        // 步骤 2: 检测数据库状态
        if targets.db {
            report_status("Database", check_db(ctx).await, &mut has_failures);
        }

        // 步骤 3: 检测缓存状态
        if targets.cache {
            report_status("Cache", check_cache(ctx).await, &mut has_failures);
        }

        // 步骤 4: 检测对象存储状态
        if targets.storage {
            report_status("Storage", check_storage(ctx).await, &mut has_failures);
        }

        // 步骤 5: 根据是否存在失败项返回执行结果
        if has_failures {
            tracing::error!("部分基础设施健康检查未通过！");
            Err(Error::string("部分基础设施健康检查未通过！"))
        } else {
            tracing::info!("[SUCCESS] 所选基础设施探针检测全数通过！");
            Ok(())
        }
    }
}
