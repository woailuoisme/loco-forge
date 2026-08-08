use loco_rs::testing::prelude::*;
use locos::{app::App, models::posts};
use sea_orm::{ActiveModelTrait, ActiveValue, EntityTrait};
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_model() {
    let boot = boot_test::<App>().await.unwrap();

    // 插入一条 Post 记录并断言持久化结果
    let new_post = posts::ActiveModel {
        title: ActiveValue::Set(Some("Loco 最佳实践".to_string())),
        content: ActiveValue::Set(Some("通过生成器生成的测试文章".to_string())),
        ..Default::default()
    };

    let inserted = new_post
        .insert(&boot.app_context.db)
        .await
        .expect("Failed to insert post");

    assert!(inserted.id > 0, "Auto-increment id should be assigned");
    assert_eq!(inserted.title.as_deref(), Some("Loco 最佳实践"));

    // 按主键回查，校验持久化数据一致
    let fetched = posts::Entity::find_by_id(inserted.id)
        .one(&boot.app_context.db)
        .await
        .expect("Failed to fetch inserted post")
        .expect("Inserted post should exist");

    assert_eq!(fetched.title.as_deref(), Some("Loco 最佳实践"));
    assert_eq!(fetched.content.as_deref(), Some("通过生成器生成的测试文章"));
}
