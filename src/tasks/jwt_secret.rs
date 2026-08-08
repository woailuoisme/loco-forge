//! # JWT Secret 秘钥生成任务
//!
//! 提供 `cargo loco task jwt:secret` 命令行任务，用于生成高强度的随机 JWT 密钥并自动写入 `.env` 配置文件。

use loco_rs::prelude::*;
use std::fs;

/// JWT Secret 生成与更新任务
pub struct JwtSecret;

#[async_trait]
impl Task for JwtSecret {
    /// 定义 Task 的 CLI 名称与描述信息
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "jwt:secret".to_string(),
            detail: "生成高强度的密码学 JWT_SECRET 密钥并自动更新 .env 文件".to_string(),
        }
    }

    /// 执行生成逻辑与文件更新
    async fn run(&self, _app_context: &AppContext, _vars: &task::Vars) -> Result<()> {
        // 步骤 1: 使用双 UUID v4 拼接生成 64 字符密码学高强度密钥
        let new_secret = format!("{}{}", uuid::Uuid::new_v4().simple(), uuid::Uuid::new_v4().simple());

        tracing::info!(secret = %new_secret, "已生成随机 JWT_SECRET 密钥");

        // 步骤 2: 读取现有的 .env 配置文件
        let env_path = ".env";
        let content = fs::read_to_string(env_path).unwrap_or_default();

        // 步骤 3: 逐行扫描并替换或追加 JWT_SECRET 赋值项
        let mut found = false;
        let mut updated_lines = Vec::new();

        for line in content.lines() {
            if line.starts_with("JWT_SECRET=") {
                updated_lines.push(format!("JWT_SECRET={new_secret}"));
                found = true;
            } else {
                updated_lines.push(line.to_string());
            }
        }

        if !found {
            updated_lines.push(format!("JWT_SECRET={new_secret}"));
        }

        let updated_content = format!("{}\n", updated_lines.join("\n").trim());

        // 步骤 4: 写回更新后的内容至 .env 文件
        fs::write(env_path, updated_content).map_err(|e| {
            tracing::error!(error = %e, "写入 .env 文件失败");
            Error::string(&format!("写入 .env 文件失败: {e}"))
        })?;

        tracing::info!("已成功同步更新 .env 中的 JWT_SECRET 配置！");
        Ok(())
    }
}
