use sea_orm_migration::prelude::*;

#[derive(Debug)]
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260919_073500_migrate_auth_data"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 旧 users 表不存在（全新库）时直接跳过
        if !manager.has_table("users").await? {
            return Ok(());
        }

        // 数据搬运：users -> user / account(credential)。
        // 说明：旧 users.api_key 为明文静态串，与新 API Key 哈希体系不兼容，不予迁移（自然失效）。
        manager
            .get_connection()
            .execute_unprepared(
                r#"INSERT INTO "user" ("id", "name", "email", "email_verified", "role", "status")
                   SELECT u."pid"::text,
                          u."name",
                          u."email",
                          u."email_verified_at" IS NOT NULL,
                          u."role",
                          'active'
                   FROM "users" u"#,
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                r#"INSERT INTO "account" ("id", "user_id", "account_id", "provider_id", "password")
                   SELECT gen_random_uuid()::text,
                          u."pid"::text,
                          u."email",
                          'credential',
                          u."password"
                   FROM "users" u"#,
            )
            .await?;

        // 迁移完成后移除遗留表
        manager
            .drop_table(
                Table::drop()
                    .table(Alias::new("users"))
                    .if_exists()
                    .cascade()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // 数据迁移不可逆：旧表结构与数据无法从新模型还原
        Ok(())
    }
}
