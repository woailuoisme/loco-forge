use loco_rs::cli;
use locos::app::App;
use migration::Migrator;

// 生产级全局内存分配器 (jemalloc)
// 1. 核心作用：替代系统默认的 malloc/ptmalloc 分配器。在高并发异步场景（Tokio 多线程、高频 JSON 序列化、数据库连接池）
//    下显著降低多线程锁竞争与内存碎片化，大幅提升 Web 服务的吞吐性能（吞吐提升约 10%~20% 并降低长尾延迟）。
// 2. 平台条件编译 #[cfg(not(target_env = "msvc"))]：
//    jemalloc 原生深度适配 Linux (Glibc/Musl) 和 macOS (Unix 类系统)；在 Windows MSVC 平台下自动回退至默认系统分配器以保证最大跨平台兼容性。
#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

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
