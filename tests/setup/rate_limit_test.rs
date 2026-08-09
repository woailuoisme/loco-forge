use axum::http::Request;
use locos::setup::middleware::rate_limit::SmartIpKeyExtractor;
use std::net::{IpAddr, Ipv4Addr};
use tower_governor::key_extractor::KeyExtractor;

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
