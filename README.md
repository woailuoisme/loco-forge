# Loco Starter

Rust Web & API 全栈应用脚手架，基于 [Loco.rs](https://loco.rs) 现代 Web 框架 (Sea-ORM, Axum, Tokio) 构建。内置完善的 Service 业务分层、统一日志/常量、DTO 验证、限流中间件、JWT/Session 认证及并行测试体系。

---

## 🚀 快速开始 (Quick Start)

### 环境依赖

- **Rust**: 1.99.1 (workspace MSRV，详见 `Cargo.toml` / `rust-toolchain.toml`)
- **Database**: PostgreSQL / SQLite (由 `config/*.yaml` 配置)
- **Tooling**: [Mise](https://mise.jdx.dev) 任务管理器 (可选但强烈推荐)

### 常用开发命令

```sh
mise run dev     # 启动热重载开发服务器 (cargo watch)
mise run test    # 使用 Nextest 运行全量单元与集成测试
mise run lint    # 运行 Rust Clippy 静态检查与格式校验
mise run fix     # 自动修复格式 (fmt) 与 Clippy 警告
```

---

## 🛠️ Mise Tasks 任务列表

| 命令 | 关联底层指令 | 功能说明 |
|---|---|---|
| `mise run dev` | `cargo watch -x 'check'` / `cargo dev` | 启动热重载开发服务 |
| `mise run start` | `cargo loco start` | 启动 Web 服务 (Debug 模式) |
| `mise run prod` | `cargo run --release -- start` | 启动 Release 高性能模式 |
| `mise run test` | `cargo nextest run` | 极速并行运行单元测试与集成测试 |
| `mise run lint` | `cargo clippy --all-targets -- -D warnings` | 运行静态代码规范检查 |
| `mise run fix` | `cargo clippy --fix && cargo fmt` | 自动修复代码格式与 Clippy 警告 |
| `mise run migrate` | `cargo loco db migrate` | 执行数据库 Migration |
| `mise run routes` | `cargo loco routes` | 查看当前所有已注册的 HTTP 路由表 |
| `mise run doctor` | `cargo loco doctor` | 检查开发环境与依赖健康状态 |
| `mise run up` | `docker compose up -d` | 启动 Docker 依赖容器 |
| `mise run down` | `docker compose down` | 停止并销毁 Docker 容器 |

---

## 📂 项目目录结构 (Directory Structure)

```text
loco-starter/
├── .agents/                 # Workspace 专属 Agent Skills 与规则 (AGENTS.md)
├── .config/
│   └── nextest.toml         # Nextest 测试并发与数据库隔离策略配置
├── config/                  # 环境配置文件 (development, test, production yaml)
├── locales/                 # i18n 多语言国际化翻译字典 (en/, zh-CN/)
├── migration/               # Sea-ORM 数据库迁移脚本
├── src/
│   ├── app.rs               # App 钩子实现与全局路由/Worker/Task 注册中心
│   ├── common/              # 全局通用基础设施 (常量定义 consts.rs, 日志系统 logger/)
│   ├── controllers/         # HTTP 控制器 (路由 handler 与接口契约)
│   │   ├── health.rs        # 健康检查接口
│   │   └── v1/              # API v1 路由 (auth/ 十二子域, files, admin/)
│   ├── dtos/                # 请求/响应数据传输对象 (common/, auth/, admin/, files.rs)
│   ├── errors.rs            # 全局统一 Error 类型定义
│   ├── initializers/        # 框架启动初始化器 (openapi, config_validator)
│   ├── mailers/             # 邮件模板与发送逻辑
│   ├── models/              # 业务模型与 Sea-ORM 交互方法
│   │   └── _entities/       # Sea-ORM 自动生成的实体 (禁止手动编辑)
│   ├── services/            # 核心业务逻辑层 (auth/ 十二子域, admin/)
│   ├── setup/               # 基础设施与中间件 (middleware/ i18n, logger, rate_limit)
│   ├── tasks/               # CLI / 管理员 Task (clean_expired_tokens.rs)
│   └── workers/             # 异步后台 Job/Task 队列
├── storages/                # 本地存储目录 (日志, 文件上传, 临时导出)
└── tests/                   # 单元测试与 HTTP API 集成测试
```

---

## 🏗️ 架构亮点 (Architecture Highlights)

1. **Model 仓储深模块与 Service 业务编排** (`src/models/`, `src/services/`):
   - 数据模型采用三段式 Data Repository 架构 (`impl Model` 读写, `impl ActiveModel` 钩子, `impl Entity` 集合查询/分页/批量操作)。
   - 跨模型事务、邮件发送与异步任务编排下沉至 `Service` 关联函数 (如 `AuthService::register`, `AuthService::login`)，避免浅层透传。

2. **统一全局常量与日志** (`src/common/`):
   - 全局 App 配置、默认分页大小、文件存储路径统一收拢于 `src/common/consts.rs`，消除字符串硬编码。
   - 基于 `tracing-subscriber` 的多 Channel 日志输出驱动 (Console, Daily, Stack, Single, Json)，支持环境变量无缝重载。

3. **智能安全与限流** (`src/setup/middleware/rate_limit.rs`):
   - 基于 `tower-governor` 的 IP 智能提取器 (`SmartIpKeyExtractor`)，针对高风险接口提供令牌桶防暴力破解。

4. **极速测试隔离** (`.config/nextest.toml`):
   - 使用 `cargo nextest` 运行测试，区分纯逻辑单元测试 (多 CPU 全速并发) 与数据库集成测试 (单线程串行隔离 `db-tests`)。

---

## 🐳 Docker 部署

```sh
docker compose up -d    # 启动应用与依赖容器
```

生产镜像仅提供 Debian Trixie 构建（`Dockerfile`，cargo-chef 分层缓存），
多架构发布通过 CI（GitHub Actions / CNB）执行，详见对应工作流配置。

---

## 🤖 Agent Skills 专精技能库

项目根目录 `.agents/skills/` 内内置了专为 AI Agent 定制的技能规范：

```text
.agents/skills/
├── api-design/                   # RESTful API 设计规范
├── database-migrations/          # 数据库 Migration 零停机变更
├── docker-patterns/              # Docker 多阶段构建与 Compose 部署
├── m15-anti-pattern/             # [在线] Rust 反模式与避坑指南
├── openapi-spec-generation/      # [在线] OpenAPI 契约生成规范
├── postgres-patterns/            # PostgreSQL 性能优化与 Sea-ORM 规范
├── rust-async-patterns/          # Tokio 异步并发控制
├── rust-best-practices/          # [在线] Apollo 官方 Rust 最佳实践
├── rust-patterns/                # Rust Idiomatic 编码风格
└── rust-testing/                 # Rust 单元/集成测试与 TDD
```
