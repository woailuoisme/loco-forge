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

/// 从环境变量按优先级读取配置
fn get_env(primary: &str, fallback: &str) -> Option<String> {
    std::env::var(primary).or_else(|_| std::env::var(fallback)).ok()
}

/// 初始化全局对象存储驱动并更新 AppContext
pub async fn init(mut ctx: AppContext) -> Result<AppContext> {
    let storage = match ctx.environment {
        // 1. 测试环境：使用 In-Memory 内存驱动，零磁盘与网络依赖
        Environment::Test => Storage::single(mem::new()),
        // 2. 开发与生产环境：从环境变量动态读取 S3 凭据与 Endpoint
        _ => {
            let bucket = get_env("STORAGE_S3_BUCKET", "AWS_S3_BUCKET").unwrap_or_else(|| "default".to_string());
            let region = get_env("STORAGE_S3_REGION", "AWS_REGION").unwrap_or_else(|| "garage".to_string());

            let key_id = get_env("STORAGE_S3_ACCESS_KEY", "AWS_ACCESS_KEY_ID");
            let secret_key = get_env("STORAGE_S3_SECRET_KEY", "AWS_SECRET_ACCESS_KEY");
            let endpoint = std::env::var("STORAGE_S3_ENDPOINT").ok();

            let cred = key_id.zip(secret_key).map(|(key_id, secret_key)| Credential {
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
                }
                None => match cred {
                    Some(cred) => aws::with_credentials(&bucket, &region, cred)?,
                    None => aws::new(&bucket, &region)?,
                },
            };

            Storage::single(store)
        }
    };

    ctx.storage = storage.into();
    Ok(ctx)
}
