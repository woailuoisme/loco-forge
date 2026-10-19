# 自动加载 .env
set dotenv-load := true

# 检测到 sccache 时自动启用编译缓存，未安装则平滑降级
export RUSTC_WRAPPER := `which sccache 2>/dev/null || echo ""`

# 列出所有可用命令
default:
    @just --list

# 释放 Web 服务端口 (读取 .env 的 APP_PORT，默认 5150)
kill-port port=env('APP_PORT', '5150'):
    @lsof -ti:{{port}} | xargs kill -9 2>/dev/null || true

# ==============================================================================
# 1. 开发与运行 (Development & Execution)
# ==============================================================================

# 启动开发热重载服务 (监听 src/ 与 config/)
dev: kill-port
    cargo loco watch

# 并行启动 Server、Worker 与 Scheduler (均带热重载)
dev-all: kill-port
    cargo loco watch --server-and-worker --scheduler
# 启动 Loco Web 服务 (Debug 模式)
start: kill-port
    cargo loco start

# 单进程同时启动 Server、Worker 与 Scheduler
start-all:
    cargo loco start --all

# 编译 Debug 二进制
build:
    cargo build

# 编译 Release 二进制
build-prod:
    cargo build --release

# 以 Release 模式启动 Loco Web 服务
prod: kill-port
    cargo run --release -- start

# 启动后台 Job 消费进程
worker:
    cargo loco start --worker

# 启动定时任务调度服务
scheduler:
    cargo loco scheduler

# 兼容旧别名
alias build-release := build-prod
alias start-release := prod

# ==============================================================================
# 2. 数据库与 Task (Database & Loco Tasks)
# ==============================================================================

# 项目初始化：建库 → 跑迁移 → 可选灌种子数据 (首次拉起本项目的唯一入口)
# 建库逻辑在 src/tasks/app_init.rs，必须早于 App boot，因此走自定义 CLI 而非 Loco Task
init env="development" seed="false":
    cargo loco init -e {{env}}
    @if [ "{{seed}}" = "true" ]; then cargo loco task seed_data refresh:true -e {{env}}; fi

# 运行数据库迁移
migrate:
    cargo loco db migrate

# 填充种子数据 (src/fixtures)
seed:
    cargo loco task seed_data

# 重置数据库并重新填充种子数据
seed-refresh:
    cargo loco task seed_data refresh:true

# 列出所有已注册路由
routes:
    cargo loco routes

# 运行 Loco Task (空参数时列出全部 Task)
task *args="":
    cargo loco task {{args}}

# 检查开发环境配置与依赖健康状态
doctor:
    cargo loco doctor

# ==============================================================================
# 3. 质量与测试 (Quality, Formatting & Testing)
# ==============================================================================

# 运行全量测试 (支持透传，如 `just test auth`)
test *args="":
    LOCO_ENV=test cargo nextest run --no-fail-fast {{args}}

bench path="/livez" *args="":
    wrk -t4 -c200 -d30s --latency {{args}} "http://localhost:${APP_PORT:-5150}{{path}}"

# 兼容别名 (nextest -> test)
alias nextest := test

# 查看 sccache 缓存命中率与磁盘统计
sccache-stats:
    sccache --show-stats

# 格式化 Rust 源码
fmt:
    cargo fmt --all

# Clippy 静态检查 (警告即错误)
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# 自动修复格式与 Clippy 警告
fix:
    cargo clippy --fix --allow-dirty --allow-staged

# ==============================================================================
# 4. 辅助工具 (Utilities)
# ==============================================================================

# 抽取 t! 文案词条并更新 locales/ 字典
i18n *args="":
    cargo i18n {{args}}

# 检查依赖安全漏洞 (RustSec 数据库)
audit:
    cargo audit

# 生成 HTML 覆盖率报告
coverage:
    cargo llvm-cov --html

# 安全清理：删增量缓存 + 清自身 crate 产物 + 将 target 压回 5GB 上限
# (仅超出上限时才从最旧产物开始淘汰，依赖产物默认保留)
clean:
    rm -rf target/debug/incremental
    cargo clean -p locos -p migration
    @which cargo-sweep >/dev/null 2>&1 \
        && cargo sweep --maxsize 5GB \
        || echo "（cargo-sweep 未安装，已跳过体积压缩；可执行 mise install 补齐）"
    @which cargo-sweep >/dev/null 2>&1 \
        && cargo sweep --installed \
        || true
    @du -sh target 2>/dev/null || true

# 彻底清空全部构建产物 (下次构建全量重编)
clean-deep:
    cargo clean

# 清理 cargo 全局缓存 (registry 中未被引用的 crate)
clean-registry:
    cargo cache --autoclean

# 检查依赖的最新可用版本 (仅扫描一级直接依赖)
outdated:
    cargo outdated -d 1

# ★ 日常推荐 ★ 刷新 Cargo.lock 并更新依赖补丁版本
update:
    cargo update
update-p *args="":
    cargo update -p {{args}}

# 升级 Cargo.toml 中的依赖版本 (支持透传，如 `just upgrade --incompatible`)
upgrade *args="":
    cargo upgrade {{args}}
