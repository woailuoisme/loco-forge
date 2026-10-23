use loco_rs::{task, testing::prelude::*};
use locos::app::App;

use loco_rs::boot::run_task;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_can_run_jwt_secret() {
    let boot = boot_test::<App>().await.unwrap();

    run_task::<App>(
        &boot.app_context,
        Some(&"jwt:secret".to_string()),
        &task::Vars::default(),
    )
    .await
    .unwrap();
}
