//! # 应用核心入口与 Hooks 接口
//!
//! 该模块定义了 Loco 全栈应用的入口结构体 `App`，实现 `Hooks` Trait。
//! 所有具体的路由、中间件、Task、Worker 注册逻辑均已委托给 [`crate::setup`] 子模块办理。

use async_trait::async_trait;
use loco_rs::{
    Result,
    app::{AppContext, Hooks, Initializer},
    bgworker::Queue,
    boot::{BootResult, StartMode, create_app},
    config::Config,
    controller::AppRoutes,
    db::{self, truncate_table},
    environment::Environment,
    task::Tasks,
};
use migration::Migrator;
use std::path::Path;

use crate::{
    models::_entities::{posts, users},
    setup,
};

/// Loco 应用主结构体
pub struct App;

#[async_trait]
impl Hooks for App {
    /// 返回当前应用版本号与 Git Hash 标识
    fn app_version() -> String {
        format!(
            "{} ({})",
            env!("CARGO_PKG_VERSION"),
            option_env!("BUILD_SHA")
                .or(option_env!("GITHUB_SHA"))
                .unwrap_or("dev")
        )
    }

    /// 返回应用 Crate 名称
    fn app_name() -> &'static str {
        env!("CARGO_CRATE_NAME")
    }

    /// 引导启动底层网络服务与数据库迁移驱动
    async fn boot(
        mode: StartMode,
        environment: &Environment,
        config: Config,
    ) -> Result<BootResult> {
        create_app::<Self, Migrator>(mode, environment, config).await
    }

    /// 在 AppContext 构建完成后，装配全局对象存储驱动 (委托给 `setup::storage::init`)
    async fn after_context(ctx: AppContext) -> Result<AppContext> {
        setup::storage::init(ctx).await
    }

    /// 绑定全局 HTTP 中间件与请求日志钩子 (委托给 `setup::middleware::register`)
    async fn after_routes(router: axum::Router, ctx: &AppContext) -> Result<axum::Router> {
        setup::middleware::register(router, ctx).await
    }

    /// 加载系统 Initializer 插件 (例如 OpenAPI 接口文档、配置校验、日志系统)
    async fn initializers(_ctx: &AppContext) -> Result<Vec<Box<dyn Initializer>>> {
        Ok(vec![
            Box::new(crate::initializers::config_validator::ConfigValidatorInitializer),
            Box::new(crate::initializers::openapi::OpenApiInitializer),
        ])
    }

    /// 绑定并返回全局控制器路由 (委托给 `setup::routes`)
    fn routes(ctx: &AppContext) -> AppRoutes {
        setup::routes::routes(ctx)
    }

    /// 注册后台异步任务 Worker (委托给 `setup::workers`)
    async fn connect_workers(ctx: &AppContext, queue: &Queue) -> Result<()> {
        setup::workers::connect_workers(ctx, queue).await
    }

    /// 注册命令行 Task 列表 (委托给 `setup::tasks`)
    fn register_tasks(tasks: &mut Tasks) {
        setup::tasks::register_tasks(tasks);
    }

    /// 清空数据库数据 (主要用于测试数据重置)
    async fn truncate(ctx: &AppContext) -> Result<()> {
        // 步骤 1: 截断/清空数据库关联表
        truncate_table(&ctx.db, users::Entity).await?;
        truncate_table(&ctx.db, posts::Entity).await?;
        Ok(())
    }

    /// 执行基础 Seed 种子数据注入
    async fn seed(ctx: &AppContext, base: &Path) -> Result<()> {
        // 步骤 1: 读取 users.yaml 并注入初始用户记录
        db::seed::<users::ActiveModel>(&ctx.db, &base.join("users.yaml").display().to_string())
            .await?;
        Ok(())
    }
}
