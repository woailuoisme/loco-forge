//! # 清理过期 Token 命令行与定时任务
//!
//! 提供 `cargo loco task clean_expired_tokens` 任务，
//! 用于清理数据库中已过期的密码重置 Token 与 Magic Link 登录 Token。

use crate::models::users;
use loco_rs::prelude::*;

/// 清理过期 Token 任务
pub struct CleanExpiredTokens;

#[async_trait]
impl Task for CleanExpiredTokens {
    /// 定义 Task 的 CLI 名称与描述信息
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "clean_expired_tokens".to_string(),
            detail: "定期清理数据库中已过期的重置密码与验证 Token".to_string(),
        }
    }

    /// 执行清理逻辑
    async fn run(&self, app_context: &AppContext, _vars: &task::Vars) -> Result<()> {
        let cleaned_count = users::Model::clean_expired_tokens(&app_context.db).await?;
        println!("Successfully cleaned {cleaned_count} expired tokens.");
        tracing::info!(cleaned_count, "Successfully executed clean_expired_tokens task");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[serial_test::serial]
    async fn test_clean_expired_tokens_task() {
        let boot = boot_test::<crate::app::App>().await.unwrap();
        let task = CleanExpiredTokens;
        let vars = task::Vars::default();
        let res = task.run(&boot.app_context, &vars).await;
        assert!(res.is_ok(), "clean_expired_tokens task failed: {:?}", res);
    }
}
