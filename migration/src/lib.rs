#![allow(elided_lifetimes_in_paths)]
#![allow(clippy::wildcard_imports)]
#![allow(unreachable_pub)]
pub use sea_orm_migration::prelude::*;

mod m20220101_000001_users;
mod m20260729_213140_posts;
mod m20260919_072338_auth;
mod m20260919_072716_default;
mod m20260919_073500_migrate_auth_data;

#[derive(Debug)]
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_users::Migration),
            Box::new(m20260729_213140_posts::Migration),
            Box::new(m20260919_072338_auth::Migration),
            Box::new(m20260919_072716_default::Migration),
            Box::new(m20260919_073500_migrate_auth_data::Migration),
        ]
    }
}
