use apache_avro::{AvroSchema, Reader, Writer, from_value, types::Value};
use redb::{Database, ReadableDatabase, TableDefinition};
use serde::{Deserialize, Serialize};
use snafu::{OptionExt, ResultExt};
use uuid::Uuid;
use crate::{error::{self, builder}, subversion::context::LogEntry};


static DATABASE: once_cell::sync::OnceCell<Database> = once_cell::sync::OnceCell::new();

fn database() -> error::Result<&'static Database> {
    DATABASE.get_or_try_init(|| {
        let db = Database::create("svnexus.db")?;
        Ok(db)
    })
}

#[easy_ext::ext]
impl apache_avro::Schema {
    fn default_writer(&self) -> Writer<'_, Vec<u8>> {
        Writer::new(self, Vec::new())
    }

    fn value<T: Serialize>(&self, value: &T) -> error::Result<Vec<u8>> {
        let mut writer = self.default_writer();
        writer.append_ser(value)?;
        Ok(writer.into_inner()?)
    }
}

#[easy_ext::ext]
impl<'a, R: std::io::Read> apache_avro::Reader<'a, R> {
    fn one(&mut self) -> error::Result<Value> {
        let value = self.next().context(builder::General {
            detail: "None"
        })??;

        Ok(value)
    }
}

#[easy_ext::ext]
impl Value {
    fn as_type<'de, T: Deserialize<'de>>(& 'de self) -> error::Result<T> {
        let v = from_value(self)?;
        Ok(v)
    }
}

#[derive(uniffi::Record)]
pub struct DatabaseManager {}

impl DatabaseManager {
    async fn call_async<F, R>(&self, call: F) -> error::Result<R>
    where
        F: (FnOnce(&'static Database) -> error::Result<R>) + Send + 'static,
        R: Send + 'static,
    {
        let result = tokio::task::spawn_blocking(move || {
            let db = database()?;
            call(db)
        }).await.context(builder::Runtime)??;

        Ok(result)
    }
}

const REGISTRY: TableDefinition<&str, &[u8]> = TableDefinition::new("registry");

pub struct RevisionsTableEntry {
    key: u32,
    log: LogEntry
}

#[derive(AvroSchema, Serialize, Deserialize, uniffi::Record, Debug, Clone)]
pub struct RepositoryTable {
    revisions: String,
}

#[derive(AvroSchema, Serialize, Deserialize, uniffi::Record, Debug)]
pub struct RelativeLogEntry {
    entry: LogEntry,
    children: Vec<RelativeLogEntry>
}

fn entries_to_relative(entries: Vec<LogEntry>) -> Vec<RelativeLogEntry> {

    // safe but slow
    fn insert(entries: &mut [RelativeLogEntry], levels: &[usize], entry: RelativeLogEntry) -> usize {
        assert!(!levels.is_empty(), "Unexpected behavior");
        if levels.len() == 1 {
            entries[levels[0]].children.push(entry);
            entries[levels[0]].children.len() - 1
        } else {
            insert(&mut entries[levels[0]].children, &levels[1..], entry)
        }
    }

    let mut result = Vec::with_capacity(entries.len()) as Vec<RelativeLogEntry>;


    let mut parents = Vec::new() as Vec<Vec<usize>>;


    for i in entries {
        let entry = RelativeLogEntry {
            entry: i,
            children: Vec::new(),
        };
        if entry.entry.has_children {
            if let Some(last) = parents.last() {
                let pos = insert(&mut result, last, entry);
                let mut levels = last.to_vec();
                levels.push(pos);
                parents.push(levels);
            } else {
                result.push(entry);
                parents.push(vec![result.len() - 1]);
            }
        } else if entry.entry.revision.is_none() {
            parents.remove(parents.len() - 1);
        } else {
            result.push(entry);
        }
    }

    result
}

#[uniffi::export(async_runtime = "tokio")]
impl DatabaseManager {


    async fn repository_insert_history(self, name: String, entries: Vec<LogEntry>) -> error::Result<()> {
        self.call_async(move |db| {
            let lock = db.begin_write()?;

            let mut table = lock.open_table(TableDefinition::new(name.as_str()) as TableDefinition<u32, &[u8]>)?;


            let relative = entries_to_relative(entries);

            for i in relative {
                let Some(revision) = i.entry.revision else {
                    continue;
                };

                let v = RelativeLogEntry::get_schema().value(&i)?;
                table.insert(revision, v.as_slice())?;
            }


            Ok(())


        }).await
    }


    async fn repository_history(self, name: String, start: u32, end: u32) -> error::Result<Vec<RelativeLogEntry>> {
        self.call_async(move |db| {
            let lock = db.begin_read()?;
            let table = lock.open_table(TableDefinition::new(name.as_str()) as TableDefinition<u32, &[u8]>)?;

            let mut result = Vec::with_capacity((end - start).try_into().unwrap());

            let range = table.range(start..end)?;

            for i in range {
                let (_, v) = i?;
                let mut reader = Reader::new(v.value())?;

                result.push(reader.one()?.as_type::<RelativeLogEntry>()?);
            }

            Ok(result)
        }).await
    }

    async fn repository_table(self, key: String) -> error::Result<Option<RepositoryTable>> {
        self.call_async(move |db| {
            let lock = db.begin_read()?;

            let registry = lock.open_table(REGISTRY)?;

            let Some(data) = registry.get(key.as_str())? else {
                return Ok(None);
            };


            let mut reader = Reader::new(data.value())?;
            let table = reader.one()?.as_type::<RepositoryTable>()?;
            Ok(Some(table))
        }).await
    }

    async fn resgister_repository(self, key: String, table: Option<RepositoryTable>) -> error::Result<RepositoryTable> {
        self.call_async(move |db| {
            let table = table.unwrap_or_else(|| RepositoryTable { revisions: Uuid::new_v4().to_string() });
            let lock = db.begin_write()?;

            let mut registry = lock.open_table(REGISTRY)?;

            let value = RepositoryTable::get_schema().value(&table)?;

            registry.insert(key.as_str(), value.as_slice())?;

            Ok(table)
        }).await
    }
    // async fn is_repository_registered(self, key: String) -> error::Result<bool> {
    //     self.call_async(move |db| {
    //         let lock = db.begin_read()?;

    //         let registry = lock.open_table(REGISTRY)?;


    //         registry.get(key)

    //     }).await
    // }

    // async fn has_repository_table(self, key: String) -> error::Result<bool> {
    //     self.call_async(move |db| {
    //         let lock = db.begin_read()?;

    //         for i in lock.list_tables()? {
    //             if i.name() == key {
    //                 return Ok(true);
    //             }
    //         }

    //         Ok(false)

    //     }).await
    // }

    // async fn create_repository_table(self, key: String) -> error::Result<RepositoryTable> {
    //     self.call_async(move |db| {

    //         let lock = db.begin_read()?;

    //         for i in lock.list_tables()? {
    //             if i.name() == key {
    //                 return builder::General {
    //                     detail: "Already exists"
    //                 }.fail();
    //             }
    //         }


    //         let table = lock.open_table(TableDefinition::new(key.as_str()) as TableDefinition<&str, &str>)?;


    //         todo!()

    //     }).await
    // }
}
