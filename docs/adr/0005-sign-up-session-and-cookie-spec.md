# 0005. 注册会话签发与 Cookie 安全属性规范

用户通过邮箱或用户名注册成功后，服务端立即完成会话签发并返回 `user`、`session` 及 `token` 载荷，避免二次登录阻断用户动线。
Cookie 遵循 Better-Auth 官方命名与现代安全规范：非安全连接采用 `better-auth.session_token`，安全连接采用 `__Secure-better-auth.session_token`；强制标注 `HttpOnly`、`Path=/` 与 `SameSite=Lax`，全面防护跨站脚本与伪造攻击。
