# Agent Guide for Loco Starter App

这是一个基于 **Loco.rs** (Rust 现代 Web 框架) 的全栈应用。
AI Agent 在此项目中编写、重构代码或排查问题时，必须严格遵守以下规范。

---

## 核心原则 (Core Principles)

1. **优先使用 Loco 原生脚手架与内置代码生成器 (`cargo loco generate`)**
   - 严禁手写繁琐的基础设施样板代码。
     新增模型、控制器、脚手架、迁移、后台任务、邮件、调度器、数据加载器、部署配置等模块时，
     **必须优先使用生成器**：
     - `cargo loco generate model <name> [field:type...]` (生成 Model 数据模型与对应测试逻辑)
     - `cargo loco generate controller <name>` (生成 API Controller 与对应集成测试)
     - `cargo loco generate scaffold <name> [field:type...]` (全栈生成 CRUD Scaffold：Model, Migration, Controller, Views/DTOs 与 Tests)
     - `cargo loco generate migration <name>` (生成 Sea-ORM 数据库迁移脚本)
     - `cargo loco generate worker <name>` (生成 Worker 异步后台任务队列)
     - `cargo loco generate task <name>` (生成 CLI 管理员 Task)
     - `cargo loco generate mailer <name>` (生成 Mailer 邮件类与模板)
     - `cargo loco generate scheduler` (生成定时调度任务配置模板)
     - `cargo loco generate data` (生成 Data Loader 数据加载与 Seed 逻辑)
     - `cargo loco generate deployment` (生成 Docker / K8s 部署配置文件)
     - `cargo loco generate override` (覆写与自定义内置的代码生成模板)
   - 生成器会自动完成新模块在 `src/app.rs` 中的模板代码绑定与注册。

2. **业务逻辑下沉 Service 层与 Controller 薄层化**
   - **Controller 保持薄层**：控制器仅负责请求解析 (Axum extractors)、调用 Service 层方法、以及统一 Response 响应格式化 (`dtos::common::format_res`)。
   - **Service 层收拢业务**：所有跨模型事务、组合查询、外部服务调用统一放置在 `src/services/` (或管理后台专属 `src/services/admin/`) 中。
   - **关联函数模式**：Service 结构体统一声明为 `pub struct XxxService;`，采用静态关联函数模式（如 `XxxService::create(&ctx, params)`），无需实例化结构体。

3. **全局常量统一管理 (`src/common/consts.rs`)**
   - 所有应用名称、默认分页参数、存储路径、日志配置默认值统一在 `src/common/consts.rs` 中使用 `pub const` 或子模块划分（如 `consts::app::*`, `consts::logger::*`）。
   - **严禁**在业务逻辑或控制器中直接写死魔数 (Magic numbers) 或配置字符串。

4. **统一使用 `AppContext` (`ctx`)**
   - 所有数据库操作、配置读取、邮件发送、缓存与队列
     **统一通过 `ctx` 处理**
     （例如 `ctx.db`, `ctx.config`, `ctx.mailer`, `ctx.cache`, `ctx.storage`）。
   - 禁止在业务代码中自行创建独立的数据库连接池或服务实例。

5. **Sea-ORM 实体约束**
   - `src/models/_entities/` 目录下的文件为 Sea-ORM
     **自动生成的代码**，**绝对禁止手动修改**。
   - 自定义业务逻辑、关联查询和模型验证必须编写在 `src/models/*.rs` 中。
   - 主键与外键类型统一使用 `i64`（Loco 0.17+ 规范）。

6. **错误处理、日志规范与代码风格**
   - 控制器、模型、Service 与 Task 返回类型统一为 `loco_rs::Result<T>`，
     优先使用 `?` 运算符向上抛出错误。
   - 模块顶部统一导入 `use loco_rs::prelude::*;`。
   - **严禁使用 `println!` / `eprintln!`**：Loco 的日志系统统一基于 **`tracing`** 基础设施。在所有控制器、Service、模型与命令行 Task（`src/tasks/`）中，**绝对禁止直接使用 `println!` 或 `eprintln!`**。所有日志与提示信息必须统一使用 `tracing::info!`、`tracing::warn!`、`tracing::error!`、`tracing::debug!`，使其受环境配置文件（`config/*.yaml`）中的日志级别、日志格式与 `file_appender` 轮转文件管控。

7. **优先使用 Loco 内置的功能服务与基础设施**
   - 各种通用功能服务（如身份认证/JWT 校验、文件存储 `ctx.storage`、缓存 `ctx.cache`、邮件发送 `ctx.mailer`、后台任务队列 `ctx.queue` / `Worker`、数据库与事务 `ctx.db`、环境配置 `ctx.config`、日志 `tracing` 等）**必须优先使用 Loco 框架原生内置的服务与能力**。
   - **严禁**在 Loco 已提供内置支持的情况下重复造轮子或盲目引入冗余的第三方替代依赖。

8. **DTO 命名与后缀规范 (DTO Naming Conventions)**
   - **请求 DTO (Request DTO)**：所有客户端提交的 Payload / Query / Path 参数结构体，统一以 **`Request`** 为后缀（例如 `CreatePostRequest`, `UpdatePostRequest`, `LoginRequest`, `PaginationRequest`）。
   - **响应 DTO (Response DTO)**：所有返回给前端或客户端展示的 Resource / Data 结构体，统一以 **`Resource`** 为后缀（例如 `PostResource`, `LoginResource`, `CurrentUserResource`, `PaginatedResource`）。
   - **严禁**使用 `Params`, `Dto`, `Response` 等非标或杂乱后缀作为 DTO 结构体名称。

---

## 分层架构与组件职责 (Layered Architecture & Responsibilities)

| 架构层级 | 所在目录 | 职责范围 (Responsibilities) | 禁止事项 (Anti-Patterns) |
|---|---|---|---|
| **Controller 层** | `src/controllers/` | 校验入参 DTO、提取 `State(ctx)`、路由映射、调用 Service 并返回 `Response` | ❌ **禁止**直接写 SQL / Sea-ORM 复杂事务查询；❌ **禁止**直接处理邮件/队列 |
| **Service 层** | `src/services/` | 编排业务流程、跨 Model 事务处理、业务规则校验、触发 Worker/Mailer | ❌ **禁止**直接依赖 HTTP Request/Header 对象；❌ **禁止**手动返回 HTTP StatusCode |
| **Model 层** | `src/models/` | 数据库表操作、模型字段 Validation、单个实体的拓展 CRUD 方法 | ❌ **禁止**手动修改 `src/models/_entities/` 自动生成文件 |
| **DTO 层** | `src/dtos/` | 定义 Request (`XxxRequest`) / Response Resource (`XxxResource`) 契约结构体、实现 `utoipa::ToSchema` 与 `From` 转换 | ❌ **禁止**放置复杂业务处理逻辑；❌ **禁止**使用 `Params`, `Dto`, `Response` 等非标后缀 |
| **Common 层** | `src/common/` | 全局常量 (`consts.rs`)、工具函数 (`utils/`) | ❌ **禁止**依赖上层 Controller 或 Service 模块 |
| **Setup 层** | `src/setup/` | HTTP 中间件注册 (i18n, rate_limit, logger)、路由总装 | ❌ **禁止**将具体业务 Endpoint 逻辑写入中间件 |

---

## 目录结构说明 (Directory Map)

- `src/app.rs`：App 钩子实现与全局路由/Worker/Task 注册中心（全局枢纽）。
- `src/common/`：全局通用基础设施 (常量定义 `consts.rs`，工具函数 `utils/`)。
- `src/controllers/`：HTTP 控制器与路由定义 (`v1/auth.rs`, `v1/admin/posts.rs`, `health.rs`)。
- `src/dtos/`：请求与响应的数据传输对象 (`common/`, `posts.rs`)。
- `src/services/`：核心业务逻辑服务 (`auth.rs`, `admin/posts.rs`)。
- `src/models/_entities/`：自动生成的 Sea-ORM 实体（**禁止手写修改**）。
- `src/models/`：业务模型与数据库扩展逻辑。
- `src/setup/middleware/`：HTTP 中间件 (`rate_limit.rs`, `logger.rs`, `i18n.rs`)。
- `src/workers/`：后台异步 Job/Task 队列。
- `src/tasks/`：命令行 / 管理员 Task。
- `src/mailers/`：邮件模板与发送逻辑。
- `migration/`：Sea-ORM 数据库迁移脚本。
- `locales/`：i18n 多语言国际化翻译字典目录 (`en/`, `zh-CN/` 内含 `app.yml`, `auth.yml`, `errors.yml`, `validation.yml`)。
- `config/*.yaml`：环境配置文件（支持 Tera `get_env` 获取环境变量）。
- `tests/`：单元测试与 HTTP API 集成测试。

---

## 常用命令与 CLI 工具规范 (Tooling & CLI)

项目集成了 **Mise** 任务管理与 **RTK (Rust Token Killer)** 命令优化代理：

```bash
# 开发与构建
mise run dev      # 启动热重载开发服务 (cargo watch)
mise run start    # 启动 Web 服务 (cargo loco start)
mise run prod     # Release 高性能模式 (cargo run --release -- start)
mise run test     # 极速并行测试 (cargo nextest run)
mise run lint     # 运行 Clippy 代码检查 (cargo clippy)
mise run fix      # 自动修复格式与 Clippy 警告

# 数据库与路由
mise run migrate  # 执行数据库迁移 (cargo loco db migrate)
mise run routes   # 查看当前所有已注册的路由表 (cargo loco routes)

# 依赖与容器
mise run doctor   # 检查开发环境健康状态 (cargo loco doctor)
mise run up       # 使用 Docker Compose 启动容器
mise run down     # 停止并销毁 Docker Compose 容器
```

> **AI 命令与工具调用规范**：
>
> 1. **优先使用现代化 Rust CLI 工具**：在终端查找文本、搜索文件或批量替换时，**必须优先使用**：
>    - **`rg` (ripgrep)**：替代传统 `grep` 进行高性能文本与代码内容搜索
>    - **`fd` (fd-find)**：替代传统 `find` 进行极速文件与目录树匹配
>    - **`sd` (sed alternative)**：替代传统 `sed` 进行直观安全且支持 Regex 的批量文本替换
> 2. **Token 压缩代理**：在终端执行 Git 和 Cargo 命令时，系统钩子会自动使用 `rtk` 进行 Token 压缩。请优先使用 `mise run <task>` 简写执行日常任务。

---

## 测试规范 (Testing Guidelines)

- **推荐测试运行器**: 优先使用 `cargo nextest run` (或 `mise run test`)
  进行极速并行测试（已配置 `.config/nextest.toml`
  控制数据库单线程串行与纯逻辑多线程并发）。
- **测试表达力与参数化**: 优先使用 **`rstest`**
  (`#[rstest]`, `#[case(...)]`, `#[fixture]`)
  替代冗余模板代码，精确定位用例边界断言。
- **并发保护与数据库隔离**：
  依赖全局 AppContext / 数据库事务的测试需标注
  `#[serial_test::serial]` 隔离。
- **限流测试保护**：测试用例中连续调用认证接口时，需注意 `auth_rate_limit_layer` 突发阈值限制。
- 新增 API 或模型功能时，必须遵守 **TDD** 流程，
  在 `tests/` 中编写或更新测试。
- HTTP 集成测试模式：
  `request::<App, _, _>(|request, ctx| async move { ... }).await;`

---

## 参考文档 (References)

- [Loco 官方 Agent 指南](https://loco.rs/AGENTS.md)
- [Loco 单文件完整参考](https://loco.rs/llms-full.txt)
- [Loco 官方文档](https://loco.rs/docs)
