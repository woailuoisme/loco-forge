# Agent Guide for Loco Starter App

这是一个基于 **Loco.rs** (Rust 现代 Web 框架) 的全栈应用。
AI Agent 在此项目中编写、重构代码或排查问题时，必须严格遵守以下规范。

---

## 核心原则 (Core Principles)

1. **优先使用 Loco 原生脚手架与内置组件**
   - 严禁手写繁琐的基础设施代码。新增模块、模型、控制器、后台任务、邮件、迁移时，**必须优先使用生成器**：
     - `cargo loco generate model <name> [field:type...]`
     - `cargo loco generate controller <name>`
     - `cargo loco generate scaffold <name> [field:type...]`
     - `cargo loco generate migration <name>`
     - `cargo loco generate worker <name>`
     - `cargo loco generate task <name>`
   - 生成器会自动将新代码绑定注册到 `src/app.rs`。

2. **统一使用 `AppContext` (`ctx`)**
   - 所有数据库操作、配置读取、邮件发送、缓存与队列
     **统一通过 `ctx` 处理**
     （例如 `ctx.db`, `ctx.config`, `ctx.mailer`, `ctx.cache`, `ctx.storage`）。
   - 禁止在业务代码中自行创建独立的数据库连接池或服务实例。

3. **Sea-ORM 实体约束**
   - `src/models/_entities/` 目录下的文件为 Sea-ORM
     **自动生成的代码**，**绝对禁止手动修改**。
   - 自定义业务逻辑、关联查询和模型验证必须编写在 `src/models/*.rs` 中。
   - 主键与外键类型统一使用 `i64`（Loco 0.17+ 规范）。

4. **错误处理与代码风格**
   - 控制器、模型与 Task 返回类型统一为 `loco_rs::Result<T>`，
     优先使用 `?` 运算符向上抛出错误。
   - 模块顶部统一导入 `use loco_rs::prelude::*;`。

---

## 目录结构说明 (Directory Map)

- `src/app.rs`：App 钩子实现与路由/Worker/Task 注册中心（全局枢纽）。
- `src/controllers/`：HTTP 控制器与路由定义。
- `src/models/_entities/`：自动生成的 Sea-ORM 实体（**禁止手写修改**）。
- `src/models/`：业务模型与数据库操作逻辑。
- `src/dtos/`：请求与响应的数据传输对象 (DTO)。
- `src/workers/`：后台异步 Job/Task 队列。
- `src/tasks/`：命令行 / 管理员 Task。
- `src/mailers/`：邮件模板与发送逻辑。
- `migration/`：Sea-ORM 数据库迁移脚本。
- `config/*.yaml`：环境配置文件（支持 Tera `get_env` 获取环境变量）。
- `tests/`：单元测试与 HTTP API 集成测试。

---

## 常用命令与 CLI 工具规范 (Tooling & CLI)

项目集成了 **Mise** 任务管理与 **RTK (Rust Token Killer)** 命令优化代理：

```bash
# 开发与构建
mise run dev          # 启动热重载开发服务 (cargo dev)
mise run start        # 启动 Web 服务 (cargo loco start)
mise run test         # 运行单元测试与集成测试 (cargo test)
mise run check        # 运行 Clippy 代码检查 (cargo clippy)
mise run fix          # 自动修复格式与 Clippy 警告

# 数据库与路由
mise run migrate      # 执行数据库迁移 (cargo loco db migrate)
mise run routes       # 查看当前所有已注册的路由表 (cargo loco routes)

# 依赖与容器
mise run doctor       # 检查开发环境健康状态 (cargo loco doctor)
mise run up           # 使用 Docker Compose 启动容器
mise run down         # 停止并销毁 Docker Compose 容器
```

> **AI 命令调用规范**：在终端执行 Git 和 Cargo 命令时，
> 系统钩子会自动使用 `rtk` 进行 Token 压缩。
> 请优先使用 `mise run <task>` 简写执行日常任务。

---

## 测试规范 (Testing Guidelines)

- 新增 API 或模型功能时，必须遵守 **TDD** 流程，在 `tests/` 中编写或更新测试。
- HTTP 集成测试模式：
  `request::<App, _, _>(|request, ctx| async move { ... }).await;`
- 保证测试的自包含性与数据库隔离。

---

## 参考文档 (References)

- [Loco 官方 Agent 指南](https://loco.rs/AGENTS.md)
- [Loco 单文件完整参考](https://loco.rs/llms-full.txt)
- [Loco 官方文档](https://loco.rs/docs)
