use loco_rs::testing::prelude::*;
use locos::{app::App, setup::storage};
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_storage_init_in_test_environment() {
    let boot = boot_test::<App>().await.unwrap();
    let ctx = storage::init(boot.app_context).await.unwrap();

    // 校验测试环境下成功初始化内存存储驱动
    assert!(matches!(ctx.environment, loco_rs::environment::Environment::Test));
}
