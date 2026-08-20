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

    // 使用 Model::find_by_id 仓储方法查找
    let repo_fetched = posts::Model::find_by_id(&boot.app_context.db, inserted.id)
        .await
        .expect("Model::find_by_id should succeed");
    assert_eq!(repo_fetched.id, inserted.id);

    // 分页查询测试
    let (items, total_items) = posts::Entity::find_all_paginated(&boot.app_context.db, 1, 10)
        .await
        .expect("find_all_paginated should succeed");
    assert!(!items.is_empty(), "Should contain inserted post");
    assert!(total_items >= 1, "Total items should be at least 1");

    // 模糊搜索查询测试
    let (search_items, _) = posts::Entity::search(&boot.app_context.db, "生成器", 1, 10)
        .await
        .expect("search should succeed");
    assert_eq!(search_items.len(), 1, "Should find 1 post matching keyword");
    assert_eq!(search_items[0].id, inserted.id);

    // 仓储方法 Model::create 测试
    let created = posts::Model::create(
        &boot.app_context.db,
        Some("仓储模式测试".to_string()),
        Some("正文内容".to_string()),
    )
    .await
    .expect("Model::create should succeed");

    assert_eq!(created.title.as_deref(), Some("仓储模式测试"));

    // 仓储方法 Model::update_post 测试
    let updated = created
        .update_post(
            &boot.app_context.db,
            Some("已更新标题".to_string()),
            Some("已更新内容".to_string()),
        )
        .await
        .expect("Model::update_post should succeed");

    assert_eq!(updated.title.as_deref(), Some("已更新标题"));

    // 仓储方法 Model::delete 测试
    let delete_id = updated.id;
    updated
        .delete(&boot.app_context.db)
        .await
        .expect("Model::delete should succeed");

    assert!(
        posts::Model::find_by_id(&boot.app_context.db, delete_id)
            .await
            .is_err(),
        "Deleted post should not exist"
    );

    // 仓储方法 Entity::delete_by_ids 批量删除测试
    let p1 = posts::Model::create(&boot.app_context.db, Some("Post 1".into()), None)
        .await
        .unwrap();
    let p2 = posts::Model::create(&boot.app_context.db, Some("Post 2".into()), None)
        .await
        .unwrap();

    let deleted_count = posts::Entity::delete_by_ids(&boot.app_context.db, &[p1.id, p2.id])
        .await
        .expect("delete_by_ids should succeed");
    assert_eq!(deleted_count, 2, "Should delete 2 posts");

    // 空切片边界测试
    let empty_deleted = posts::Entity::delete_by_ids(&boot.app_context.db, &[])
        .await
        .expect("empty delete_by_ids should succeed");
    assert_eq!(empty_deleted, 0, "Empty slice should delete 0 posts");
}
