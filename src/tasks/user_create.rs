//! # 管理员用户创建任务
//!
//! 提供 `cargo loco task user:create` 命令行任务，用于通过命令行直接创建新用户、写入哈希密码并自动发送欢迎与邮箱验证邮件。

use loco_rs::prelude::*;

use crate::{dtos::auth::SignUpEmailRequest, services::auth::SessionService};

/// 用户创建管理员任务
pub struct UserCreate;

#[async_trait]
impl Task for UserCreate {
    /// 定义 Task 的 CLI 名称、用法与参数说明
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "user:create".to_string(),
            detail: "通过命令行创建新用户，写入密码哈弃与账户记录。\n用法:\ncargo run task user:create email:user@example.com name:\"John Doe\" password:\"securepassword\"".to_string(),
        }
    }

    /// 执行用户创建流程
    async fn run(&self, app_context: &AppContext, vars: &task::Vars) -> Result<()> {
        // 步骤 1: 解析命令行参数；若未在 CLI 传入，则自动通过 inquire 弹出交互提示
        let email = match vars.cli_arg("email") {
            Ok(val) => {
                if !val.contains('@') || !val.contains('.') {
                    return Err(Error::string("电子邮箱格式不正确"));
                }
                val.to_string()
            },
            Err(_) => inquire::Text::new("请输入新用户的电子邮箱 (Email):")
                .with_validator(inquire::required!("电子邮箱不能为空"))
                .with_validator(|input: &str| {
                    if input.contains('@') && input.contains('.') {
                        Ok(inquire::validator::Validation::Valid)
                    } else {
                        Ok(inquire::validator::Validation::Invalid(
                            "电子邮箱格式不正确".into(),
                        ))
                    }
                })
                .prompt()
                .map_err(|e| Error::string(&format!("取消或输入失败: {e}")))?,
        };

        let name = match vars.cli_arg("name") {
            Ok(val) => val.to_string(),
            Err(_) => inquire::Text::new("请输入新用户的真实姓名/昵称 (Name):")
                .with_validator(inquire::required!("姓名不能为空"))
                .prompt()
                .map_err(|e| Error::string(&format!("取消或输入失败: {e}")))?,
        };

        let password = match vars.cli_arg("password") {
            Ok(val) => val.to_string(),
            Err(_) => inquire::Password::new("请输入新用户的初始密码 (Password):")
                .with_display_mode(inquire::PasswordDisplayMode::Masked)
                .with_validator(inquire::required!("密码不能为空"))
                .prompt()
                .map_err(|e| Error::string(&format!("取消或输入失败: {e}")))?,
        };

        let sign_up_req = SignUpEmailRequest {
            email: email.clone(),
            password: secrecy::SecretString::from(password),
            name,
            image: None,
            username: None,
        };

        // 步骤 2: 调用 SessionService 进行用户与账户创建
        let res = SessionService::sign_up_email(&app_context.db, &sign_up_req, None, None).await;

        match res {
            Ok(resource) => {
                tracing::info!(
                    user_id = %resource.user.id,
                    email = %resource.user.email,
                    name = %resource.user.name,
                    "✅ 用户创建成功！"
                );
                Ok(())
            },
            Err(err) => {
                tracing::error!(
                    message = %err,
                    user_email = %email,
                    "通过 Task 创建用户失败"
                );
                Err(Error::string(&format!("创建用户失败: {err}")))
            },
        }
    }
}
