//! # JWT Secret Key Generation Task
//!
//! Provides the `cargo loco task jwt:secret` CLI task for generating a strong random JWT key and
//! automatically writing it to the `.env` configuration file.

use loco_rs::prelude::*;
use std::fs;

/// Generates a 64-char strong JWT secret (concatenated dual UUID v4 hex) and writes it to `.env`.
///
/// Scans the `.env` file for an existing `JWT_SECRET=` line and replaces it,
/// or appends to the end of the file if not found.
/// Returns the generated secret string.
///
/// # Errors
///
/// Returns `loco_rs::Error` if file writing fails.
pub fn generate_and_persist_jwt_secret() -> Result<String> {
    // Step 1: Generate a 64-char cryptographically strong key by concatenating dual UUID v4 hex
    let new_secret = format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    );

    tracing::info!(secret = %new_secret, "Generated a random JWT_SECRET key");

    // Step 2: Read the existing .env config file
    let env_path = ".env";
    let content = fs::read_to_string(env_path).unwrap_or_default();

    // Step 3: Scan line by line and replace or append the JWT_SECRET assignment
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

    // Step 4: Write the updated content back to the .env file
    fs::write(env_path, &updated_content).map_err(|e| {
        tracing::error!(error = %e, "Failed to write .env file");
        Error::string(&format!("Failed to write .env file: {e}"))
    })?;

    tracing::info!("Successfully synced JWT_SECRET in .env!");
    Ok(new_secret)
}

/// JWT Secret generation and update task
pub struct JwtSecret;

#[async_trait]
impl Task for JwtSecret {
    /// Defines the CLI name and description for this task
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "jwt:secret".to_string(),
            detail: "Generate a strong cryptographic JWT_SECRET key and auto-update the .env file"
                .to_string(),
        }
    }

    /// Executes the generation logic and file update
    async fn run(&self, _app_context: &AppContext, _vars: &task::Vars) -> Result<()> {
        generate_and_persist_jwt_secret().map(|_| ())
    }
}
