# 默认列出所有可用的 just 命令
default:
    @just --list

# 启动 Loco Web 服务
start:
    cargo loco start

# 启动开发环境热重载服务 (自动检测 src/ 及 config/ 变更并重启)
dev:
    cargo watch -w src -w config -s 'cargo run -- start'

# 运行数据库迁移
migrate:
    cargo loco db migrate

# 查看当前所有已注册的路由列表
routes:
    cargo loco routes

# 运行项目单元测试与集成测试 (基础驱动，兼容 SQLite 串行测试)
test:
    cargo test

# 使用 nextest 运行极速并行测试
nextest:
    cargo nextest run

# 检查依赖包的安全漏洞 (RustSec 漏洞数据库)
audit:
    cargo audit

# 生成并查看 HTML 格式的代码测试覆盖率报告
coverage:
    cargo llvm-cov --html

# 运行类型检查与 Clippy 代码规范检查
check:
    cargo clippy

# 格式化所有 Rust 源代码
fmt:
    cargo fmt --all

# 自动修复格式（fmt）与 Clippy 警告（允许未提交/暂存的改动）
fix:
    cargo clippy --fix --allow-dirty --allow-staged && cargo fmt --all

# 检查当前 Loco 开发环境配置与依赖健康状态
doctor:
    cargo loco doctor

# 清理 target 构建目录以释放磁盘空间
clean:
    cargo clean

# 检查项目依赖包的最新可用版本 (仅扫描 Cargo.toml 一级直接依赖)
outdated:
    cargo outdated -d 1

# 将 Cargo.toml 中的依赖包版本直接升级至最新版本 (仅修改配置文件，类比 ncu -u)
upgrade:
    cargo upgrade

# 刷新 Cargo.lock 锁文件并下载安装最新依赖 (类比 bun install)
update:
    cargo update

# update 的同义别名 (类比 bun install / npm install)
alias install := update

# 使用 Docker Compose 启动 App 容器
up:
    docker compose up --build -d

# 停止并销毁 Docker Compose 容器
down:
    docker compose down
