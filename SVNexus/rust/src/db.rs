use std::sync::Arc;

use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};
use snafu::ResultExt;
use uuid::Uuid;
use crate::{error::{self, builder}, subversion::context::LogEntry};


static DATABASE: once_cell::sync::OnceCell<Database> = once_cell::sync::OnceCell::new();

fn database() -> error::Result<&'static Database> {
    DATABASE.get_or_try_init(|| {
        let db = Database::create("svnexus.db")?;
        Ok(db)
    })
}

#[derive(uniffi::Record)]
pub struct DatabaseManager {}

impl DatabaseManager {
    async fn call_async<F, R>(self, call: F) -> error::Result<R>
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

// const REGISTRY: TableDefinition<&str, &str> = TableDefinition::new("registry");

// const GLOBAL: TableDefinition<&str, &str> = TableDefinition::new("global");


#[derive(Serialize, Deserialize, Default, uniffi::Record, Debug, Clone)]
pub struct GlobalSettings {
    default_username: Option<String>,
    default_password: Option<String>
}

#[derive(Serialize, Deserialize, uniffi::Enum, Debug, Clone)]
pub enum WorkspaceHistory {
    WorkingCopy {
        working_copy_root: String,
        working_copy_path: String,
        repository_root_url: Option<String>,
        last_used_time: Option<i64>,
        checkout: bool,
        order: i32,
        star: bool,
        id: String,
        remark: Option<String>,
    },
    Repository {
        root_url: String,
        repository_uuid: String,
        last_used_time: Option<i64>,
        order: i32,
        star: bool,
        id: String,
        remark: Option<String>,
    }
}

impl WorkspaceHistory {
    fn uuid(&self) -> &str {
        match self {
            WorkspaceHistory::WorkingCopy { id, .. } => id,
            WorkspaceHistory::Repository { id, .. } => id,
        }
    }
}

pub struct SettingsTable;
impl SettingsTable {
    pub const TABLE_NAME: &'static str = "Settings";
    pub const TABLE: TableDefinition<'static, &str, &str> = TableDefinition::new(Self::TABLE_NAME);
    pub const SETTINGS_KEY: &'static str = "setting";
}

pub struct HistoryGroup {
    pub name: String,
    pub children: Vec<String>
}

pub struct HistoryTable;

impl HistoryTable {
    pub const TABLE_NAME: &'static str = "History";
    pub const TABLE: TableDefinition<'static, &str, &str> = TableDefinition::new(Self::TABLE_NAME);

    pub const HISTORY_KEY: &'static str = "history";

    pub const HISTORY_GROUPS_KEY: &'static str = "history_groups";
}

pub struct RegistryTable;
impl RegistryTable {
    pub const TABLE_NAME: &'static str = "Registry";
    pub const TABLE: TableDefinition<'static, &str, &str> = TableDefinition::new(Self::TABLE_NAME);

}

#[derive(Serialize, Deserialize, uniffi::Record, Debug, Clone)]
pub struct RepositoryTable {
    revisions: String,
}

#[derive(Serialize, Deserialize, uniffi::Record, Debug)]
pub struct RelativeLogEntry {
    entry: LogEntry,
    children: Vec<RelativeLogEntry>
}

#[easy_ext::ext(JsonExtension)]
pub impl<T> T {
    fn as_json(&self) -> error::Result<String>
        where Self: Serialize
    {
        let v = serde_json::to_string(self)?;
        Ok(v)
    }
    fn from_json<'de>(json: &'de str) -> error::Result<Self>
        where Self: Deserialize<'de>
    {
        let v = serde_json::from_str(json)?;
        Ok(v)
    }
}

#[uniffi::export(with_foreign)]
pub trait WorkspaceHistoryUpdateOperation: Send + Sync + 'static {
    fn update(&self, v: WorkspaceHistory) -> WorkspaceHistory;
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

    async fn settings(self) -> error::Result<GlobalSettings> {
        self.call_async(move |db| {
            let lock = db.begin_write()?;
            let table = lock.open_table(SettingsTable::TABLE)?;

            let Some(value) = table.get(SettingsTable::SETTINGS_KEY)? else {
                return Ok(Default::default());
            };

            Ok(GlobalSettings::from_json(value.value())?)

        }).await
    }

    async fn set_settings(self, settings: GlobalSettings) -> error::Result<()> {
        self.call_async(move |db| {
            let lock = db.begin_write()?;
            let mut table = lock.open_table(SettingsTable::TABLE)?;

            table.insert(SettingsTable::SETTINGS_KEY, settings.as_json()?.as_str())?;


            Ok(())
        }).await
    }

    async fn delete_workspace_history(self, uuid: String) -> error::Result<()> {
        self.call_async(move |db| {
            let lock = db.begin_write()?;
            {
                let mut table = lock.open_table(HistoryTable::TABLE)?;

                let mut old_histories = vec![];

                if let Some(value) = table.get(HistoryTable::HISTORY_KEY)? {
                    old_histories = serde_json::from_str::<Vec<WorkspaceHistory>>(value.value())?;
                }

                old_histories.retain(|h| h.uuid() != uuid);

                table.insert(HistoryTable::HISTORY_KEY, old_histories.as_json()?.as_str())?;
            }

            lock.commit()?;

            Ok(())
        }).await
    }

    async fn update_workspace_history(self, uuid: String, operation: Arc<dyn WorkspaceHistoryUpdateOperation>) -> error::Result<()> {
        self.call_async(move |db| {
            let lock = db.begin_write()?;
            {
                let mut table = lock.open_table(HistoryTable::TABLE)?;

                let mut old_histories = vec![];

                if let Some(value) = table.get(HistoryTable::HISTORY_KEY)? {
                    old_histories = serde_json::from_str::<Vec<WorkspaceHistory>>(value.value())?;
                }

                for i in old_histories.iter_mut() {
                    if i.uuid() == uuid {
                        *i = operation.update(i.clone());
                        break;
                    }
                }

                table.insert(HistoryTable::HISTORY_KEY, old_histories.as_json()?.as_str())?;
            }

            lock.commit()?;

            Ok(())
        }).await
    }

    async fn set_workspace_history(self, history: WorkspaceHistory) -> error::Result<()> {
        self.call_async(move |db| {
            let lock = db.begin_write()?;
            {
                let mut table = lock.open_table(HistoryTable::TABLE)?;

                let mut old_histories = vec![];

                if let Some(value) = table.get(HistoryTable::HISTORY_KEY)? {
                    old_histories = serde_json::from_str::<Vec<WorkspaceHistory>>(value.value())?;
                }

                for i in old_histories.iter_mut() {
                    if i.uuid() == history.uuid() {
                        *i = history;
                        break;
                    }
                }

                table.insert(HistoryTable::HISTORY_KEY, old_histories.as_json()?.as_str())?;
            }

            lock.commit()?;

            Ok(())
        }).await
    }

    async fn insert_workspace_histories(self, histories: Vec<WorkspaceHistory>) -> error::Result<()> {
        self.call_async(move |db| {
            let lock = db.begin_write()?;

            {
                let mut table = lock.open_table(HistoryTable::TABLE)?;
                let mut old_histories = vec![];

                if let Some(value) = table.get(HistoryTable::HISTORY_KEY)? {
                    old_histories = serde_json::from_str::<Vec<WorkspaceHistory>>(value.value())?;
                }



                tracing::info!("Insert history: {histories:#?}");

                old_histories.extend(histories);


                table.insert(HistoryTable::HISTORY_KEY, old_histories.as_json()?.as_str())?;
            }
            lock.commit()?;

            Ok(())
        }).await
    }


    async fn workspace_histories(self) -> error::Result<Vec<WorkspaceHistory>> {

        self.call_async(move |db| {

            let lock = db.begin_write()?;

            let table = lock.open_table(HistoryTable::TABLE)?;

            let Some(history) = table.get(HistoryTable::HISTORY_KEY)? else {
                return Ok(vec![]);
            };


            let history = serde_json::from_str::<Vec<WorkspaceHistory>>(history.value())?;



            Ok(history)

        }).await

    }


    async fn insert_repository_history(self, name: String, entries: Vec<LogEntry>) -> error::Result<()> {
        self.call_async(move |db| {
            let lock = db.begin_write()?;

            {
                let mut table = lock.open_table(TableDefinition::new(name.as_str()) as TableDefinition<u32, &str>)?;


                let relative = entries_to_relative(entries);

                for i in relative {
                    let Some(revision) = i.entry.revision else {
                        continue;
                    };

                    let v = i.as_json()?;
                    table.insert(revision, v.as_str())?;
                }

            }
            lock.commit()?;

            Ok(())


        }).await
    }


    async fn repository_history(self, name: String, start: u32, end: u32) -> error::Result<Vec<RelativeLogEntry>> {
        self.call_async(move |db| {
            let lock = db.begin_write()?;
            let table = lock.open_table(TableDefinition::new(name.as_str()) as TableDefinition<u32, &str>)?;

            let mut result = Vec::with_capacity((end - start).try_into().unwrap());

            let range = table.range(start..end)?;

            for i in range {
                let (_, v) = i?;

                let v: RelativeLogEntry = serde_json::from_str(v.value())?;

                result.push(v);
            }

            Ok(result)
        }).await
    }

    async fn repository_table(self, key: String) -> error::Result<Option<RepositoryTable>> {
        self.call_async(move |db| {
            let lock = db.begin_read()?;

            let registry = lock.open_table(RegistryTable::TABLE)?;

            let Some(data) = registry.get(key.as_str())? else {
                return Ok(None);
            };


            // let mut reader = Reader::new(data.value())?;
            let v: RepositoryTable = serde_json::from_str(data.value())?;
            // let table = reader.one()?.as_type::<RepositoryTable>()?;
            Ok(Some(v))
        }).await
    }

    async fn resgister_repository(self, key: String, table: Option<RepositoryTable>) -> error::Result<RepositoryTable> {
        self.call_async(move |db| {
            let table = table.unwrap_or_else(|| RepositoryTable { revisions: Uuid::new_v4().to_string() });
            let lock = db.begin_write()?;

            let mut registry = lock.open_table(RegistryTable::TABLE)?;

            // let value = RepositoryTable::get_schema().value(&table)?;
            //

            let value = table.as_json()?;

            registry.insert(key.as_str(), value.as_str())?;

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
