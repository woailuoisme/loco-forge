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

5. **Sea-ORM 实体约束与 Data Repository 规范**
   - `src/models/_entities/` 目录下的文件为 Sea-ORM
     **自动生成的代码**，**绝对禁止手动修改**。
   - 自定义业务逻辑、关联查询和模型验证必须编写在 `src/models/*.rs` 中。
   - 主键与外键类型统一使用 `i64`（Loco 0.17+ 规范）。
   - **Data Repository 三段式结构**：每个 `src/models/<name>.rs` 必须按以下三个 `impl` 块组织，明确职责边界：
     - `impl Model { … }`：单实例读写方法（`find_by_id`, `create`, `update_xxx`, `delete`）
     - `impl ActiveModel { … }`：字段级钩子与 `ActiveModelBehavior`（如 `before_save`）
     - `impl Entity { … }`：集合级查询（分页、搜索、批量删除、聚合统计）
   - **SeaORM 原生 filter 写法**：统一使用 `Column::Field.eq(value)` / `.is_in(...)` / `.contains(...)` 等原生表达式，
     **禁止**使用已废弃的 `model::query::condition().eq(...).build()` 旧式写法。
   - **批量操作规范**：
     - 批量删除：`Entity::delete_many().filter(Column::Id.is_in(ids)).exec(db)` — 单条 SQL，零 N+1
     - 批量更新：`Entity::update_many().col_expr(Column::Field, Expr::value(val)).filter(...).exec(db)` — 禁止循环逐行 update
     - 空切片保护：批量操作入口处须 `if ids.is_empty() { return Ok(0); }` 提前返回
   - **分页查询规范**：统一使用 SeaORM `Paginator`（`.paginate(db, page_size)`），
     页码从 1 开始入参，内部转换 `page.saturating_sub(1)` 传给 `.fetch_page()`，
     同时返回 `(Vec<Model>, total_pages: u64)` 元组。
   - **`#[must_use]` 使用规则**：
     - 纯查询、纯计算的无副作用方法（如 `is_admin`, `verify_password`, `has_role`）**须**标注 `#[must_use]`
     - 返回类型已是 `#[must_use]`（如 `Result<T>`, `Option<T>`）的方法**禁止**再叠加 `#[must_use]`（避免 `clippy::double_must_use`）

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

9. **Rust 2024 现代编码范式 (Rust 2024 Modern Idioms)**
   - **`let-else` 提前返回**：复杂条件解包与模式匹配防护子句（Guard Clauses）优先使用 `let ... else { return ...; }`，消除深层嵌套的 `if let` / `match`。
   - **零堆内存分配链式处理**：在 Iterator、Option 与 Result 的转换中，优先使用 `find_map`、`and_then`、`or_else`、`then_some` 等函数式组合算子。
   - **Precise Capturing 与生命周期省略**：遵循 Rust 2024 原生 Trait 异步与生命周期省略规范，禁止书写冗余的 `'a` 生命周期标注（符合 `clippy::needless_lifetimes`）。
   - **Workspace 依赖解析器**：项目统一配置 `edition = "2024"` 与 `resolver = "3"` (MSRV-aware 依赖解析器)。
   - **零 Clippy 警告门禁**：所有新增与重构代码必须通过 `cargo clippy --all-targets -- -D warnings` 零警告检查。

10. **代码质量门禁 (Code Quality Gate)**
    - **MUST**：每次完成全部代码修改后（一轮 multi-file 编辑结束、一个 feature/fix 任务收尾），**必须**按顺序执行：
     1. `just fmt` — 统一格式化所有 Rust 源码与 Markdown 文档
     2. `just lint` — 全量 Clippy 静态检查 (`--all-targets --all-features -- -D warnings`) + Markdown 审计
    - 若 `just lint` 报错或警告，必须在任务结束前修复全部问题，确保零警告通过。
    - 此门禁是"完成"的定义之一：未通过 `just lint` 的代码视为未完成。

---

## 分层架构与组件职责 (Layered Architecture & Responsibilities)

| 架构层级 | 所在目录 | 职责范围 (Responsibilities) | 禁止事项 (Anti-Patterns) |
|---|---|---|---|
| **Controller 层** | `src/controllers/` | `#[debug_handler]` + `#[utoipa::path(...)]` 标注；`State(ctx)` 提取上下文；`Json(params)` / `Path(id)` 提取入参；调用 Service 方法；用 `format_res::ok/empty_msg/unauthorized` 统一格式化响应；`routes()` 函数内注册路由并按需挂载中间件层 | ❌ **禁止**直接写 SQL / Sea-ORM 查询；❌ **禁止**直接触发邮件/队列；❌ **禁止**在 Handler 函数内包含业务逻辑 |
| **Service 层** | `src/services/` | `pub struct XxxService;` 关联函数模式；接收 `&AppContext` + DTO 参数；编排 Model 查询、Mailer 调用、Worker 投递；静默处理用户不存在（返回 `Ok(())` 避免信息泄露）；用 `OnceLock<Regex>` 缓存复用正则编译结果 | ❌ **禁止**直接依赖 `axum::http::Request` / Header 对象；❌ **禁止**手动构造 HTTP 状态码或 Response；❌ **禁止**在 Service 层直接访问数据库（须通过 Model 方法） |
| **Model 层** | `src/models/` | Data Repository 三段式结构（`impl Model` 单实例读写 / `impl ActiveModel` 钩子行为 / `impl Entity` 集合查询）；SeaORM 原生 filter；批量操作用 `delete_many`/`update_many` 消除 N+1；分页返回 `(Vec<Model>, total_pages)`；纯方法标注 `#[must_use]` | ❌ **禁止**手动修改 `src/models/_entities/` 自动生成文件；❌ **禁止**在 `impl Entity` 或 `impl Model` 中放置跨模型事务（归 Service 层）；❌ **禁止**使用旧式 `query::condition().build()` 写法；❌ **禁止**在批量方法中逐行循环 update/delete |
| **DTO 层** | `src/dtos/` | Request (`XxxRequest`) 标注 `#[derive(Deserialize, Validate, ToSchema)]`；Response Resource (`XxxResource`) 标注 `#[derive(Serialize, ToSchema)]` 并实现 `From<Model>`；分页统一用 `PaginatedResource<T>` + `PageMeta` + `PaginationRequest` | ❌ **禁止**放置业务逻辑；❌ **禁止**使用 `Params`, `Dto`, `Response` 等非标后缀；❌ **禁止** Resource 直接暴露密码/Token 等敏感字段 |
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
> 3. **代码质量门禁（参见核心原则第 10 条）**：每轮完整代码修改后，**必须**依次执行 `just fmt` → `just lint`，零警告通过才算任务完成。

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

## Controller 层最佳实践 (Controller Best Practices)

### Handler 标准结构

每个 Handler 函数保持**薄层**：提取入参 → 调用 Service → 格式化响应，禁止在函数体内编写业务逻辑。

```rust
/// 用户登录
#[utoipa::path(
    post, path = "/api/v1/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, body = ApiResponse<LoginResource>),
        (status = 401, body = ApiError)
    ),
    tag = TAG_AUTH
)]
#[debug_handler]
pub async fn login(
    State(ctx): State<AppContext>,
    Json(params): Json<LoginRequest>,
) -> Result<Response> {
    match AuthService::login(&ctx, &params).await? {
        Ok(res) => format_res::ok(res),
        Err(msg) => format_res::unauthorized(msg),
    }
}
```

### 统一响应格式化 (`format_res`)

| 场景 | 调用方式 |
|---|---|
| 成功返回数据 | `format_res::ok(resource)` |
| 成功但无数据 | `format_res::empty_msg("操作成功")` |
| 401 未授权 | `format_res::unauthorized("msg")` |
| 400 参数错误 | `format_res::bad_request("msg")` |

### 路由注册规范

路由统一在 `routes()` 关联函数中注册，中间件按需挂载到 Routes 层：

```rust
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/v1/auth")
        .add("/login", post(login))
        .add("/current", get(current))
        .layer(auth_rate_limit_layer())   // 按需挂载限流层
}
```

---

## Service 层最佳实践 (Service Best Practices)

### 关联函数模式

Service 结构体无字段，所有方法为关联函数，接收 `&AppContext` 作为第一参数：

```rust
pub struct AuthService;

impl AuthService {
    pub async fn register(ctx: &AppContext, params: &RegisterRequest) -> Result<()> {
        // 调用 Model 层 → 触发 Mailer → 返回 Result
    }
}
```

### 静默处理用户不存在

认证类接口**不暴露用户是否存在**，统一静默返回 `Ok(())` 防止信息泄露：

```rust
// ✅ 推荐
let Ok(user) = users::Model::find_by_email(&ctx.db, email).await else {
    return Ok(());  // 静默返回，不透露用户是否存在
};

// ❌ 禁止
if !user_exists { return Err(Error::Unauthorized()); }
```

### 正则编译缓存

使用 `OnceLock<Regex>` 全局懒加载，避免每次请求重复编译正则：

```rust
pub static EMAIL_DOMAIN_RE: OnceLock<Regex> = OnceLock::new();

fn get_allow_email_domain_re() -> &'static Regex {
    EMAIL_DOMAIN_RE.get_or_init(|| {
        Regex::new(ALLOW_EMAIL_DOMAIN_PATTERN).expect("Invalid regex")
    })
}
```

### admin 子模块

管理后台专属 Service 放置于 `src/services/admin/`，避免与用户端逻辑混用。

---

## DTO 层最佳实践 (DTO Best Practices)

### Request DTO 规范

```rust
/// 创建文章请求参数 [Request DTO]
#[derive(Deserialize, Validate, TS, ToSchema)]
pub struct CreatePostRequest {
    /// 文章标题
    #[validate(length(min = 1, message = "文章标题不能为空"))]
    pub title: Option<String>,
    pub content: Option<String>,
}
```

- 统一 `#[derive(Deserialize, Validate, ToSchema)]`
- 使用 `validator::Validate` 做字段级校验（`#[validate(...)]`）
- 敏感字段（密码）使用 `SecretString` + `#[serde(skip_serializing)]`

### Response Resource 规范

```rust
/// 文章响应数据对象 [Response DTO]
#[derive(Serialize, TS, ToSchema)]
pub struct PostResource {
    pub id: i64,
    pub title: Option<String>,
    pub content: Option<String>,
    pub created_at: DateTimeWithTimeZone,
}

impl From<posts::Model> for PostResource {
    fn from(m: posts::Model) -> Self {
        Self { id: m.id, title: m.title, content: m.content, created_at: m.created_at }
    }
}
```

- 统一实现 `From<Model>` 解耦 Model 与响应格式
- 纯工厂方法（`::new()`）标注 `#[must_use]`
- **禁止**在 Resource 中暴露密码、内部 Token 等敏感字段

### 分页 DTO 三件套

| 类型 | 用途 |
|---|---|
| `PaginationRequest` | Query 参数提取（`page`, `per_page`，含上下界合法化） |
| `PaginatedResource<T>` | 统一分页响应容器（`items` + `meta`） |
| `PageMeta` | 分页元数据（`current_page`, `per_page`, `last_page`, `total`） |

```rust
// Controller 端
async fn list(Query(pq): Query<PaginationRequest>, ...) -> Result<Response> {
    let (posts, total_pages) = posts::Entity::find_all_paginated(
        &ctx.db, pq.page(), pq.per_page()
    ).await?;
    let resources: Vec<PostResource> = posts.into_iter().map(Into::into).collect();
    format_res::ok(PaginatedResource {
        items: resources,
        meta: PageMeta::new(pq.page(), pq.per_page(), total_pages),
        extras: None,
    })
}
```

---

## Model Data Repository 最佳实践 (Model Best Practices)

### 三段式 `impl` 结构

每个 `src/models/<name>.rs` 文件按如下三个块组织，**不得跨块混写**：

```rust
// ① 单实例读写 — find_by_id / create / update_xxx / delete
impl Model { … }

// ② ActiveModel 钩子 — before_save / before_delete 等生命周期
impl ActiveModel { … }

// ③ 集合查询 — 分页、搜索、批量删除、聚合统计
impl Entity { … }
```

### SeaORM 原生 Filter 写法

```rust
// ✅ 推荐 — 原生列表达式
Entity::find().filter(Column::Email.eq(email))
Entity::find().filter(Column::Id.is_in(ids.to_vec()))
Entity::find().filter(Column::Title.contains(&pattern))

// ❌ 禁止 — 已废弃旧式写法
model::query::condition().email().eq(email).build()
```

### 批量删除（单条 SQL，零 N+1）

```rust
pub async fn delete_by_ids(db: &DatabaseConnection, ids: &[i64]) -> ModelResult<u64> {
    if ids.is_empty() { return Ok(0); }          // 空切片保护
    let res = Entity::delete_many()
        .filter(Column::Id.is_in(ids.to_vec()))
        .exec(db)
        .await?;
    Ok(res.rows_affected)
}
```

### 批量字段更新（单条 SQL，替代 N+1 循环）

```rust
Entity::update_many()
    .col_expr(Column::Field, Expr::value(Option::<Type>::None))
    .filter(Column::ExpiredAt.lt(now))
    .exec(db)
    .await?;
```

### 分页查询约定

- 入参 `page: u64` 从 **1 开始**，内部转 `page.saturating_sub(1)` 传给 `.fetch_page()`
- 返回值统一为 `ModelResult<(Vec<Model>, u64)>` — `(数据列表, 总页数)`
- 排序默认 `order_by_desc(Column::CreatedAt)` 保持新旧一致性

### `#[must_use]` 使用规则

| 情形 | 处理方式 |
|---|---|
| 纯查询 / 纯计算方法（`is_admin`, `verify_password`, `has_role`） | **加** `#[must_use]` |
| 返回 `Result<T>` / `Option<T>` 的方法 | **不加**（类型本身已是 `#[must_use]`，重复会触发 `clippy::double_must_use`） |
| 有副作用的写入方法（`create`, `update`, `delete`） | **不加** |

---

## 参考文档 (References)

- [Loco 官方 Agent 指南](https://loco.rs/AGENTS.md)
- [Loco 单文件完整参考](https://loco.rs/llms-full.txt)
- [Loco 官方文档](https://loco.rs/docs)
