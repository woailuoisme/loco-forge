//! 认证相关集成测试，按子领域拆分（与 src 四层同构）。

// 说明：测试模块通过 `mod xxx;` 声明即可被 cargo test 发现，无需 pub。
mod admin;
mod api_key;
mod device_code;
mod jwt;
mod magic_link;
mod oauth2;
mod organization;
mod otp;
mod passkey;
mod session;
mod two_factor;
