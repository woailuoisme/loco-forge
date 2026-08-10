use crate::tasks::jwt_secret::generate_and_persist_jwt_secret;
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
        // Skip strict config validation in test environment for CI/CD parallel unit tests
        if ctx.environment == loco_rs::environment::Environment::Test {
            return Ok(());
        }

        // 1. Validate JWT config: exists, non-empty, and length >= 32 chars
        let jwt_config = ctx.config.get_jwt_config().map_err(|e| {
            Error::string(&format!(
                "Config validation failed: unable to get JWT config: {e}"
            ))
        })?;

        let secret = jwt_config.secret.trim();
        if secret.is_empty() {
            return Err(Error::string(
                "Config validation failed: auth.jwt.secret (JWT_SECRET) must not be empty",
            ));
        }
        if secret.len() < 32 {
            // Auto-generate a strong key and write back to .env, avoiding manual jwt:secret run
            let auto_gen_msg = match generate_and_persist_jwt_secret() {
                Ok(new_secret) => {
                    tracing::warn!(
                        old_len = secret.len(),
                        new_len = new_secret.len(),
                        "JWT_SECRET too short, auto-generated a new key and wrote it to .env"
                    );
                    format!(
                        "A strong JWT_SECRET key ({} chars) has been auto-generated and written to .env, please restart the application",
                        new_secret.len()
                    )
                },
                Err(e) => {
                    tracing::error!(error = %e, "Auto-generation of JWT_SECRET failed");
                    format!(
                        "Auto-generation of secret failed ({e}), please run `cargo loco task jwt:secret` manually"
                    )
                },
            };

            return Err(Error::string(&format!(
                "Config validation failed: auth.jwt.secret must be at least 32 chars (current length: {}). {auto_gen_msg}",
                secret.len()
            )));
        }

        // 2. Validate database connection URI: exists and has a valid protocol prefix
        let db_uri = ctx.config.database.uri.trim();
        if db_uri.is_empty() {
            return Err(Error::string(
                "Config validation failed: database.uri (DATABASE_URL) must not be empty",
            ));
        }

        let is_valid_scheme = db_uri.starts_with("postgres://")
            || db_uri.starts_with("postgresql://")
            || db_uri.starts_with("mysql://")
            || db_uri.starts_with("sqlite://");

        if !is_valid_scheme {
            return Err(Error::string(&format!(
                "Config validation failed: invalid database.uri scheme ('{db_uri}'), only postgres://, mysql://, or sqlite:// prefixes are supported"
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
