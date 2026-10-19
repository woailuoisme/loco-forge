//! `app init` CLI：在 App boot 之前完成项目初始化（建库 + 迁移）。
//!
//! boot 阶段就会连接目标数据库，库不存在时任何 Loco Task 都跑不起来，
//! 所以这一步只能做在 CLI 入口，不能注册成 Task。
//!
//! 用法：
//! ```sh
//! cargo loco init                  # 默认 development
//! cargo loco init -e production
//! ```

use loco_rs::app::Hooks;
use loco_rs::db;
use loco_rs::environment::Environment;
use loco_rs::prelude::*;
use migration::Migrator;

use crate::app::App;

/// 初始化数据库：不存在则建库，随后执行全部待应用迁移。
pub async fn run(env: &str) -> Result<()> {
    // 本命令早于 loco 的 logger 初始化，装一个最小 subscriber 让日志可见
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .try_init();

    let environment = Environment::from(env.to_string());
    let config = App::load_config(&environment).await?;

    // `db::create` 在库已存在时同样返回错误，用"能否连上目标库"区分真实失败
    if let Err(create_err) = db::create(&config.database.uri).await {
        if db::connect(&config.database).await.is_err() {
            return Err(create_err);
        }
        tracing::info!(environment = env, "数据库已存在，跳过创建");
    }

    let conn = db::connect(&config.database).await?;
    db::migrate::<Migrator>(&conn).await?;

    tracing::info!(environment = env, "初始化完成");
    Ok(())
}
