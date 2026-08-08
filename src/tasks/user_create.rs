//! # 管理员用户创建任务
//!
//! 提供 `cargo loco task user:create` 命令行任务，用于通过命令行直接创建新用户、写入哈希密码并自动发送欢迎与邮箱验证邮件。

use loco_rs::prelude::*;

use crate::{
    mailers::auth::AuthMailer,
    models::{_entities::users, users::RegisterRequest},
};

/// 用户创建管理员任务
pub struct UserCreate;

#[async_trait]
impl Task for UserCreate {
    /// 定义 Task 的 CLI 名称、用法与参数说明
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "user:create".to_string(),
            detail: "通过命令行创建新用户，写入密码哈希，并自动发送欢迎/邮箱验证邮件。\n用法:\ncargo run task user:create email:user@example.com name:\"John Doe\" password:\"securepassword\"".to_string(),
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
            }
            Err(_) => inquire::Text::new("请输入新用户的电子邮箱 (Email):")
                .with_validator(inquire::required!("电子邮箱不能为空"))
                .with_validator(|input: &str| {
                    if input.contains('@') && input.contains('.') {
                        Ok(inquire::validator::Validation::Valid)
                    } else {
                        Ok(inquire::validator::Validation::Invalid("电子邮箱格式不正确".into()))
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

        let register_params = RegisterRequest {
            email,
            password: secrecy::SecretString::from(password),
            name,
        };

        // 步骤 2: 调用 User Model 的 create_with_password 进行密码哈希与记录插入
        let res = users::Model::create_with_password(&app_context.db, &register_params).await;

        let user = match res {
            Ok(user) => {
                tracing::info!(
                    message = "User created successfully",
                    user_email = &register_params.email,
                    user_pid = user.pid.to_string(),
                    "通过 Task 成功创建用户"
                );
                user
            }
            Err(err) => {
                tracing::error!(
                    message = err.to_string(),
                    user_email = &register_params.email,
                    "通过 Task 创建用户失败"
                );
                return Err(Error::string(&format!("创建用户失败: {err}")));
            }
        };

        // 步骤 3: 生成邮箱验证 Token 并标记验证邮件发送状态
        let user = user
            .into_active_model()
            .set_email_verification_sent(&app_context.db)
            .await
            .map_err(|err| {
                tracing::error!(
                    message = err.to_string(),
                    user_email = &register_params.email,
                    "设置邮箱验证标记失败"
                );
                Error::string("设置邮箱验证失败")
            })?;

        // 步骤 4: 发送注册欢迎与验证邮件
        AuthMailer::send_welcome(app_context, &user).await.map_err(|err| {
            tracing::error!(
                message = err.to_string(),
                user_email = &register_params.email,
                "发送欢迎邮件失败"
            );
            Error::string("发送欢迎邮件失败")
        })?;

        tracing::info!(
            message = "User creation task completed successfully",
            user_email = &register_params.email,
            user_pid = user.pid.to_string(),
            "用户创建 Task 执行完毕"
        );

        // 步骤 5: 终端控制台记录创建成功明细
        tracing::info!(
            email = %user.email,
            name = %user.name,
            pid = %user.pid,
            "✅ 用户创建成功！"
        );

        Ok(())
    }
}
