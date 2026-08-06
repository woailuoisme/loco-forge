# 开启自动装载 .env 环境变量文件
set dotenv-load := true

# 默认列出所有可用的 just 命令
default:
    @just --list

# 释放 Web 服务端口占用 (自动读取 .env 中的 PORT 变量，默认 5150)
kill-port port=env('PORT', '5150'):
    @lsof -ti:{{port}} | xargs kill -9 2>/dev/null || true

# ==============================================================================
# 1. 核心开发与运行 (Development & Execution)
# ==============================================================================

# 启动开发环境热重载服务 (自动清理端口并监听 src/ 及 config/)
dev: kill-port
    cargo watch -w src -w config -s 'cargo run -- start'

# 使用 concurrently 并行启动 Server、Worker 与 Scheduler（带热重载与端口清理）
dev-all: kill-port
    concurrently -k -n "server,worker,scheduler" -c "blue,magenta,yellow" \
        "cargo watch -w src -w config -s 'cargo run -- start'" \
        "cargo watch -w src -w config -s 'cargo run -- start --worker'" \
        "cargo watch -w src -w config -s 'cargo run -- scheduler'"

# 启动默认 Loco Web 服务 (Debug 调试模式，自动清理端口)
start: kill-port
    cargo loco start

# 单进程同时启动 Web Server、Worker 与 Scheduler
start-all:
    cargo loco start --all

# 开发环境编译构建 (Cargo Check & Debug 二进制)
build:
    cargo build

# 生产环境 Release 高性能优化编译
build-prod:
    cargo build --release

# 以 Release 高性能优化模式启动 Loco Web 服务 (自动清理端口)
prod: kill-port
    cargo run --release -- start

# 启动后台 Job 消费进程 (Worker)
worker:
    cargo loco start --worker

# 启动定时任务调度服务 (Scheduler)
scheduler:
    cargo loco scheduler

# 兼容与方便调用的旧别名 (Aliases for backward compatibility)
alias build-release := build-prod
alias start-release := prod

# ==============================================================================
# 2. 数据库与领域 Task (Database & Loco Tasks)
# ==============================================================================

# 运行数据库迁移
migrate:
    cargo loco db migrate

# 查看当前所有已注册的路由列表
routes:
    cargo loco routes

# 运行 Loco 后台/管理 Task (空参数时列出所有 Task 列表)
task *args="":
    cargo loco task {{args}}

# 检查当前 Loco 开发环境配置与依赖健康状态
doctor:
    cargo loco doctor

# ==============================================================================
# 3. 代码质量与测试 (Quality, Formatting & Testing)
# ==============================================================================

# 运行项目单元测试与集成测试
test:
    cargo test

# 使用 nextest 运行极速并行测试
nextest:
    cargo nextest run

# 运行全量代码静态规范检查 (Clippy 检查与 Rustfmt 格式验证)
lint:
    cargo clippy --all-targets --all-features -- -D warnings
    cargo fmt --all --check

alias check := lint

# 格式化所有 Rust 源代码
fmt:
    cargo fmt --all

# 自动修复格式（fmt）与 Clippy 警告（允许未提交/暂存的改动）
fix:
    cargo clippy --fix --allow-dirty --allow-staged && cargo fmt --all

# 检查依赖包的安全漏洞 (RustSec 漏洞数据库)
audit:
    cargo audit

# 生成并查看 HTML 格式的代码测试覆盖率报告
coverage:
    cargo llvm-cov --html

# 清理 target 构建目录以释放磁盘空间
clean:
    cargo clean

# 检查项目依赖包的最新可用版本 (仅扫描 Cargo.toml 一级直接依赖)
outdated:
    cargo outdated -d 1

# ★ 日常推荐 ★ 刷新 Cargo.lock 锁文件并下载安装最新依赖补丁包 (类比 bun install / npm install)
update:
    cargo update

alias install := update

# 将 Cargo.toml 中的依赖包版本升级 (支持透传参数如 `just upgrade --incompatible`)
upgrade *args="":
    cargo upgrade {{args}}


