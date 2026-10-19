# ADR-0008: auth 域主键采用 UUIDv7 字符串（覆盖 i64 默认规范）

- 状态：Accepted
- 日期：2026-09-19

## 背景

项目规范（AGENTS.md）默认主键与外键统一使用 `i64`（Loco 0.17 规范）。但 auth 域（`user`、`account`、`session`、`organization`、`member`、`passkey`、`apikey` 等 13 张表）按 Better-Auth 端点与数据模型标准实现（见 ADR-0001）。

## 决策

auth 域全部主键与外键使用 **UUIDv7 字符串**（`text` 存储），生成于应用层（`Uuid::now_v7().to_string()`）。

理由：

1. **Better-Auth 协议兼容**：客户端 SDK、One Tap `userHandle`、Passkey `user.id` 等均以 UUID 语义交换用户标识。
2. **ID 可在客户端生成**：多端写入与离线合并场景不需要集中式序列分配。
3. **UUIDv7 有序性**：时间前缀保证按主键近似按时间排序，索引局部性接近自增 i64，分页遍历无性能惩罚。
4. **不泄露业务量**：避免 i64 序号暴露用户/组织增长速率。

## 数据库默认值

迁移中主键列默认值使用 **`gen_random_uuid()::text`**（PG 13+ 内置，v4）作为兜底——应用层始终显式生成 UUIDv7，该默认值仅防御手工 SQL 插入，避免引入 `uuidv7()` 的 PG 18 / 扩展依赖。

## 后果

- `AGENTS.md` 的 i64 主键规则**不适用于** `src/models/_entities/` 中 auth 域实体；新增 auth 域表沿用 UUIDv7。
- 非 auth 业务域表仍维持 `i64` 主键规范。
- 旧 `users` 表（i64 自增）数据经 `m20260919_073500_migrate_auth_data` 以 `pid`（既有 UUID）迁移至新 `user` 表，账号 ID 保持稳定。
