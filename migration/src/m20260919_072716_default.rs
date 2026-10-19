use sea_orm_migration::prelude::*;

#[derive(Debug)]
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260919_072716_default"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        create_media_table(manager).await?;
        create_notifications_table(manager).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(Notifications::Table)
                    .if_exists()
                    .cascade()
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(Media::Table)
                    .if_exists()
                    .cascade()
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

async fn create_media_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(Media::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(Media::Id)
                        .big_integer()
                        .not_null()
                        .auto_increment()
                        .primary_key(),
                )
                .col(ColumnDef::new(Media::ModelType).string_len(255).not_null())
                .col(ColumnDef::new(Media::ModelId).string_len(255).not_null())
                .col(
                    ColumnDef::new(Media::Uuid)
                        .string_len(36)
                        .not_null()
                        .default(Expr::cust("gen_random_uuid()::text")),
                )
                .col(
                    ColumnDef::new(Media::CollectionName)
                        .string_len(255)
                        .not_null(),
                )
                .col(ColumnDef::new(Media::Name).string_len(255).not_null())
                .col(ColumnDef::new(Media::FileName).string_len(255).not_null())
                .col(ColumnDef::new(Media::MimeType).string_len(255).not_null())
                .col(ColumnDef::new(Media::Disk).string_len(255).not_null())
                .col(
                    ColumnDef::new(Media::ConversionsDisk)
                        .string_len(255)
                        .null(),
                )
                .col(
                    ColumnDef::new(Media::Size)
                        .big_integer()
                        .not_null()
                        .default(0),
                )
                .col(
                    ColumnDef::new(Media::ConversionStatus)
                        .string_len(50)
                        .not_null()
                        .default("completed"),
                )
                .col(ColumnDef::new(Media::OrderColumn).integer().null())
                .col(ColumnDef::new(Media::Manipulations).json_binary().null())
                .col(ColumnDef::new(Media::CustomProperties).json_binary().null())
                .col(
                    ColumnDef::new(Media::GeneratedConversions)
                        .json_binary()
                        .not_null()
                        .default(Expr::cust("'{}'::jsonb")),
                )
                .col(ColumnDef::new(Media::ResponsiveImages).json_binary().null())
                .col(
                    ColumnDef::new(Media::CreatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .col(
                    ColumnDef::new(Media::UpdatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_media_uuid")
                .table(Media::Table)
                .col(Media::Uuid)
                .unique()
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_media_model")
                .table(Media::Table)
                .col(Media::ModelType)
                .col(Media::ModelId)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_media_collection")
                .table(Media::Table)
                .col(Media::CollectionName)
                .to_owned(),
        )
        .await?;

    Ok(())
}

async fn create_notifications_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(Notifications::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(Notifications::Id)
                        .uuid()
                        .not_null()
                        .default(Expr::cust("gen_random_uuid()"))
                        .primary_key(),
                )
                .col(
                    ColumnDef::new(Notifications::NotifiableType)
                        .string_len(255)
                        .not_null(),
                )
                .col(
                    ColumnDef::new(Notifications::NotifiableId)
                        .string_len(255)
                        .not_null(),
                )
                .col(
                    ColumnDef::new(Notifications::Type)
                        .string_len(255)
                        .not_null(),
                )
                .col(ColumnDef::new(Notifications::Data).text().not_null())
                .col(
                    ColumnDef::new(Notifications::ReadAt)
                        .timestamp_with_time_zone()
                        .null(),
                )
                .col(
                    ColumnDef::new(Notifications::CreatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .col(
                    ColumnDef::new(Notifications::UpdatedAt)
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .col(
                    ColumnDef::new(Notifications::DeletedAt)
                        .timestamp_with_time_zone()
                        .null(),
                )
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_notifiable")
                .table(Notifications::Table)
                .col(Notifications::NotifiableType)
                .col(Notifications::NotifiableId)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_notifications_read_at")
                .table(Notifications::Table)
                .col(Notifications::ReadAt)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_notifications_deleted_at")
                .table(Notifications::Table)
                .col(Notifications::DeletedAt)
                .to_owned(),
        )
        .await?;

    Ok(())
}

#[derive(DeriveIden)]
enum Media {
    Table,
    Id,
    ModelType,
    ModelId,
    Uuid,
    CollectionName,
    Name,
    FileName,
    MimeType,
    Disk,
    ConversionsDisk,
    Size,
    ConversionStatus,
    OrderColumn,
    Manipulations,
    CustomProperties,
    GeneratedConversions,
    ResponsiveImages,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Notifications {
    Table,
    Id,
    NotifiableType,
    NotifiableId,
    #[sea_orm(iden = "type")]
    Type,
    Data,
    ReadAt,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}
