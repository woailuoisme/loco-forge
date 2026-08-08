//! 数据播种与初始环境初始化 Task (Data Seeding Task)
//!
//! # 示例 (Example)
//!
//! 1. 交互式模式 (直接运行，自动弹出操作菜单选择):
//! ```sh
//! cargo loco task seed_data
//! ```
//!
//! 2. 非交互自动化模式 (带参重置并播种数据):
//! ```sh
//! cargo loco task seed_data refresh:true
//! ```

use std::fmt;
use std::path::Path;

use inquire::Select;
use loco_rs::{db, prelude::*};
use migration::Migrator;

use crate::app::App;

/// 播种任务交互选项菜单
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeedMode {
    /// 仅追加数据
    Append,
    /// 重置数据库并重新播种
    ResetAndSeed,
    /// 取消操作
    Cancel,
}

impl fmt::Display for SeedMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Append => write!(f, "仅追加数据 (保留现有数据库表数据直接注入)"),
            Self::ResetAndSeed => write!(f, "重置数据库并重新播种 (清空表并重新执行 Migration)"),
            Self::Cancel => write!(f, "取消操作 (放弃执行数据播种)"),
        }
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct SeedData;

#[async_trait]
impl Task for SeedData {
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "seed_data".to_string(),
            detail: "初始化开发与演示环境的数据播种任务 (支持交互式菜单)".to_string(),
        }
    }

    async fn run(&self, app_context: &AppContext, vars: &task::Vars) -> Result<()> {
        let has_refresh_arg = vars.cli_arg("refresh").is_ok();
        let refresh_val = vars.cli_arg("refresh").is_ok_and(|refresh| refresh == "true");

        // 确定执行模式：若有显式 CLI 参数则直接解析；无参数时调起 inquire Select 动态菜单
        let mode = if has_refresh_arg {
            if refresh_val {
                SeedMode::ResetAndSeed
            } else {
                SeedMode::Append
            }
        } else {
            let options = vec![SeedMode::Append, SeedMode::ResetAndSeed, SeedMode::Cancel];
            Select::new("请选择数据播种执行模式:", options)
                .with_help_message("使用上下方向键选择，回车键确认")
                .prompt()
                .unwrap_or(SeedMode::Cancel)
        };

        match mode {
            SeedMode::Cancel => {
                tracing::info!("[Seed] 操作已取消，未执行数据播种");
                return Ok(());
            }
            SeedMode::ResetAndSeed => {
                tracing::warn!("[Seed] 正在重置并重新执行数据库迁移...");
                db::reset::<Migrator>(&app_context.db).await?;
                tracing::info!("[Seed] 数据库重置完成");
            }
            SeedMode::Append => {
                tracing::info!("[Seed] 以追加模式执行数据播种");
            }
        }

        tracing::info!("[Seed] 开始填充种子数据 (src/fixtures)...");
        let path = Path::new("src/fixtures");
        db::run_app_seed::<App>(app_context, path).await?;
        tracing::info!("[Seed] 种子数据填充完成！");

        Ok(())
    }
}
