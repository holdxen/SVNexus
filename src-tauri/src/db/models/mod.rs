use serde::{Deserialize, Serialize};
use surrealdb::types::SurrealValue;

#[derive(Debug, serde::Serialize, serde::Deserialize, ts_rs::TS, SurrealValue)]
#[ts(export)]
pub struct WorkspaceGroup {
    pub identity: String,
    pub name: String,
    pub members: Vec<String>,
    pub order: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, ts_rs::TS, SurrealValue)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum WorkspaceItem {
    #[serde(rename_all = "camelCase")]
    WorkingCopy {
        working_copy_root: String,
        working_copy_path: String,
        repository_root_url: Option<String>,
        #[ts(type = "number | null")]
        last_used_time: Option<i64>,
        #[ts(type = "number | null")]
        checkout: Option<i64>,
        order: String,
        star: bool,
        identity: String,
        remark: Option<String>,
        name: String,
    },
    #[serde(rename_all = "camelCase")]
    Repository {
        repository_root_url: String,
        repository_uuid: String,
        #[ts(type = "number | null")]
        last_used_time: Option<i64>,
        order: String,
        star: bool,
        identity: String,
        remark: Option<String>,
        name: String,
    },
}

impl WorkspaceItem {
    pub fn identity(&self) -> &str {
        match self {
            WorkspaceItem::WorkingCopy { identity, .. } => identity,
            WorkspaceItem::Repository { identity, .. } => identity,
        }
    }
}
