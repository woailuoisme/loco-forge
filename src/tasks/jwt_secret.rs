use loco_rs::prelude::*;
use std::fs;

pub struct JwtSecret;

#[async_trait]
impl Task for JwtSecret {
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "jwt:secret".to_string(),
            detail: "Generates a cryptographically strong JWT_SECRET and updates .env file".to_string(),
        }
    }

    async fn run(&self, _app_context: &AppContext, _vars: &task::Vars) -> Result<()> {
        let new_secret = format!("{}{}", uuid::Uuid::new_v4().simple(), uuid::Uuid::new_v4().simple());

        println!("Generated Strong JWT_SECRET: {}", new_secret);

        if let Ok(content) = fs::read_to_string(".env") {
            let updated_content = content
                .lines()
                .map(|line| {
                    if line.starts_with("JWT_SECRET=") {
                        format!("JWT_SECRET={}", new_secret)
                    } else {
                        line.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");

            if fs::write(".env", format!("{}\n", updated_content.trim())).is_ok() {
                println!("Successfully updated JWT_SECRET in .env file!");
            }
        }
        Ok(())
    }
}
