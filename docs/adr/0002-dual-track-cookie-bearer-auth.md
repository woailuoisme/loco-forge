# 0002. Cookie 与 Bearer Token 双轨鉴权提取

为了同时满足 Better-Auth Web 端依赖 HttpOnly Cookie（防 XSS）和 [api.md](file:///Users/seaside/Projects/rust/locos/api.md) 第 3.1 节及移动端/微服务依赖 `Authorization: Bearer <token>` 的要求，服务端在登录成功时同时写入 Cookie 与返回 Token。
受保护路由统一通过 Axum Extractor 优先提取 Bearer Token，未提供时自动回退提取 Cookie，完美契合 Better-Auth `bearer` 插件规范。
