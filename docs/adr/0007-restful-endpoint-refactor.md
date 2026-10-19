# 0007. v1 接口全面 RESTful 资源化（取代 0001 的路径约定）

## 状态

已采纳（Accepted）。取代 [0001](./0001-better-auth-endpoints-and-api-standards.md) 的路径前缀与命名约定。

## 背景

`api.md` §2.1 要求资源路径使用复数名词、全小写短横线且不含动词。旧实现沿用 Better-Auth
动作风格（`/v1/auth/sign-in/email`、`/v1/auth/api-key/create`、`/v1/auth/organization/create`…），
约 70 个端点全部违反该约定，且 `{"success": true}`、`200 null` 等返回形态与 §5 冲突。

## 决策

1. **取消 `/v1/auth` 前缀**，按资源重新归属：`/v1/users`、`/v1/sessions`、`/v1/access-tokens`、
   `/v1/organizations`、`/v1/members`、`/v1/invitations`、`/v1/passkeys`、`/v1/api-keys`、
   `/v1/verification-codes`、`/v1/device-authorizations`。
2. **路径不含动词**：`/v1/sessions/{id}`（DELETE）取代 `/v1/revoke-session`；
   `PATCH /v1/users/me/password` 取代 `/v1/email-otp/reset-password`。
3. **直接破坏 v1，不引入 `/v2`**：前后端同步切换，`api.md` §8.1 已注明本次为一次性例外。
4. **错误响应扁平化**：`{code, message, errors?}`，`code` 取值对齐 `api.md` §5.2 清单。
5. **无响应体操作返回 204**，创建资源返回 201。

## 取舍

- **保留 Rust 符号与模块结构**：仅改 `#[utoipa::path]`、`routes()` 与测试断言，handler 函数名、
  模块路径、`openapi.rs` 的 `paths(...)` 列表零改动，把改造爆炸半径压到 URL 表面。
- **重复语义的动作端点合并为同一路径的多方法**：如 `/v1/passkeys` 上同时挂
  POST（注册校验）/ GET（列表）/ DELETE（删除）/ PATCH（更新）。
- **无法合并的动作拆为独立名词资源**：`/v1/device-authorization-approvals` 与
  `/v1/device-authorization-denials`，避免同路径同方法冲突。
- **管理端与认证域的用户列表分属不同资源**：管理后台为 `/v1/admin/users`，
  认证域为 `/v1/admin/accounts`，避免路由重叠。

## 后果

- 所有既有客户端需同步改造路径与请求方法。
- `tests/requests/**` 中全部路径断言随之更新。
- 新增端点必须遵循 §2.1，不得再出现动词路径。
