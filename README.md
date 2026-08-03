# Loco Starter

Rust web & API application built with [Loco](https://loco.rs) (Sea-ORM, Axum, Tokio).

## Quick Start

```sh
mise run dev     # 启动热重载开发服务器
mise run test    # 运行测试套件
mise run check   # 运行 Clippy 规范检查
```

### Mise Tasks

| 命令 | 说明 |
|---|---|
| `mise run dev` | 启动热重载开发服务器 (`cargo watch`) |
| `mise run start` | 启动应用主服务 |
| `mise run test` | 运行单元测试与集成测试 |
| `mise run migrate` | 执行数据库 Migration |
| `mise run routes` | 查看所有注册路由 |
| `mise run check` | 运行 Clippy 检查 |
| `mise run fix` | 自动修复格式 (fmt) 与 Clippy 警告 |
| `mise run up` | 启动 Docker Compose 容器 |
| `mise run down` | 停止并销毁 Docker Compose 容器 |

## Docker

```sh
docker compose up -d    # 启动容器服务
```

## Agent Skills

`.agents/skills/` 项目专精技能库：

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
