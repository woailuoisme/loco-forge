# 0001. 认证路由采用版本化 /v1 前缀与统一响应结构

> **状态：已被 [0007](./0007-restful-endpoint-refactor.md) 取代（Superseded）。**
> 第 1 条「/v1/auth/* 前缀」与 Better-Auth 动作风格路径已废弃，其余决策（单根响应结构）仍然有效。

## 决策

1. **路径前缀**：认证路由统一挂载于 `/v1/auth/*`，而非 Better-Auth 官方默认的 `/api/auth`。
2. **登录响应结构**：`sign-in/*` 系列返回**单一根结构** `{token?, user?, session?, twoFactorRedirect?}`，
   而非 Better-Auth 客户端的 `#[serde(untagged)]` 多态结构。

## 背景与取舍

早期曾采用 `/api/auth` 前缀并保留 Better-Auth 官方多态响应，以换取前端 SDK
（如 `@better-auth/react`）零配置接入。但该方案与 [api.md](../../api.md) 存在两处冲突：

| 冲突点 | api.md 要求 | Better-Auth 默认行为 |
| --- | --- | --- |
| 版本管理 | URI 路径版本控制（`/{version}/{resources}`） | 无版本前缀（`/api/auth`） |
| 响应结构 | 同一端点根结构必须唯一（§8.1 反模式红线） | `untagged` 枚举按条件产生两种根结构 |

## 结论

以 `api.md` 规范为优先：

- 认证路由改为 `/v1/auth/*`，与文件接口 `/v1/files/*` 统一在 `/v1` 版本维度下；
- 登录响应改为单一结构，2FA 分支通过顶层 `twoFactorRedirect` 字段判别（字段存在即为挑战分支）；
- auth 子路径结构（`sign-up/email`、`two-factor/enable`、`passkey/*` 等）保持 Better-Auth 原样，
  仅变更前缀，以便未来以自定义 `basePath` 方式重新对接；
- 基础设施入口（`/`、`/livez`、`/readyz`、`/docs`、`/redoc`、`/api-docs/openapi.json`）不纳入版本管理。

## 后果

- **失去**：Better-Auth 官方前端 SDK 无法零配置直接对接——需自定义 `basePath`，
  并适配登录响应的扁平化字段解析（不再依赖 `untagged` 判别）。
- **获得**：全站 URL 版本策略统一；登录响应根结构可预测，符合 `api.md` §8.1
  对「不可预测的多变响应结构」的禁令。

## 相关

- [0002. 双轨 Cookie / Bearer 认证](0002-dual-track-cookie-bearer-auth.md)
- [0005. 注册会话与 Cookie 规范](0005-sign-up-session-and-cookie-spec.md)
