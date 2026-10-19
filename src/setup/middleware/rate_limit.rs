//! Rate Limiting 请求速率限制中间件
//!
//! 基于 `tower-governor` (Leaky Bucket / Token Bucket 算法)，提供 IP 抽取、限流配额配置与防暴力破解层。

use axum::{body::Body, http::Request};
use governor::{clock::QuantaInstant, middleware::NoOpMiddleware};
use tower_governor::{
    GovernorError, GovernorLayer, governor::GovernorConfigBuilder, key_extractor::KeyExtractor,
};

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

    fn name(&self) -> &'static str {
        "SmartIpKeyExtractor"
    }

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

        let ip = tower_governor::key_extractor::PeerIpKeyExtractor
            .extract(req)
            .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST));
        Ok(ip)
    }
}

/// 认证接口专用 Rate Limiter Layer 类型别名
pub type AuthRateLimitLayer =
    GovernorLayer<SmartIpKeyExtractor, NoOpMiddleware<QuantaInstant>, Body>;

/// 构造用于高风险认证接口 (/api/auth/*) 的 Rate Limiter Layer
/// 配额：每 500 毫秒生成 1 令牌，突发容忍量为 10 次
pub fn auth_rate_limit_layer() -> AuthRateLimitLayer {
    let config = GovernorConfigBuilder::default()
        .key_extractor(SmartIpKeyExtractor)
        .per_second(2)
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

/// 构造后台管理员登录专用的 Rate Limiter Layer
///
/// 配额：每 1 秒生成 1 令牌，突发容忍量仅 3 次。
///
/// 比 [`auth_rate_limit_layer`] 更严格 —— 后台登录是攻击面最集中的单点：
/// 成功一次即获得全量用户管理权限，故将单 IP 的密码尝试窗口压缩到 3 次。
/// 正常管理员不会在 1 秒内连点 3 次登录，无感知影响。
pub fn admin_login_rate_limit_layer() -> AuthRateLimitLayer {
    let config = GovernorConfigBuilder::default()
        .key_extractor(SmartIpKeyExtractor)
        .per_second(1)
        .burst_size(3)
        .finish()
        .expect("Failed to build admin login rate limiter config");

    GovernorLayer::new(config)
}

#[cfg(test)]
mod tests {
    use axum::http::Request;
    use std::net::{IpAddr, Ipv4Addr};
    use tower_governor::key_extractor::KeyExtractor;

    use super::*;

    #[test]
    fn test_smart_ip_key_extractor_x_forwarded_for() {
        let req = Request::builder()
            .header(
                "x-forwarded-for",
                "203.0.113.195, 70.41.3.18, 150.172.238.178",
            )
            .body(())
            .unwrap();

        let key = SmartIpKeyExtractor.extract(&req).unwrap();
        assert_eq!(key, IpAddr::V4(Ipv4Addr::new(203, 0, 113, 195)));
    }

    #[test]
    fn test_smart_ip_key_extractor_x_real_ip() {
        let req = Request::builder()
            .header("x-real-ip", "198.51.100.1")
            .body(())
            .unwrap();

        let key = SmartIpKeyExtractor.extract(&req).unwrap();
        assert_eq!(key, IpAddr::V4(Ipv4Addr::new(198, 51, 100, 1)));
    }

    #[test]
    fn test_smart_ip_key_extractor_fallback_localhost() {
        let req = Request::builder().body(()).unwrap();

        let key = SmartIpKeyExtractor.extract(&req).unwrap();
        assert_eq!(key, IpAddr::V4(Ipv4Addr::LOCALHOST));
    }
}
