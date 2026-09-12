use super::*;
use crate::error;

pub struct V0;

impl Migrator for V0 {
    fn migrate(&self, _db: &surrealdb::Surreal<surrealdb::engine::local::Db>) -> error::Result<()> {
        Ok(())
    }
}
