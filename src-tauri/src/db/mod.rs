// mod migrations;
pub mod models;
mod tables;

use crate::db::tables::*;
use crate::subversion::context::log::LogEntry;
use crate::{
    app,
    error::{self, builder},
    extensions::*,
};
use serde::{Deserialize, Serialize};
use snafu::ResultExt;
use surrealdb::engine::local::{Db, SurrealKv};
use surrealdb::types::SurrealValue;
use surrealdb::Surreal;

pub(crate) fn messagepack_serialize<T: Serialize>(value: &T) -> error::Result<Vec<u8>> {
    Ok(rmp_serde::to_vec_named(value)?)
}

/// 每个 LogEntry 一个独立文档，record ID: log_entry:{repository}:{revision}
#[derive(Serialize, Deserialize, Debug, SurrealValue)]
struct LogEntryRecord {
    repository: String,
    revision: u32,
    entry: LogEntry,
}

/// 每个 (repository, path) 一个文档，包含 revision 索引
#[derive(Serialize, Deserialize, Debug, SurrealValue)]
struct LogReferenceStore {
    repository: String,
    path: String,
    revisions: Vec<u32>,
}

#[derive(Serialize, Deserialize, Debug, SurrealValue)]
struct RevisionLocationRecord {
    repository: String,
    path: String,
    peg_revision: u32,
    revision: u32,
    location: String,
}

#[derive(Debug)]
pub struct DatabaseConnection {
    connection: Surreal<Db>,
}

impl DatabaseConnection {
    pub async fn create() -> error::Result<Self> {
        let project = app::project().any_context("Failed to get project directory")?;
        let path = project.database_file();

        tracing::info!("database file: {}", path.display());

        let path_str = path.to_str().any_context("Failed to get database path")?;

        let connection = Surreal::new::<SurrealKv>(path_str)
            .await
            .context(builder::Database)?;

        connection
            .use_ns("svnexus")
            .use_db("main")
            .await
            .context(builder::Database)?;

        // 预定义表，避免读取空表时报错
        let define_sql = format!(
            "DEFINE TABLE IF NOT EXISTS {WORKSPACE_GROUP};
             DEFINE TABLE IF NOT EXISTS {WORKSPACE_ITEM};
             DEFINE TABLE IF NOT EXISTS {LOG_ENTRY};
             DEFINE TABLE IF NOT EXISTS {LOG_REFERENCE};
             DEFINE TABLE IF NOT EXISTS {REVISION_LOCATION};"
        );
        connection
            .query(&define_sql)
            .await
            .context(builder::Database)?;

        // migrations::execute(&connection).await?;

        Ok(Self { connection })
    }

    // pub async fn truncate_repository_logs(&self, _repository_uuid: &str) -> error::Result<()> {
    //     Ok(())
    // }

    // ========== WorkspaceItem ==========

    pub async fn add_workspace_item(&self, item: models::WorkspaceItem) -> error::Result<()> {
        // let identity = item.identity().to_string();
        self.connection
            .upsert::<Option<models::WorkspaceItem>>((WORKSPACE_ITEM, item.identity()))
            .content(item)
            .await
            .context(builder::Database)?;
        Ok(())
    }

    pub async fn delete_workspace_item(&self, uuid: String) -> error::Result<()> {
        self.connection
            .delete::<Option<models::WorkspaceItem>>((WORKSPACE_ITEM, uuid.as_str()))
            .await
            .context(builder::Database)?;
        Ok(())
    }

    pub async fn workspace_items(&self) -> error::Result<Vec<models::WorkspaceItem>> {
        let items: Vec<models::WorkspaceItem> = self
            .connection
            .select(WORKSPACE_ITEM)
            .await
            .context(builder::Database)?;
        Ok(items)
    }

    // ========== WorkspaceGroup ==========

    pub async fn add_workspace_group(&self, group: models::WorkspaceGroup) -> error::Result<()> {
        // let identity = group.identity.clone();
        self.connection
            .upsert::<Option<models::WorkspaceGroup>>((WORKSPACE_GROUP, group.identity.as_str()))
            .content(group)
            .await
            .context(builder::Database)?;
        Ok(())
    }

    pub async fn delete_workspace_group(&self, uuid: String) -> error::Result<()> {
        self.connection
            .delete::<Option<models::WorkspaceGroup>>((WORKSPACE_GROUP, uuid.as_str()))
            .await
            .context(builder::Database)?;
        Ok(())
    }

    pub async fn workspace_groups(&self) -> error::Result<Vec<models::WorkspaceGroup>> {
        let groups: Vec<models::WorkspaceGroup> = self
            .connection
            .select(WORKSPACE_GROUP)
            .await
            .context(builder::Database)?;
        Ok(groups)
    }

    // pub async fn workspace_group(
    //     &self,
    //     identity: String,
    // ) -> error::Result<Option<models::WorkspaceGroup>> {
    //     // typed API: 按 record ID 查询单条记录
    //     let group: Option<models::WorkspaceGroup> = self
    //         .connection
    //         .select((WORKSPACE_GROUP, identity.as_str()))
    //         .await
    //         .context(builder::Database)?;
    //     Ok(group)
    // }

    pub async fn update_workspace_group(&self, item: models::WorkspaceGroup) -> error::Result<()> {
        let identity = item.identity.clone();
        self.connection
            .upsert::<Option<models::WorkspaceGroup>>((WORKSPACE_GROUP, identity.as_str()))
            .content(item)
            .await
            .context(builder::Database)?;
        Ok(())
    }

    // ========== Repository Log ==========

    pub async fn repository_log_entry_reverse(
        &self,
        repository: String,
        path: String,
        start: u32,
        limit: u32,
    ) -> error::Result<Option<Vec<LogEntry>>> {
        let reference: Option<LogReferenceStore> = self
            .connection
            .select((LOG_REFERENCE, format!("{}:{}", repository, path).as_str()))
            .await
            .context(builder::Database)?;

        let Some(reference) = reference else {
            tracing::info!("No such reference: {}:{}", repository, path);
            return Ok(None);
        };

        let revisions: Vec<u32> = reference
            .revisions
            .iter()
            .copied()
            .take_while(|r| *r >= start)
            .take(limit as usize)
            .collect();

        if revisions.is_empty() {
            return Ok(None);
        }

        let entries = self.query_log_entries(&repository, &revisions).await?;
        if entries.is_empty() {
            Ok(None)
        } else {
            Ok(Some(entries))
        }
    }

    pub async fn repository_log_entry(
        &self,
        repository: String,
        path: String,
        start: u32,
        limit: u32,
    ) -> error::Result<Option<Vec<LogEntry>>> {
        let reference: Option<LogReferenceStore> = self
            .connection
            .select((LOG_REFERENCE, format!("{}:{}", repository, path).as_str()))
            .await
            .context(builder::Database)?;

        let Some(reference) = reference else {
            tracing::info!("No such reference: {}:{}", repository, path);
            return Ok(None);
        };

        let position = reference.revisions.iter().position(|i| start > *i);

        let revisions: Vec<u32> = match position {
            Some(pos) => reference.revisions[pos..]
                .iter()
                .copied()
                .take(limit as usize)
                .collect(),
            None => return Ok(None),
        };

        tracing::info!(
            "Repository log revisions: repository={} revisions={:?}",
            repository,
            revisions
        );

        if revisions.is_empty() {
            return Ok(None);
        }

        let entries = self.query_log_entries(&repository, &revisions).await?;
        if entries.is_empty() {
            Ok(None)
        } else {
            tracing::info!(
                "Got entries: {:#?}",
                entries.iter().map(|v| v.revision).collect::<Vec<_>>()
            );
            Ok(Some(entries))
        }
    }

    /// 查询指定 revisions 的 LogEntry，每个 entry 是独立文档
    async fn query_log_entries(
        &self,
        repository: &str,
        revisions: &[u32],
    ) -> error::Result<Vec<LogEntry>> {
        let records: Vec<LogEntryRecord> = self
            .connection
            .query(&format!(
                "SELECT * FROM {LOG_ENTRY} WHERE repository = $repo AND revision INSIDE $revisions ORDER BY revision DESC"
            ))
            .bind(("repo", repository))
            .bind(("revisions", revisions.to_vec()))
            .await
            .context(builder::Database)?
            .take(0)
            .context(builder::Database)?;

        Ok(records.into_iter().map(|r| r.entry).collect())
    }

    pub async fn update_repository_log(
        &self,
        repository: String,
        path: String,
        entries: Vec<LogEntry>,
    ) -> error::Result<()> {
        // 逐条 upsert 每个 entry 为独立文档，无需加载旧数据
        for entry in entries.iter() {
            let Some(revision) = entry.revision else {
                continue;
            };
            let record = LogEntryRecord {
                repository: repository.clone(),
                revision,
                entry: entry.clone(),
            };
            self.connection
                .query(&format!(
                    "UPSERT {LOG_ENTRY}:⟨{repository}:{revision}⟩ CONTENT $data"
                ))
                .bind(("data", record))
                .await
                .context(builder::Database)?;
        }

        // 更新 reference 的 revision 索引
        let reference: Option<LogReferenceStore> = self
            .connection
            .select((LOG_REFERENCE, format!("{}:{}", repository, path).as_str()))
            .await
            .context(builder::Database)?;

        let mut revisions: Vec<u32> = reference.map(|r| r.revisions).unwrap_or_default();

        // 合并新 revisions
        for entry in entries.iter() {
            if let Some(revision) = entry.revision {
                if !revisions.contains(&revision) {
                    revisions.push(revision);
                }
            }
        }
        revisions.sort_by(|a, b| b.cmp(a));

        let reference = LogReferenceStore {
            repository: repository.clone(),
            path: path.clone(),
            revisions,
        };

        tracing::info!("Update log reference: {:?}", reference);
        tracing::info!(
            "Update log reference: repository={}, path={}",
            repository,
            path
        );

        self.connection
            .upsert::<Option<LogReferenceStore>>((
                LOG_REFERENCE,
                format!("{}:{}", repository, path).as_str(),
            ))
            .content(reference)
            .await
            .context(builder::Database)?;

        Ok(())
    }

    // ========== Revision Location ==========

    pub async fn update_revision_location(
        &self,
        repository: String,
        path: String,
        peg_revision: u32,
        revision: u32,
        location: String,
    ) -> error::Result<()> {
        let record = RevisionLocationRecord {
            repository: repository.clone(),
            path: path.clone(),
            peg_revision,
            revision,
            location,
        };
        // typed API: upsert 到表（每次创建/覆盖）
        self.connection
            .upsert::<Vec<RevisionLocationRecord>>(REVISION_LOCATION)
            .content(record)
            .await
            .context(builder::Database)?;
        Ok(())
    }

    pub async fn revision_location(
        &self,
        repository: String,
        path: String,
        peg_revision: u32,
        revision: u32,
    ) -> error::Result<Option<String>> {
        // typed API 不支持 WHERE 过滤，使用 SQL 查询
        let record: Option<RevisionLocationRecord> = self
            .connection
            .query(&format!("SELECT * FROM {REVISION_LOCATION} WHERE repository = $repo AND path = $path AND peg_revision = $peg AND revision = $rev LIMIT 1"))
            .bind(("repo", repository.as_str()))
            .bind(("path", path.as_str()))
            .bind(("peg", peg_revision))
            .bind(("rev", revision))
            .await
            .context(builder::Database)?
            .take(0)
            .context(builder::Database)?;

        Ok(record.map(|r| r.location))
    }
}
