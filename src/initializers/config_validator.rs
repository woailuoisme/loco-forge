use async_trait::async_trait;
use loco_rs::{
    Error, Result,
    app::{AppContext, Initializer},
};

pub struct ConfigValidatorInitializer;

#[async_trait]
impl Initializer for ConfigValidatorInitializer {
    fn name(&self) -> String {
        "config-validator".to_string()
    }

    async fn before_run(&self, ctx: &AppContext) -> Result<()> {
        // 在 test 环境下跳过配置强校验，方便 CI/CD 并行单元测试
        if ctx.environment == loco_rs::environment::Environment::Test {
            return Ok(());
        }

        // 1. 强校验 JWT 配置是否存在、非空且长度 >= 32 字符
        let jwt_config = ctx
            .config
            .get_jwt_config()
            .map_err(|e| Error::string(&format!("配置校验失败：无法获取 JWT 配置: {e}")))?;

        let secret = jwt_config.secret.trim();
        if secret.is_empty() {
            return Err(Error::string(
                "配置校验失败：auth.jwt.secret (JWT_SECRET) 不能为空",
            ));
        }
        if secret.len() < 32 {
            return Err(Error::string(&format!(
                "配置校验失败：auth.jwt.secret 长度必须 ≥ 32 字符 (当前长度: {})，请在配置文件或环境变量中设置安全的随机密钥",
                secret.len()
            )));
        }

        // 2. 强校验数据库连接 URI 存在性与协议格式
        let db_uri = ctx.config.database.uri.trim();
        if db_uri.is_empty() {
            return Err(Error::string(
                "配置校验失败：database.uri (DATABASE_URL) 不能为空",
            ));
        }

        let is_valid_scheme = db_uri.starts_with("postgres://")
            || db_uri.starts_with("postgresql://")
            || db_uri.starts_with("mysql://")
            || db_uri.starts_with("sqlite://");

        if !is_valid_scheme {
            return Err(Error::string(&format!(
                "配置校验失败：database.uri 连接串格式无效 ('{db_uri}')，仅支持 postgres://, mysql:// 或 sqlite:// 前缀"
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initializer_name() {
        let init = ConfigValidatorInitializer;
        assert_eq!(init.name(), "config-validator");
    }
}
