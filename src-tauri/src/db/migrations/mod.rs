use crate::{
    error,
};

mod v0;

pub trait Migrator {
    fn migrate(&self, db: &surrealdb::Surreal<surrealdb::engine::local::Db>) -> error::Result<()>;
}

pub async fn execute(db: &surrealdb::Surreal<surrealdb::engine::local::Db>) -> error::Result<()> {
    error::ok(())
}
