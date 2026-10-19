# 0003. 弃用 SQLite，集成测试全面采用 PostgreSQL 独立测试库

由于认证与用户中心模式深度使用了 PostgreSQL 18 原生特性（包括单调递增 UUIDv7 函数、JSONB 扩展索引与特定约束），SQLite 方言无法完整且保真地支持这套模式。
我们决定测试环境彻底废除 SQLite，统一使用独立的 PostgreSQL 数据库 `locos_test`，并在 `.config/nextest.toml` 中配置单线程串行执行与测试初始化自动迁移与清空，实现极致的测试保真度与执行效率。
