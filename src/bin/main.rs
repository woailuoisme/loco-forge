use loco_rs::cli;
use locos::app::App;
use migration::Migrator;

#[tokio::main]
async fn main() -> loco_rs::Result<()> {
    color_eyre::config::HookBuilder::default()
        .issue_url("https://github.com/woailuoisme/loco-forge/issues")
        .add_default_filters()
        .install()
        .ok();

    cli::main::<App, Migrator>().await
}
