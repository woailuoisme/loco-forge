//! Rate Limiting 请求速率限制中间件
//!
//! 基于 `tower-governor` (Leaky Bucket / Token Bucket 算法)，提供 IP 抽取、限流配额配置与防暴力破解层。

use axum::{body::Body, http::Request};
use governor::{clock::QuantaInstant, middleware::NoOpMiddleware};
use tower_governor::{GovernorError, GovernorLayer, governor::GovernorConfigBuilder, key_extractor::KeyExtractor};

/// 智能 IP Key 提取器 (Smart IP Key Extractor)
///
/// 优先级：
/// 1. `X-Forwarded-For` 请求头 (取第一个 IP)
/// 2. `X-Real-IP` 请求头
/// 3. TCP Connect Peer Address 物理连接 IP
#[derive(Clone, Copy, Debug)]
pub struct SmartIpKeyExtractor;

impl KeyExtractor for SmartIpKeyExtractor {
    type Key = std::net::IpAddr;

    fn extract<B>(&self, req: &Request<B>) -> Result<Self::Key, GovernorError> {
        // 步骤 1: 尝试解析 X-Forwarded-For 请求头
        if let Some(ip) = req
            .headers()
            .get("x-forwarded-for")
            .and_then(|h| h.to_str().ok())
            .and_then(|val| val.split(',').next())
            .and_then(|first_ip| first_ip.trim().parse().ok())
        {
            return Ok(ip);
        }

        // 步骤 2: 尝试解析 X-Real-IP 请求头
        if let Some(ip) = req
            .headers()
            .get("x-real-ip")
            .and_then(|h| h.to_str().ok())
            .and_then(|val| val.trim().parse().ok())
        {
            return Ok(ip);
        }

        // 步骤 3: 降级尝试解析物理连接 Peer IP（在测试框架或内存 Mock 请求下如无法提取，降级保底为 127.0.0.1）
        match tower_governor::key_extractor::PeerIpKeyExtractor.extract(req) {
            Ok(ip) => Ok(ip),
            Err(_) => Ok("127.0.0.1".parse().unwrap()),
        }
    }
}

/// 认证接口专用 Rate Limiter Layer 类型别名
pub type AuthRateLimitLayer = GovernorLayer<SmartIpKeyExtractor, NoOpMiddleware<QuantaInstant>, Body>;

/// 构造用于高风险认证接口 (/api/auth/*) 的 Rate Limiter Layer
/// 配额：每 12 秒生成 1 令牌 (平均 5 次/分钟)，突发容忍量为 5 次
pub fn auth_rate_limit_layer() -> AuthRateLimitLayer {
    let config = GovernorConfigBuilder::default()
        .key_extractor(SmartIpKeyExtractor)
        .per_second(12)
        .burst_size(10)
        .finish()
        .expect("Failed to build auth rate limiter config");

    GovernorLayer::new(config)
}

/// 构造用于常规 API 接口的 Rate Limiter Layer
/// 配额：每 1 秒生成 1 令牌 (平均 60 次/分钟)，突发容忍量为 20 次
pub fn api_rate_limit_layer() -> AuthRateLimitLayer {
    let config = GovernorConfigBuilder::default()
        .key_extractor(SmartIpKeyExtractor)
        .per_second(1)
        .burst_size(20)
        .finish()
        .expect("Failed to build api rate limiter config");

    GovernorLayer::new(config)
}
