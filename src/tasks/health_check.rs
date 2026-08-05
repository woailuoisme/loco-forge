use axum::body::Bytes;
use loco_rs::prelude::*;
use std::path::Path;
use std::time::{Duration, Instant};

pub struct HealthCheck;

#[derive(Debug, Default)]
struct TargetFilter {
    db: bool,
    cache: bool,
    storage: bool,
}

fn parse_targets(vars: &task::Vars) -> TargetFilter {
    let raw = vars
        .cli_arg("target")
        .or_else(|_| vars.cli_arg("--target"))
        .or_else(|_| vars.cli_arg("only"))
        .or_else(|_| vars.cli_arg("--only"))
        .ok();

    let mut filter = TargetFilter::default();

    if let Some(target_str) = raw {
        let cleaned = target_str.trim_start_matches("--target=").trim_start_matches("--only=");

        for item in cleaned.split(',') {
            match item.trim().to_lowercase().as_str() {
                "db" | "database" => filter.db = true,
                "cache" | "redis" => filter.cache = true,
                "storage" | "s3" => filter.storage = true,
                "all" => {
                    filter.db = true;
                    filter.cache = true;
                    filter.storage = true;
                }
                _ => {}
            }
        }
    }

    if !filter.db && !filter.cache && !filter.storage {
        filter.db = true;
        filter.cache = true;
        filter.storage = true;
    }

    filter
}

async fn check_db(ctx: &AppContext) -> std::result::Result<Duration, String> {
    let start = Instant::now();
    ctx.db.ping().await.map_err(|e| e.to_string())?;
    Ok(start.elapsed())
}

async fn check_cache(ctx: &AppContext) -> std::result::Result<Duration, String> {
    let start = Instant::now();
    let test_key = "health_check_test_key";
    let test_val = "ping";

    ctx.cache
        .insert(test_key, &test_val)
        .await
        .map_err(|e| format!("insert failed: {}", e))?;

    let res: Option<String> = ctx
        .cache
        .get(test_key)
        .await
        .map_err(|e| format!("get failed: {}", e))?;

    if res.as_deref() != Some(test_val) {
        return Err(format!("value mismatch: expected '{}', got {:?}", test_val, res));
    }

    let _ = ctx.cache.remove(test_key).await;
    Ok(start.elapsed())
}

async fn check_storage(ctx: &AppContext) -> std::result::Result<Duration, String> {
    let start = Instant::now();
    let test_path = Path::new("health_check_test.txt");
    let test_bytes = Bytes::from_static(b"health_check_content");

    // First try AppContext default storage
    if ctx.storage.upload(test_path, &test_bytes).await.is_ok() {
        let _ = ctx.storage.delete(test_path).await;
        return Ok(start.elapsed());
    }

    // Fallback: build S3 Garage driver from environment variables or defaults
    let bucket = std::env::var("STORAGE_S3_BUCKET").unwrap_or_else(|_| "default".to_string());
    let region = std::env::var("STORAGE_S3_REGION").unwrap_or_else(|_| "garage".to_string());
    let endpoint = std::env::var("STORAGE_S3_ENDPOINT").unwrap_or_else(|_| "http://127.0.0.1:3900".to_string());
    let key_id =
        std::env::var("STORAGE_S3_ACCESS_KEY").unwrap_or_else(|_| "GKfd16f76e81fb861a868a01b9b19452ad".to_string());
    let secret_key = std::env::var("STORAGE_S3_SECRET_KEY")
        .unwrap_or_else(|_| "18d5eda476ff7fb88f0bec38eae3a5e468020ae6dfc610defe2f3f72a65c4339".to_string());

    let credential = loco_rs::storage::drivers::aws::Credential {
        key_id,
        secret_key,
        token: None,
    };

    let driver = loco_rs::storage::drivers::aws::with_credentials_and_endpoint(&bucket, &region, &endpoint, credential)
        .map_err(|e| format!("s3 driver creation failed: {}", e))?;

    let storage = loco_rs::storage::Storage::single(driver);

    storage
        .upload(test_path, &test_bytes)
        .await
        .map_err(|e| format!("upload failed: {}", e))?;

    storage
        .delete(test_path)
        .await
        .map_err(|e| format!("delete failed: {}", e))?;

    Ok(start.elapsed())
}

fn report_status(label: &str, res: std::result::Result<Duration, String>, has_failures: &mut bool) {
    match res {
        Ok(duration) => println!("  [OK]   {:<10} ({:?})", label, duration),
        Err(err) => {
            *has_failures = true;
            println!("  [FAIL] {:<10} - Error: {}", label, err);
        }
    }
}

#[async_trait]
impl Task for HealthCheck {
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "health:check".to_string(),
            detail: "Verify database, cache, and storage connections.\nUsage:\ncargo run task health:check [target:db,cache|only:storage]".to_string(),
        }
    }

    async fn run(&self, ctx: &AppContext, vars: &task::Vars) -> Result<()> {
        let targets = parse_targets(vars);

        println!("\n[INFO] Running Infrastructure Health Checks...\n");
        let mut has_failures = false;

        if targets.db {
            report_status("Database", check_db(ctx).await, &mut has_failures);
        }

        if targets.cache {
            report_status("Cache", check_cache(ctx).await, &mut has_failures);
        }

        if targets.storage {
            report_status("Storage", check_storage(ctx).await, &mut has_failures);
        }

        println!();
        if has_failures {
            Err(Error::string("One or more selected health checks failed."))
        } else {
            println!("[SUCCESS] Selected infrastructure checks completed successfully!\n");
            Ok(())
        }
    }
}
