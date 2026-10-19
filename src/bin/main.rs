use loco_rs::cli;
use locos::app::App;
use migration::Migrator;

// 生产级全局内存分配器 (jemalloc)
// 1. 核心作用：替代系统默认的 malloc/ptmalloc 分配器。在高并发异步场景（Tokio 多线程、高频 JSON 序列化、数据库连接池）
//    下显著降低多线程锁竞争与内存碎片化，大幅提升 Web 服务的吞吐性能（吞吐提升约 10%~20% 并降低长尾延迟）。
// 2. 平台条件编译 #[cfg(target_os = "linux")]：
//    jemalloc 的收益在 Linux (Glibc/Musl) 生产环境最为显著；macOS 本地开发改用系统 malloc，
//    以避免编译整个 jemalloc C 库带来的构建开销（依赖声明同步见 Cargo.toml）。
#[cfg(target_os = "linux")]
use tikv_jemallocator::Jemalloc;

#[cfg(target_os = "linux")]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[tokio::main]
async fn main() -> loco_rs::Result<()> {
    // 加载 .env 文件，确保 cargo task / cargo start 都能读取环境变量
    dotenvy::dotenv().ok();

    // loco 以 default-features = false 引入 opendal，未安装默认 HTTP transport，
    // S3 存储会报 "default HTTP transport is not installed"。幂等，可安全重复调用。
    opendal::install_default();

    color_eyre::config::HookBuilder::default()
        .issue_url("https://github.com/woailuoisme/loco-forge/issues")
        .add_default_filters()
        .install()
        .ok();

    // `app init` 必须早于 loco CLI 分发：boot 会连接目标库，库不存在时直接失败
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let is_init = matches!(argv.first().map(String::as_str), Some("init"))
        || (argv.first().map(String::as_str) == Some("app")
            && argv.get(1).map(String::as_str) == Some("init"));
    if is_init {
        let env = argv
            .iter()
            .position(|a| a == "-e" || a == "--environment")
            .and_then(|i| argv.get(i + 1))
            .map_or("development", String::as_str);
        return locos::tasks::app_init::run(env).await;
    }

    cli::main::<App, Migrator>().await
}
