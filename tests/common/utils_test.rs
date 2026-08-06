use locos::common::utils::{Hash, OrderNoOptions, OrderPrefix, OrderUtil};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

#[test]
fn test_hash_make_and_check() {
    let password = "MySuperSecretPassword123!";
    let hash = Hash::make(password).expect("Should generate Argon2id hash");

    // 验证导出的哈希串格式符合 Argon2id 规范
    assert!(hash.starts_with("$argon2id$"));

    // 校验正确密码匹配
    assert!(Hash::check(password, &hash));

    // 校验错误密码拒绝
    assert!(!Hash::check("WrongPassword123!", &hash));
    assert!(!Hash::check("", &hash));
}

#[rstest::rstest]
#[case("invalid-hash-string")]
#[case("$pbkdf2-sha256$i=10000$saltsalt$hash")]
fn test_hash_needs_rehash(#[case] invalid_hash: &str) {
    let password = "TestPassword888";
    let valid_hash = Hash::make(password).unwrap();

    // 合规的 Argon2id 哈希不需要 rehash
    assert!(!Hash::needs_rehash(&valid_hash));

    // 无法解析或非 Argon2id 结构的哈希需要 rehash
    assert!(Hash::needs_rehash(invalid_hash));
}

#[test]
fn test_generate_and_parse_order_no() {
    // 1. 生成默认订单号
    let order_no = OrderUtil::generate_default();
    assert!(order_no.starts_with(OrderPrefix::ORDER));
    assert_eq!(order_no.len(), 26);

    // 2. 逆向解析订单号
    let parsed = OrderUtil::parse(&order_no).expect("Should parse order no successfully");
    assert_eq!(parsed.prefix, OrderPrefix::ORDER);
    assert_eq!(parsed.machine_id, 1);
    assert!(parsed.sequence < 10000);

    // 3. 自定义前缀与机器 ID
    let pay_no = OrderUtil::generate(
        OrderPrefix::PAYMENT,
        Some(OrderNoOptions {
            machine_id: Some(8),
            sequence_digits: Some(4),
        }),
    );
    assert!(pay_no.starts_with(OrderPrefix::PAYMENT));

    let parsed_pay = OrderUtil::parse(&pay_no).expect("Should parse payment order no");
    assert_eq!(parsed_pay.prefix, OrderPrefix::PAYMENT);
    assert_eq!(parsed_pay.machine_id, 8);
}

#[rstest::rstest]
#[case("INVALID")]
#[case("ORD12345")]
#[case("")]
#[case("PAY_SHORT")]
fn test_parse_invalid_order_no(#[case] input: &str) {
    assert_eq!(OrderUtil::parse(input), None);
}

#[test]
fn test_order_no_high_concurrency_uniqueness() {
    // 模拟多线程高并发生成 1000 个订单号，验证绝对防重唯一性
    let set = Arc::new(Mutex::new(HashSet::new()));
    let handles: Vec<_> = (0..10)
        .map(|_| {
            let set_clone = Arc::clone(&set);
            std::thread::spawn(move || {
                for _ in 0..100 {
                    let no = OrderUtil::generate(OrderPrefix::ORDER, None);
                    let mut guard = set_clone.lock().unwrap();
                    assert!(guard.insert(no), "Duplicate order_no detected!");
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(set.lock().unwrap().len(), 1000);
}
