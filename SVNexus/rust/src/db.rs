use std::{collections::HashMap, str::FromStr, sync::Arc};

use crate::{AnyValue, extensions::*};

use crate::{
    entities::*,
    error::{self, builder},
    subversion::context::{LogChangedPathEntry, LogEntry},
};
use migration::MigratorTrait;
use sea_orm::{ActiveModelTrait, ConnectOptions};
use sea_orm::{
    ActiveValue::{self},
    ColumnTrait, DatabaseConnection, EntityTrait, ModelTrait, QueryFilter, QueryOrder, QuerySelect,
    TransactionTrait,
};
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

#[derive(uniffi::Object, Debug)]
pub struct SeaDatabaseConnection {
    connection: DatabaseConnection,
}

impl SeaDatabaseConnection {
    fn entities_to_entries(
        &self,
        entities: Vec<(
            log_entry::Model,
            Vec<property::Model>,
            Vec<log_changed_path_entry::Model>,
        )>,
    ) -> error::Result<Vec<IndexedLogEntry>> {
        let mut result = Vec::with_capacity(entities.len());
        for (model, properties, changes) in entities {
            let mut revision_properties = HashMap::new();

            for property in properties {
                revision_properties.insert(property.name, property.value);
            }

            let mut changed_path_entries = HashMap::new();
            for change in changes {
                changed_path_entries.insert(
                    change.path,
                    LogChangedPathEntry {
                        action: change
                            .action
                            .as_str()
                            .try_into()
                            .context(builder::EnumParse {
                                detail: "Failed to parse LogChangedPathAction",
                            })?,
                        copy_from_path: change.copy_from_path,
                        copy_from_revision: change
                            .copy_from_revision
                            .map(|v| v.try_into())
                            .transpose()
                            .any_context("Invalid revision")?,
                        node_kind: change.node_kind.as_str().try_into().context(
                            builder::EnumParse {
                                detail: format!("Failed to parse NodeKind"),
                            },
                        )?,
                        text_modified: change.text_modified,
                        props_modified: change.properties_modified,
                    },
                );
            }

            let entry = LogEntry {
                revision: model
                    .revision
                    .map(|v| v.try_into())
                    .transpose()
                    .any_context("Invalid revision")?,
                date: model.date,
                author: model.author,
                message: model.message,
                revision_properties: if revision_properties.is_empty() {
                    None
                } else {
                    Some(revision_properties)
                },
                changed_path_entries: changed_path_entries,
                has_children: model.has_children,
                non_inheritable: model.non_inheritable,
                subtractive_merge: model.subtractive_merge,
            };
            result.push(IndexedLogEntry {
                id: model.id,
                entry,
            });
        }
        Ok(result)
    }
}

#[derive(uniffi::Record, Clone, Debug)]
pub struct IndexedLogEntry {
    pub id: i64,
    pub entry: LogEntry,
}

#[uniffi::export(async_runtime = "tokio")]
impl SeaDatabaseConnection {
    #[uniffi::constructor]
    pub async fn create() -> error::Result<Self> {
        use sea_orm::sqlx::sqlite::{SqliteJournalMode, SqliteSynchronous};
        use std::time::Duration;
        let mut opt = ConnectOptions::new("sqlite://svnexus.db?mode=rwc");
        opt.sqlx_logging(false);
        opt.map_sqlx_sqlite_opts(|o| {
            o.journal_mode(SqliteJournalMode::Wal)
                .synchronous(SqliteSynchronous::Normal)
                .busy_timeout(Duration::from_secs(5))
                .statement_cache_capacity(512)
                .pragma("cache_size", "-32768") // 32 MiB / connection
                .pragma("temp_store", "MEMORY")
                .pragma("mmap_size", "268435456") // 256 MiB
                .optimize_on_close(true, 400)
        });

        let connection = sea_orm::Database::connect(opt).await?;
        migration::Migrator::up(&connection, None).await?;
        Ok(Self { connection })
    }

    pub async fn truncate_repository_logs(&self, repository_uuid: &str) -> error::Result<()> {
        log_entry::Entity::delete_many()
            .filter(log_entry::Column::RepositoryUuid.eq(repository_uuid))
            .exec(&self.connection)
            .await?;
        Ok(())
    }

    pub async fn repository_logs_with_revisions(&self, repository_uuid: &str, revisions: Vec<u32>) -> error::Result<Vec<IndexedLogEntry>> {
        let logs = log_entry::Entity::find()
            .filter(log_entry::Column::RepositoryUuid.eq(repository_uuid))
            .filter(log_entry::Column::Revision.is_in(revisions))
            .all(&self.connection)
            .await?;

        let mut entries = Vec::with_capacity(logs.len());

        for i in logs {
            let properties = property::Entity::find()
                .filter(property::Column::LogEntryId.eq(i.id))
                .all(&self.connection)
                .await?;

            let changed_paths = log_changed_path_entry::Entity::find()
                .filter(log_changed_path_entry::Column::LogEntryId.eq(i.id))
                .all(&self.connection)
                .await?;

            entries.push((i, properties, changed_paths));
        }

        self.entities_to_entries(entries)
    }

    pub async fn insert_repository_logs(
        &self,
        tail: bool,
        repository_uuid: String,
        entries: Vec<LogEntry>,
    ) -> error::Result<Vec<IndexedLogEntry>> {
        self.connection
            .transaction(move |transaction| {
                Box::pin(async move {
                    let mut primary_id = ActiveValue::NotSet;

                    if !tail {
                        let id: Option<i64> = log_entry::Entity::find()
                            .select_only()
                            .column_as(log_entry::Column::Id.min(), "min_id")
                            .into_tuple()
                            .one(transaction)
                            .await?;
                        primary_id.set_value(id.map(|id| id - 1).unwrap_or_default());
                    }

                    let mut result = Vec::with_capacity(entries.len());

                    for entry in entries {
                        let mut model = log_entry::ActiveModel::default_value();

                        model.id = primary_id.clone();
                        model.repository_uuid.set_value(repository_uuid.clone());
                        model.revision.set_value(entry.revision.inner_into());
                        model.author.set_value(entry.author.clone());
                        model.date.set_value(entry.date);

                        model.message.set_value(entry.message.clone());
                        model.has_children.set_value(entry.has_children);
                        model.non_inheritable.set_value(entry.non_inheritable);
                        model.subtractive_merge.set_value(entry.subtractive_merge);

                        let insert_result =
                            log_entry::Entity::insert(model).exec(transaction).await?;

                        if let Some(properties) = entry.revision_properties.clone() {
                            for (key, value) in properties {
                                let mut property_model = property::ActiveModel::default_value();
                                property_model
                                    .log_entry_id
                                    .set_value(Some(insert_result.last_insert_id));
                                property_model.name.set_value(key);
                                property_model.value.set_value(value);
                                property::Entity::insert(property_model)
                                    .exec(transaction)
                                    .await?;
                            }
                        }

                        for (path, change) in entry.changed_path_entries.clone() {
                            let mut changed_path_model =
                                log_changed_path_entry::ActiveModel::default_value();
                            changed_path_model.path.set_value(path);
                            changed_path_model
                                .action
                                .set_value(change.action.to_string());
                            changed_path_model
                                .copy_from_path
                                .set_value(change.copy_from_path);
                            changed_path_model.copy_from_revision.set_value(
                                change.copy_from_revision.map(|e| e.try_into().unwrap()),
                            );
                            changed_path_model
                                .node_kind
                                .set_value(change.node_kind.to_string());

                            changed_path_model
                                .text_modified
                                .set_value(change.text_modified);

                            changed_path_model
                                .properties_modified
                                .set_value(change.props_modified);

                            changed_path_model
                                .log_entry_id
                                .set_value(insert_result.last_insert_id);
                            log_changed_path_entry::Entity::insert(changed_path_model)
                                .exec(transaction)
                                .await?;
                        }

                        if let ActiveValue::Set(value) = primary_id {
                            primary_id.set_value(value - 1);
                        }

                        result.push(IndexedLogEntry {
                            id: insert_result.last_insert_id,
                            entry,
                        });
                    }
                    error::ok(result)
                })
            })
            .await
            .map_err(Into::into)
    }

    pub async fn delete_repository_logs(
        &self,
        repository_uuid: &str,
        ids: Vec<i64>,
    ) -> error::Result<u64> {
        if ids.is_empty() {
            return Ok(0);
        }

        let result = log_entry::Entity::delete_many()
            .filter(log_entry::Column::RepositoryUuid.eq(repository_uuid))
            .filter(log_entry::Column::Id.is_in(ids))
            .exec(&self.connection)
            .await?;

        Ok(result.rows_affected)
    }

    pub async fn repository_logs(
        &self,
        repository_uuid: &str,
        start: Option<u32>,
        limit: Option<u32>,
        reverse: bool,
    ) -> error::Result<Vec<IndexedLogEntry>> {
        let logs = log_entry::Entity::find()
            .filter(log_entry::Column::RepositoryUuid.eq(repository_uuid))
            .if_some(start, |this, start| {
                this.filter(log_entry::Column::Revision.lte(start))
            })
            // .find_also_related(property::Entity)
            // .find_also_related(log_changed_path_entry::Entity)
            .if_or(
                reverse,
                |this| this.order_by_desc(log_entry::Column::Revision),
                |this| this.order_by_asc(log_entry::Column::Revision),
            )
            .limit(limit.map(|v| v as u64))
            // .consolidate()
            .all(&self.connection)
            .await?;

        let mut entries = Vec::with_capacity(logs.len());

        for i in logs {
            let properties = property::Entity::find()
                .filter(property::Column::LogEntryId.eq(i.id))
                .all(&self.connection)
                .await?;

            let changed_paths = log_changed_path_entry::Entity::find()
                .filter(log_changed_path_entry::Column::LogEntryId.eq(i.id))
                .all(&self.connection)
                .await?;

            entries.push((i, properties, changed_paths));
        }

        tracing::info!("Got entries size: {} limit: {:?}", entries.len(), limit);

        self.entities_to_entries(entries)
    }

    pub async fn add_workspace_histories(
        &self,
        histories: Vec<WorkspaceHistory>,
    ) -> error::Result<()> {
        let histories = histories
            .into_iter()
            .map(|history| workspace_history::ActiveModel::from(history))
            .collect::<Vec<_>>();

        workspace_history::Entity::insert_many(histories)
            .exec(&self.connection)
            .await?;

        Ok(())
    }

    pub async fn add_workspace_history(&self, history: WorkspaceHistory) -> error::Result<()> {
        let model = workspace_history::ActiveModel::from(history);

        model.insert(&self.connection).await?;

        Ok(())
    }

    pub async fn delete_workspace_history(&self, uuid: &str) -> error::Result<()> {
        let model = workspace_history::Entity::find()
            .filter(workspace_history::Column::Uuid.eq(uuid))
            .one(&self.connection)
            .await?;

        if let Some(model) = model {
            model.delete(&self.connection).await?;
        }

        Ok(())
    }

    pub async fn update_workspace_history(
        &self,
        uuid: &str,
        operation: Arc<dyn UpdateOperation>,
    ) -> error::Result<WorkspaceHistory> {
        let model = workspace_history::Entity::find()
            .filter(workspace_history::Column::Uuid.eq(uuid))
            .one(&self.connection)
            .await?
            .with_any_context(|| "Failed to find record")?;

        let id = model.id;
        let v = WorkspaceHistory::try_from(model)?
            .into_any_value()
            .into_arc();

        let v = operation.update(v)?;

        let v = v
            .to_workspace_history()
            .any_context("Unexpected value return")?;

        let model = workspace_history::ActiveModel::from(v.clone()).also_build(|mut this| {
            this.id.set_value(id);
            this
        });

        model.update(&self.connection).await?;

        Ok(v)
    }

    pub async fn workspace_histories(&self) -> error::Result<Vec<WorkspaceHistory>> {
        let models = workspace_history::Entity::find()
            .all(&self.connection)
            .await?;

        let mut result = Vec::with_capacity(models.len());

        for i in models {
            result.push(WorkspaceHistory::try_from(i)?);
        }

        Ok(result)
    }

    pub async fn add_history_group(&self, group: WorkspaceHistoryGroup) -> error::Result<()> {
        let model = workspace_history_group::ActiveModel::from(group);

        model.insert(&self.connection).await?;

        Ok(())
    }

    pub async fn update_history_group(
        &self,
        uuid: &str,
        operation: Arc<dyn UpdateOperation>,
    ) -> error::Result<WorkspaceHistoryGroup> {
        let model = workspace_history_group::Entity::find_by_uuid(uuid)
            .one(&self.connection)
            .await?
            .with_any_context(|| "Failed to find history group")?;

        let id = model.id;
        let value = WorkspaceHistoryGroup::from(model);

        let v = operation.update(value.into_any_value().into_arc())?;

        let v = v
            .to_workspace_history_group()
            .any_context("Unexpected value return")?;

        let model = workspace_history_group::ActiveModel::from(v.clone()).also_build(|mut this| {
            this.id.set_value(id);
            this
        });

        model.update(&self.connection).await?;

        Ok(v)
    }

    pub async fn delete_history_group(&self, uuid: &str) -> error::Result<()> {
        if let Some(model) = workspace_history_group::Entity::find_by_uuid(uuid)
            .one(&self.connection)
            .await?
        {
            model.delete(&self.connection).await?;
        }
        Ok(())
    }

    pub async fn history_groups(&self) -> error::Result<Vec<WorkspaceHistoryGroup>> {
        let groups = workspace_history_group::Entity::find()
            .all(&self.connection)
            .await?;

        if groups.is_empty() {
            return Ok(vec![]);
        }

        let result = groups
            .into_iter()
            .map(|i| WorkspaceHistoryGroup {
                uuid: i.uuid,
                name: i.name,
                children: {
                    if i.children.is_none() {
                        vec![]
                    } else {
                        i.children
                            .unwrap()
                            .split(',')
                            .map(|s| s.to_string())
                            .collect()
                    }
                },
            })
            .collect();

        Ok(result)
    }
}

// static DATABASE: once_cell::sync::OnceCell<Database> = once_cell::sync::OnceCell::new();

// fn database() -> error::Result<&'static Database> {
//     DATABASE.get_or_try_init(|| {
//         let db = Database::create("svnexus.db")?;
//         Ok(db)
//     })
// }

// #[derive(uniffi::Record)]
// pub struct DatabaseManager {}

// impl DatabaseManager {
//     async fn call_async<F, R>(self, call: F) -> error::Result<R>
//     where
//         F: (FnOnce(&'static Database) -> error::Result<R>) + Send + 'static,
//         R: Send + 'static,
//     {
//         let result = tokio::task::spawn_blocking(move || {
//             let db = database()?;
//             call(db)
//         })
//         .await
//         .context(builder::Runtime)??;

//         Ok(result)
//     }
// }

// const REGISTRY: TableDefinition<&str, &str> = TableDefinition::new("registry");

// const GLOBAL: TableDefinition<&str, &str> = TableDefinition::new("global");

#[derive(Serialize, Deserialize, Default, uniffi::Record, Debug, Clone)]
pub struct GlobalSettings {
    default_username: Option<String>,
    default_password: Option<String>,
}

#[derive(Serialize, Deserialize, uniffi::Enum, Debug, Clone, strum::EnumDiscriminants)]
#[strum_discriminants(name(WorkspaceHistoryKind))]
#[strum_discriminants(derive(strum::Display, strum::EnumString))]
pub enum WorkspaceHistory {
    WorkingCopy {
        working_copy_root: String,
        working_copy_path: String,
        repository_root_url: Option<String>,
        last_used_time: Option<i64>,
        checkout: bool,
        order: i32,
        star: bool,
        uuid: String,
        remark: Option<String>,
    },
    Repository {
        repository_root_url: String,
        repository_uuid: String,
        last_used_time: Option<i64>,
        order: i32,
        star: bool,
        uuid: String,
        remark: Option<String>,
    },
}

impl From<WorkspaceHistory> for workspace_history::ActiveModel {
    fn from(value: WorkspaceHistory) -> Self {
        let mut model = Self::default_value();
        model
            .kind
            .set_value(WorkspaceHistoryKind::from(&value).to_string());

        match value {
            WorkspaceHistory::WorkingCopy {
                working_copy_root,
                working_copy_path,
                repository_root_url,
                last_used_time,
                checkout,
                order,
                star,
                uuid,
                remark,
            } => {
                model.uuid.set_value(uuid);
                model.last_used_time.set_value(last_used_time);
                model.star.set_value(star);
                model.remark.set_value(remark);
                model.repository_root_url.set_value(repository_root_url);
                model.working_copy_path.set_value(Some(working_copy_path));
                model.working_copy_root.set_value(Some(working_copy_root));
                model.checkout.set_value(checkout);

                model
            }
            WorkspaceHistory::Repository {
                repository_root_url,
                repository_uuid,
                last_used_time,
                order,
                star,
                uuid,
                remark,
            } => {
                model
                    .repository_root_url
                    .set_value(Some(repository_root_url));
                model.repository_uuid.set_value(Some(repository_uuid));
                model.last_used_time.set_value(last_used_time);
                model.star.set_value(star);
                model.uuid.set_value(uuid);
                model.remark.set_value(remark);

                model
            }
        }
    }
}

impl TryFrom<workspace_history::Model> for WorkspaceHistory {
    type Error = error::Error;

    fn try_from(model: workspace_history::Model) -> Result<Self, Self::Error> {
        let kind = WorkspaceHistoryKind::from_str(&model.kind).context(builder::EnumParse {
            detail: "Failed to parse WorkspaceHistory",
        })?;

        let uuid = model.uuid;
        let last_used_time = model.last_used_time;
        let star = model.star;
        let remark = model.remark;

        match kind {
            WorkspaceHistoryKind::WorkingCopy => {
                let working_copy_path = model
                    .working_copy_path
                    .any_context("Invalid database data")?;
                let working_copy_root = model
                    .working_copy_root
                    .any_context("Invalid database data")?;
                let repository_root_url = model.repository_root_url;
                let checkout = model.checkout;
                Ok(WorkspaceHistory::WorkingCopy {
                    working_copy_root,
                    working_copy_path,
                    repository_root_url,
                    last_used_time,
                    checkout,
                    order: 0,
                    star,
                    uuid,
                    remark,
                })
            }
            WorkspaceHistoryKind::Repository => {
                let repository_uuid = model.repository_uuid.any_context("invalid database data")?;

                let repository_root_url = model
                    .repository_root_url
                    .any_context("repository_root_url must be set for repository history")?;

                Ok(WorkspaceHistory::Repository {
                    repository_root_url,
                    repository_uuid,
                    last_used_time,
                    order: 0,
                    star,
                    uuid,
                    remark,
                })
            }
        }
    }
}

// impl WorkspaceHistory {
//     fn uuid(&self) -> &str {
//         match self {
//             WorkspaceHistory::WorkingCopy { uuid, .. } => uuid,
//             WorkspaceHistory::Repository { uuid, .. } => uuid,
//         }
//     }
// }

// pub struct SettingsTable;
// impl SettingsTable {
//     pub const TABLE_NAME: &'static str = "Settings";
//     pub const TABLE: TableDefinition<'static, &str, &str> = TableDefinition::new(Self::TABLE_NAME);
//     pub const SETTINGS_KEY: &'static str = "setting";
// }

#[derive(Serialize, Deserialize, Default, uniffi::Record, Debug, Clone)]
pub struct WorkspaceHistoryGroup {
    #[serde(default)]
    pub uuid: String,
    pub name: String,
    pub children: Vec<String>,
}

impl From<WorkspaceHistoryGroup> for workspace_history_group::ActiveModel {
    fn from(value: WorkspaceHistoryGroup) -> Self {
        let mut model = Self::default_value();
        model.uuid.set_value(value.uuid);
        model.name.set_value(value.name);
        model.children.set_value(value.children.so_if_or(
            |v| v.is_empty(),
            |_| None,
            |v| Some(v.join(",")),
        ));
        model
    }
}

impl From<workspace_history_group::Model> for WorkspaceHistoryGroup {
    fn from(model: workspace_history_group::Model) -> Self {
        Self {
            uuid: model.uuid,
            name: model.name,
            children: model
                .children
                .map(|v| {
                    if v.is_empty() {
                        vec![]
                    } else {
                        v.split(',').map(|s| s.to_string()).collect()
                    }
                })
                .unwrap_or_default(),
        }
    }
}

// pub trait Table {
//     const TABLE_NAME: &'static str;
//     const TABLE: TableDefinition<'static, &str, &str> = TableDefinition::new(Self::TABLE_NAME);
// }

// pub struct TimelineTable {}

// impl Table for TimelineTable {
//     const TABLE_NAME: &'static str = "Timeline";
// }

// pub struct HistoryTable;

// impl HistoryTable {
//     pub const TABLE_NAME: &'static str = "History";
//     pub const TABLE: TableDefinition<'static, &str, &str> = TableDefinition::new(Self::TABLE_NAME);

//     pub const HISTORY_KEY: &'static str = "history";

//     pub const HISTORY_GROUPS_KEY: &'static str = "history_groups";
// }

// pub struct RegistryTable;
// impl RegistryTable {
//     pub const TABLE_NAME: &'static str = "Registry";
//     pub const TABLE: TableDefinition<'static, &str, &str> = TableDefinition::new(Self::TABLE_NAME);
// }

// #[derive(Serialize, Deserialize, uniffi::Record, Debug, Clone)]
// pub struct RepositoryTable {
//     revisions: String,
// }

#[derive(Serialize, Deserialize, uniffi::Record, Debug)]
pub struct RelativeLogEntry {
    entry: LogEntry,
    children: Vec<RelativeLogEntry>,
}

#[uniffi::export(with_foreign)]
pub trait UpdateOperation: Send + Sync + 'static {
    fn update(&self, v: Arc<AnyValue>) -> Result<Arc<AnyValue>, error::CSharpError>;
}

fn entries_to_relative(entries: Vec<LogEntry>) -> Vec<RelativeLogEntry> {
    // safe but slow
    fn insert(
        entries: &mut [RelativeLogEntry],
        levels: &[usize],
        entry: RelativeLogEntry,
    ) -> usize {
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

// #[uniffi::export(async_runtime = "tokio")]
// impl DatabaseManager {
//     async fn settings(self) -> error::Result<GlobalSettings> {
//         self.call_async(move |db| {
//             let lock = db.begin_write()?;
//             let table = lock.open_table(SettingsTable::TABLE)?;

//             let Some(value) = table.get(SettingsTable::SETTINGS_KEY)? else {
//                 return Ok(Default::default());
//             };

//             Ok(GlobalSettings::from_json(value.value())?)
//         })
//         .await
//     }

//     async fn set_settings(self, settings: GlobalSettings) -> error::Result<()> {
//         self.call_async(move |db| {
//             let lock = db.begin_write()?;
//             let mut table = lock.open_table(SettingsTable::TABLE)?;

//             table.insert(SettingsTable::SETTINGS_KEY, settings.as_json()?.as_str())?;

//             Ok(())
//         })
//         .await
//     }

//     async fn delete_history_group(self, id: String) -> error::Result<()> {
//         self.call_async(move |db| {
//             let lock = db.begin_write()?;
//             {
//                 let mut table = lock.open_table(HistoryTable::TABLE)?;

//                 let mut groups = vec![];
//                 if let Some(value) = table.get(HistoryTable::HISTORY_GROUPS_KEY)? {
//                     groups = Vec::<WorkspaceHistoryGroup>::from_json(value.value())?;
//                 }

//                 groups.retain(|i| i.uuid != id);

//                 table.insert(HistoryTable::HISTORY_GROUPS_KEY, groups.as_json()?.as_str())?;
//             }

//             lock.commit()?;

//             Ok(())
//         })
//         .await
//     }

//     async fn add_history_group(self, group: WorkspaceHistoryGroup) -> error::Result<()> {
//         self.call_async(move |db| {
//             let lock = db.begin_write()?;
//             {
//                 let mut table = lock.open_table(HistoryTable::TABLE)?;

//                 let mut groups = vec![];
//                 if let Some(value) = table.get(HistoryTable::HISTORY_GROUPS_KEY)? {
//                     groups = Vec::<WorkspaceHistoryGroup>::from_json(value.value())?;
//                 }

//                 groups.push(group);

//                 table.insert(HistoryTable::HISTORY_GROUPS_KEY, groups.as_json()?.as_str())?;
//             }

//             lock.commit()?;

//             Ok(())
//         })
//         .await
//     }

//     async fn history_groups(self) -> error::Result<Vec<WorkspaceHistoryGroup>> {
//         self.call_async(move |db| {
//             let lock = db.begin_write()?;

//             let table = lock.open_table(HistoryTable::TABLE)?;

//             let Some(value) = table.get(HistoryTable::HISTORY_GROUPS_KEY)? else {
//                 return Ok(vec![]);
//             };

//             let groups = Vec::<WorkspaceHistoryGroup>::from_json(value.value())?;

//             Ok(groups)
//         })
//         .await
//     }

//     async fn delete_workspace_history(self, uuid: String) -> error::Result<()> {
//         self.call_async(move |db| {
//             let lock = db.begin_write()?;
//             {
//                 let mut table = lock.open_table(HistoryTable::TABLE)?;

//                 let mut old_histories = vec![];

//                 if let Some(value) = table.get(HistoryTable::HISTORY_KEY)? {
//                     old_histories = serde_json::from_str::<Vec<WorkspaceHistory>>(value.value())?;
//                 }

//                 old_histories.retain(|h| h.uuid() != uuid);

//                 table.insert(HistoryTable::HISTORY_KEY, old_histories.as_json()?.as_str())?;
//             }

//             lock.commit()?;

//             Ok(())
//         })
//         .await
//     }

//     async fn update_workspace_history(
//         self,
//         uuid: String,
//         operation: Arc<dyn UpdateOperation>,
//     ) -> error::Result<()> {
//         self.call_async(move |db| {
//             let lock = db.begin_write()?;
//             {
//                 let mut table = lock.open_table(HistoryTable::TABLE)?;

//                 let mut old_histories = vec![];

//                 if let Some(value) = table.get(HistoryTable::HISTORY_KEY)? {
//                     old_histories = serde_json::from_str::<Vec<WorkspaceHistory>>(value.value())?;
//                 }

//                 for i in old_histories.iter_mut() {
//                     if i.uuid() == uuid {
//                         let v = operation.update(i.clone().into_any_value().into_arc())?;
//                         *i = v.to_workspace_history().any_context("Unexpected value return")?;
//                         // *i = WorkspaceHistory::from_json(&operation.update(i.as_json()?)?)?;
//                         break;
//                     }
//                 }

//                 table.insert(HistoryTable::HISTORY_KEY, old_histories.as_json()?.as_str())?;
//             }

//             lock.commit()?;

//             Ok(())
//         })
//         .await
//     }

//     async fn set_workspace_history(self, history: WorkspaceHistory) -> error::Result<()> {
//         self.call_async(move |db| {
//             let lock = db.begin_write()?;
//             {
//                 let mut table = lock.open_table(HistoryTable::TABLE)?;

//                 let mut old_histories = vec![];

//                 if let Some(value) = table.get(HistoryTable::HISTORY_KEY)? {
//                     old_histories = serde_json::from_str::<Vec<WorkspaceHistory>>(value.value())?;
//                 }

//                 for i in old_histories.iter_mut() {
//                     if i.uuid() == history.uuid() {
//                         *i = history;
//                         break;
//                     }
//                 }

//                 table.insert(HistoryTable::HISTORY_KEY, old_histories.as_json()?.as_str())?;
//             }

//             lock.commit()?;

//             Ok(())
//         })
//         .await
//     }

//     async fn insert_workspace_histories(
//         self,
//         histories: Vec<WorkspaceHistory>,
//     ) -> error::Result<()> {
//         self.call_async(move |db| {
//             let lock = db.begin_write()?;

//             {
//                 let mut table = lock.open_table(HistoryTable::TABLE)?;
//                 let mut old_histories = vec![];

//                 if let Some(value) = table.get(HistoryTable::HISTORY_KEY)? {
//                     old_histories = serde_json::from_str::<Vec<WorkspaceHistory>>(value.value())?;
//                 }

//                 tracing::info!("Insert history: {histories:#?}");

//                 old_histories.extend(histories);

//                 table.insert(HistoryTable::HISTORY_KEY, old_histories.as_json()?.as_str())?;
//             }
//             lock.commit()?;

//             Ok(())
//         })
//         .await
//     }

//     async fn workspace_histories(self) -> error::Result<Vec<WorkspaceHistory>> {
//         self.call_async(move |db| {
//             let lock = db.begin_write()?;

//             let table = lock.open_table(HistoryTable::TABLE)?;

//             let Some(history) = table.get(HistoryTable::HISTORY_KEY)? else {
//                 return Ok(vec![]);
//             };

//             let history = serde_json::from_str::<Vec<WorkspaceHistory>>(history.value())?;

//             Ok(history)
//         })
//         .await
//     }

//     async fn insert_repository_history(
//         self,
//         name: String,
//         entries: Vec<LogEntry>,
//     ) -> error::Result<()> {
//         self.call_async(move |db| {
//             let lock = db.begin_write()?;

//             {
//                 let mut table = lock
//                     .open_table(TableDefinition::new(name.as_str()) as TableDefinition<u32, &str>)?;

//                 let relative = entries_to_relative(entries);

//                 for i in relative {
//                     let Some(revision) = i.entry.revision else {
//                         continue;
//                     };

//                     let v = i.as_json()?;
//                     table.insert(revision, v.as_str())?;
//                 }
//             }
//             lock.commit()?;

//             Ok(())
//         })
//         .await
//     }

//     async fn repository_history(
//         self,
//         name: String,
//         start: u32,
//         end: u32,
//     ) -> error::Result<Vec<RelativeLogEntry>> {
//         self.call_async(move |db| {
//             let lock = db.begin_write()?;
//             let table =
//                 lock.open_table(TableDefinition::new(name.as_str()) as TableDefinition<u32, &str>)?;

//             let mut result = Vec::with_capacity((end - start).try_into().unwrap());

//             let range = table.range(start..end)?;

//             for i in range {
//                 let (_, v) = i?;

//                 let v: RelativeLogEntry = serde_json::from_str(v.value())?;

//                 result.push(v);
//             }

//             Ok(result)
//         })
//         .await
//     }

//     async fn repository_table(self, key: String) -> error::Result<Option<RepositoryTable>> {
//         self.call_async(move |db| {
//             let lock = db.begin_read()?;

//             let registry = lock.open_table(RegistryTable::TABLE)?;

//             let Some(data) = registry.get(key.as_str())? else {
//                 return Ok(None);
//             };

//             // let mut reader = Reader::new(data.value())?;
//             let v: RepositoryTable = serde_json::from_str(data.value())?;
//             // let table = reader.one()?.as_type::<RepositoryTable>()?;
//             Ok(Some(v))
//         })
//         .await
//     }

//     async fn resgister_repository(
//         self,
//         key: String,
//         table: Option<RepositoryTable>,
//     ) -> error::Result<RepositoryTable> {
//         self.call_async(move |db| {
//             let table = table.unwrap_or_else(|| RepositoryTable {
//                 revisions: Uuid::new_v4().to_string(),
//             });
//             let lock = db.begin_write()?;

//             let mut registry = lock.open_table(RegistryTable::TABLE)?;

//             // let value = RepositoryTable::get_schema().value(&table)?;
//             //

//             let value = table.as_json()?;

//             registry.insert(key.as_str(), value.as_str())?;

//             Ok(table)
//         })
//         .await
//     }
// }
