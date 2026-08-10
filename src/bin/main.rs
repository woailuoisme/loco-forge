use loco_rs::cli;
use locos::app::App;
use migration::Migrator;

#[tokio::main]
async fn main() -> loco_rs::Result<()> {
    // 加载 .env 文件，确保 cargo task / cargo start 都能读取环境变量
    dotenvy::dotenv().ok();

    color_eyre::config::HookBuilder::default()
        .issue_url("https://github.com/woailuoisme/loco-forge/issues")
        .add_default_filters()
        .install()
        .ok();

    cli::main::<App, Migrator>().await
}
