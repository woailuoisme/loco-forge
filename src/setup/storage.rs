//! # 对象存储 (Storage) 基础设施装配模块
//!
//! 在 `AppContext` 构建阶段 (`after_context`)，根据当前运行环境 (`Environment`) 与环境变量
//! 动态初始化对象存储驱动 (`ctx.storage`)：
//! - `Test` 环境：装配 `In-Memory` 内存存储驱动，隔离网络与磁盘依赖。
//! - `Development` / `Production` 环境：按规范从环境变量读取 S3 / AWS 凭据装配 S3 Garage 驱动。

use loco_rs::{
    Result,
    app::AppContext,
    environment::Environment,
    storage::{
        Storage,
        drivers::{
            aws::{self, Credential},
            mem,
        },
    },
};

/// 从环境变量列表按顺序读取首个有效的配置值
fn get_env_first(keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| std::env::var(key).ok())
}

/// 初始化全局对象存储驱动并更新 AppContext
pub async fn init(mut ctx: AppContext) -> Result<AppContext> {
    let storage = match ctx.environment {
        // 1. 测试环境：使用 In-Memory 内存驱动，零磁盘与网络依赖
        Environment::Test => {
            tracing::info!(driver = "memory", "初始化测试环境内存对象存储驱动");
            Storage::single(mem::new())
        },
        // 2. 开发与生产环境：从环境变量动态读取 S3 凭据与 Endpoint
        _ => {
            let bucket = std::env::var("AWS_BUCKET").unwrap_or_else(|_| "default".to_string());
            let region =
                std::env::var("AWS_DEFAULT_REGION").unwrap_or_else(|_| "garage".to_string());

            let key_id = std::env::var("AWS_ACCESS_KEY_ID").ok();
            let secret_key = std::env::var("AWS_SECRET_ACCESS_KEY").ok();
            let endpoint = get_env_first(&["AWS_ENDPOINT", "AWS_ENDPOINT_URL_S3"]);

            let cred = key_id
                .zip(secret_key)
                .map(|(key_id, secret_key)| Credential {
                    key_id,
                    secret_key,
                    token: std::env::var("AWS_SESSION_TOKEN").ok(),
                });

            let store = match endpoint {
                Some(endpoint) => {
                    let cred = cred.unwrap_or_else(|| Credential {
                        key_id: "garage_key".to_string(),
                        secret_key: "garage_secret".to_string(),
                        token: None,
                    });
                    aws::with_credentials_and_endpoint(&bucket, &region, &endpoint, cred)?
                },
                None => match cred {
                    Some(cred) => aws::with_credentials(&bucket, &region, cred)?,
                    None => aws::new(&bucket, &region)?,
                },
            };

            tracing::info!(
                driver = "s3",
                bucket = %bucket,
                region = %region,
                "初始化 S3 对象存储驱动"
            );
            Storage::single(store)
        },
    };

    ctx.storage = storage.into();
    Ok(ctx)
}
