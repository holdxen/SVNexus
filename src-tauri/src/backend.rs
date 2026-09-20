use camino::Utf8PathBuf;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use snafu::{OptionExt, ResultExt};

use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};

use crate::{
    db::models::{WorkspaceGroup, WorkspaceItem},
    error::{builder, FrontendError},
    extensions::CommonExtension,
    messagepack_command::MessagePackChannel,
    platform::Platform,
    subversion::{
        context::{
            blame::{BlameOptions, BlameResult},
            cat::{CatOptions, CatResult},
            copy::{CopyOptions, CopyResult},
            difference::{ClientDifferenceOptions, ClientDifferenceResult},
            log::{LogEntry, LogOptions, LogResult},
            merge::MergeOptions,
            move_::{MoveOptions, MoveResult},
            revert::RevertOptions,
            switch::SwitchOptions,
            upgrade::{UpgradeOptions, UpgradeResult},
            vacuum::VacuumOptions,
            AddOptions, Authentication, CheckoutOptions, CleanupOptions, ClientCertificate,
            CommitOptions, CommitResult, ConflictWalkOptions, ConflictWalkResult, ContextNotifier,
            CreateContextOptions, DeleteOptions, DeleteResult, ExportOptions, ImportOptions,
            ImportResult, InfoOptions, InfoResult, ListOptions, ListResult, LockOptions,
            MkdirOptions, MkdirResult, PatchOptions, PropertyGetOptions, PropertyGetResult,
            PropertyListOptions, PropertyListResult, PropertySetOptions, RelocateOptions, Revision,
            RevisionNumber, RevisionPropertyListOptions, RevisionPropertyListResult, RevisionRange,
            SslServerCertInfo, StatusOptions, StatusResult, TrustServer, UnlockOptions,
            UpdateOptions,
        },
        export::AsyncContext,
        version::{self, Version},
        wc::{
            WcReplacedNode, WorkingCopyConflictDescription, WorkingCopyConflictResult,
            WorkingCopyNotify, WorkingCopyRevisionStatusOptions, WorkingCopyRevisionStatusResult,
        },
    },
};

use super::*;

use crate::subversion::ra::LocationSegment;
use db::DatabaseConnection;
use error::Result;
use tokio::sync::{oneshot, Mutex, OnceCell};

pub static DATABASE: OnceCell<db::DatabaseConnection> = OnceCell::const_new();

async fn load_database() -> Result<&'static DatabaseConnection> {
    let db = DATABASE
        .get_or_try_init(|| async move {
            let db = db::DatabaseConnection::create().await?;

            error::ok(db)
        })
        .await?;

    Ok(db)
}

#[svnexus_macro::messagepack_command]
pub async fn database_workspace_groups() -> error::Result<Vec<WorkspaceGroup>> {
    let db = load_database().await?;

    let groups = db.workspace_groups().await?;

    Ok(groups)
}

#[svnexus_macro::messagepack_command]
pub async fn database_delete_workspace_group(identity: String) -> error::Result<()> {
    let db = load_database().await?;

    db.delete_workspace_group(identity).await?;

    Ok(())
}

#[svnexus_macro::messagepack_command]
pub async fn database_add_workspace_group(group: WorkspaceGroup) -> error::Result<()> {
    let db = load_database().await?;

    db.add_workspace_group(group).await?;

    Ok(())
}

#[svnexus_macro::messagepack_command]
pub async fn database_update_workspace_group(group: WorkspaceGroup) -> error::Result<()> {
    let db = load_database().await?;

    db.update_workspace_group(group).await?;

    Ok(())
}

#[svnexus_macro::messagepack_command]
pub async fn database_add_workspace_item(item: WorkspaceItem) -> error::Result<()> {
    let db = load_database().await?;

    let result = db.add_workspace_item(item).await;

    tracing::info!("result is: {:?}", result);

    Ok(())
}

#[svnexus_macro::messagepack_command]
pub async fn database_workspace_items() -> error::Result<Vec<WorkspaceItem>> {
    let db = load_database().await?;

    let items = db.workspace_items().await?;

    Ok(items)
}

#[svnexus_macro::messagepack_command]
pub async fn database_delete_workspace_item(identity: String) -> error::Result<()> {
    let db = load_database().await?;

    db.delete_workspace_item(identity).await?;

    Ok(())
}

#[svnexus_macro::messagepack_command]
pub async fn database_update_repository_log(
    repository: String,
    path: String,
    entries: Vec<LogEntry>,
) -> error::Result<()> {
    let db = load_database().await?;
    db.update_repository_log(repository, path, entries).await?;
    Ok(())
}

#[svnexus_macro::messagepack_command]
#[tracing::instrument(err)]
pub async fn database_revision_location(
    repository: String,
    path: String,
    peg_revision: u32,
    revision: u32,
) -> error::Result<Option<String>> {
    let db = load_database().await?;
    let location = db
        .revision_location(repository, path, peg_revision, revision)
        .await?;
    Ok(location)
}

#[svnexus_macro::messagepack_command]
pub async fn database_update_revision_location(
    repository: String,
    path: String,
    peg_revision: u32,
    revision: u32,
    location: String,
) -> error::Result<()> {
    let db = load_database().await?;
    db.update_revision_location(repository, path, peg_revision, revision, location)
        .await?;
    Ok(())
}

#[svnexus_macro::messagepack_command]
pub fn path_get_parent(path: String) -> Option<String> {
    let path = Utf8PathBuf::from(path);

    let file_name = path.parent();

    file_name.map(|v| v.to_string())
}

#[svnexus_macro::messagepack_command]
pub fn path_get_file_name(path: String) -> Option<String> {
    let path = Utf8PathBuf::from(path);

    let file_name = path.file_name();

    file_name.map(|v| v.to_string())
}

#[svnexus_macro::messagepack_command]
pub fn path_strip_prefix(path: String, prefix: String) -> error::Result<String> {
    let path = Utf8PathBuf::from(path);
    let prefix = Utf8PathBuf::from(prefix);

    let left = path
        .strip_prefix(&prefix)
        .ok()
        .context(builder::General { detail: "ok" })?;

    Ok(left.to_string())
}

#[svnexus_macro::messagepack_command]
pub fn path_combine(parts: Vec<String>) -> String {
    let mut path = Utf8PathBuf::new();

    for i in parts {
        path.push(i);
    }

    path.to_string()
}

#[svnexus_macro::messagepack_command]
pub fn path_into_parts(path: String) -> Vec<String> {
    let path = Utf8PathBuf::from(path);

    path.into_iter().map(|v| v.to_string()).collect()
}

#[svnexus_macro::messagepack_command]
pub fn path_start_with(path: String, prefix: String) -> bool {
    let path = Utf8PathBuf::from(path);
    let prefix = Utf8PathBuf::from(prefix);
    path.starts_with(prefix)
}

#[svnexus_macro::messagepack_command]
pub async fn fs_read_link(path: String) -> error::Result<String> {
    let target = tokio::fs::read_link(&path).await?;

    target
        .to_str()
        .map(|v| v.to_string())
        .context(builder::General {
            detail: format!("the target of symlink {} is not valid UTF-8", path),
        })
}

#[derive(Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum SubversionEvent {
    SavePasswordAsPlainText {
        #[ts(type = "number")]
        id: u64,
        realm: String,
    },
    WorkingCopyNotify {
        notify: WorkingCopyNotify,
    },
    #[serde(rename_all = "camelCase")]
    SslServerTrustPrompt {
        #[ts(type = "number")]
        id: u64,
        realm: String,
        #[ts(type = "number")]
        failures: u32,
        info: SslServerCertInfo,
        may_save: bool,
    },
    ProgressNotify {
        #[ts(type = "number")]
        pos: i64,
        #[ts(type = "number")]
        total: i64,
    },
    #[serde(rename_all = "camelCase")]
    Authenticate {
        #[ts(type = "number")]
        id: u64,
        realm: String,
        username: String,
        may_save: bool,
        need_password: bool,
    },
    Conflict {
        #[ts(type = "number")]
        id: u64,
        description: WorkingCopyConflictDescription,
    },
    #[serde(rename_all = "camelCase")]
    SslClientCertificate {
        #[ts(type = "number")]
        id: u64,
        realm: String,
        may_save: bool,
    },
}

#[derive(derive_more::Debug)]
pub struct ContextNotifierImpl {
    channel: MessagePackChannel,
    cancel_msg: Mutex<Option<String>>,
}

impl ContextNotifierImpl {
    fn request<T: DeserializeOwned, F: FnOnce(u64) -> SubversionEvent>(
        &self,
        f: F,
    ) -> Result<T, FrontendError> {
        let (sender, receiver) = oneshot::channel();

        let id = {
            let mut reply = Reply::instance().blocking_lock();

            let id = reply.insert(sender);

            id
        };

        let event = f(id);

        self.channel
            .send(&event)
            .ok()
            .context(error::UnexpectedSnafu {
                detail: "Failed to send message from channel",
            })?;

        let value = receiver
            .blocking_recv()
            .ok()
            .context(error::UnexpectedSnafu {
                detail: "Failed to received message",
            })?;

        let value = match value {
            ReplyMessage::Success(value) => value,
            ReplyMessage::Failure(error) => return Err(error),
        };

        tracing::info!("Reply value: {:?}", value);

        let value = rmp_serde::from_slice(&value).map_err(|error| {
            error::UnexpectedSnafu {
                detail: format!("Unexpected value: {}", error),
            }
            .build()
        })?;

        Ok(value)
    }
}

impl ContextNotifier for ContextNotifierImpl {
    fn may_save_password_as_plain_text(&self, realm: String) -> Result<bool, FrontendError> {
        // let (sender, receiver) = oneshot::channel();

        // let mut reply = Reply::instance().blocking_lock();

        // let id = reply.insert(sender);

        // self.channel
        //     .send(SubversionEvent::SavePasswordAsPlainText { id, realm })?;

        // let value = receiver.blocking_recv().ok().context(error::UnexpectedSnafu {
        // 	detail: "Failed to received message"
        // })?;

        // let value = match value {
        // 	ReplyMessage::Success(value) => value,
        // 	ReplyMessage::Failure(error) => return Err(error),
        // };

        // Ok(value.as_bool().context(error::UnexpectedSnafu {
        // 	detail: "Unexpected value"
        // })?)

        self.request(|id| SubversionEvent::SavePasswordAsPlainText { id, realm })
    }

    fn working_copy_notify(&self, notify: WorkingCopyNotify) -> Result<(), FrontendError> {
        let event = SubversionEvent::WorkingCopyNotify { notify };

        self.channel
            .send(&event)
            .ok()
            .context(error::UnexpectedSnafu {
                detail: "Failed to send message from channel",
            })?;
        Ok(())
    }

    fn ssl_server_trust_prompt(
        &self,
        realm: String,
        failures: u32,
        info: SslServerCertInfo,
        may_save: bool,
    ) -> Result<Option<TrustServer>, FrontendError> {
        self.request(|id| SubversionEvent::SslServerTrustPrompt {
            id,
            realm,
            failures,
            info,
            may_save,
        })
    }

    fn cancel(&self) -> Result<Option<String>, FrontendError> {
        let msg = self.cancel_msg.blocking_lock().take();
        Ok(msg)
    }

    fn progress_notify(&self, pos: i64, total: i64) -> Result<(), FrontendError> {
        let event = SubversionEvent::ProgressNotify { pos, total };

        self.channel
            .send(&event)
            .ok()
            .context(error::UnexpectedSnafu {
                detail: "Failed to send message from channel",
            })?;
        Ok(())
    }

    fn authenticate(
        &self,
        realm: String,
        username: String,
        may_save: bool,
        need_password: bool,
    ) -> Result<Option<Authentication>, FrontendError> {
        self.request(|id| SubversionEvent::Authenticate {
            id,
            realm,
            username,
            may_save,
            need_password,
        })
    }

    #[tracing::instrument(ret, err)]
    fn conflict(
        &self,
        description: WorkingCopyConflictDescription,
    ) -> Result<WorkingCopyConflictResult, FrontendError> {
        self.request(|id| SubversionEvent::Conflict { id, description })
    }

    fn ssl_client_certificate(
        &self,
        realm: String,
        may_save: bool,
    ) -> Result<Option<ClientCertificate>, FrontendError> {
        self.request(|id| SubversionEvent::SslClientCertificate {
            id,
            realm,
            may_save,
        })
    }
}

#[derive(Default)]
struct Subversion {
    next: u64,
    context: HashMap<u64, AsyncContext>,
    notifier: HashMap<u64, Arc<ContextNotifierImpl>>,
}

impl Subversion {
    fn instance() -> &'static Mutex<Subversion> {
        static INSTANCE: OnceLock<Mutex<Subversion>> = OnceLock::new();

        INSTANCE.get_or_init(|| Default::default())
    }

    fn clear(&mut self) {
        self.next = 0;
        tracing::info!("Clean context size:{}", self.context.len());
        self.context.clear();
        self.notifier.clear();
    }

    fn take_context(&mut self, id: u64) -> Option<AsyncContext> {
        self.context.remove(&id)
    }

    fn take_notifier(&mut self, id: u64) -> Option<Arc<ContextNotifierImpl>> {
        self.notifier.remove(&id)
    }

    fn create(
        &mut self,
        name: Option<String>,
        default_username: Option<String>,
        default_password: Option<String>,
        channel: MessagePackChannel,
        config: subversion::context::Config,
    ) -> error::Result<u64> {
        let notifier = ContextNotifierImpl {
            channel,
            cancel_msg: Default::default(),
        };

        let notifier = Arc::new(notifier);

        let options = CreateContextOptions {
            name,
            default_password,
            default_username,
            context_notifier: notifier.clone() as _,
            config,
        };

        let context = AsyncContext::create(options)?;

        self.next = self.next.wrapping_add(1);
        self.context.insert(self.next, context);
        self.notifier.insert(self.next, notifier);

        Ok(self.next)
    }

    fn get(&self, id: u64) -> error::Result<AsyncContext> {
        let context = self.context.get(&id).context(builder::General {
            detail: "No such context",
        })?;

        Ok(context.clone())
    }

    fn get_notifier(&self, id: u64) -> error::Result<Arc<ContextNotifierImpl>> {
        let context = self.notifier.get(&id).context(builder::General {
            detail: "No such context",
        })?;

        Ok(context.clone())
    }
}

#[derive(Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SourceLocation<'a> {
    pub file: &'a str,
    pub member: &'a str,
    pub line: i32,
}

#[svnexus_macro::messagepack_command]
pub fn log_info(message: &str, location: SourceLocation<'_>) {
    utils::log_info(location.line, location.file, location.member, message);
}

#[svnexus_macro::messagepack_command]
pub fn log_debug(message: &str, location: SourceLocation<'_>) {
    utils::log_debug(location.line, location.file, location.member, message);
}

#[svnexus_macro::messagepack_command]
pub fn log_warn(message: &str, location: SourceLocation<'_>) {
    utils::log_warn(location.line, location.file, location.member, message);
}

#[svnexus_macro::messagepack_command]
pub fn log_trace(message: &str, location: SourceLocation<'_>) {
    utils::log_trace(location.line, location.file, location.member, message);
}

#[svnexus_macro::messagepack_command]
pub fn log_error(message: &str, location: SourceLocation<'_>) {
    utils::log_error(location.line, location.file, location.member, message);
}

macro_rules! context {
    ($id:expr) => {{
        let subversion = Subversion::instance().lock().await;
        let context = subversion.get($id)?;
        context
    }};
}

#[svnexus_macro::messagepack_command]
pub async fn subversion_cat(id: u64, options: CatOptions) -> error::Result<CatResult> {
    let context = context!(id);
    context.cat(options).await
}

#[svnexus_macro::messagepack_command]
pub async fn subversion_status(id: u64, options: StatusOptions) -> error::Result<StatusResult> {
    let context = context!(id);

    context.status(options).await
}

#[svnexus_macro::messagepack_command]
pub async fn subversion_add(id: u64, options: AddOptions) -> error::Result<()> {
    let context = context!(id);
    context.add(options).await
}

#[svnexus_macro::messagepack_command]
pub async fn subversion_checkout(
    id: u64,
    options: CheckoutOptions,
) -> error::Result<RevisionNumber> {
    let context = context!(id);
    context.checkout(options).await
}

#[svnexus_macro::messagepack_command]
pub async fn subversion_cleanup(id: u64, options: CleanupOptions) -> error::Result<()> {
    let context = context!(id);
    context.cleanup(options).await
}

#[svnexus_macro::messagepack_command]
pub async fn subversion_commit(id: u64, options: CommitOptions) -> error::Result<CommitResult> {
    let context = context!(id);
    context.commit(options).await
}

#[svnexus_macro::messagepack_command]
pub async fn subversion_copy(id: u64, options: CopyOptions) -> error::Result<CopyResult> {
    let context = context!(id);
    context.call_async(|mut ctx| ctx.copy(options)).await
}

#[svnexus_macro::messagepack_command]
pub async fn subversion_conflict_walk(
    id: u64,
    options: ConflictWalkOptions,
) -> error::Result<ConflictWalkResult> {
    let context = context!(id);
    context.conflict_walk(options).await
}

#[svnexus_macro::messagepack_command]
pub async fn subversion_delete(id: u64, options: DeleteOptions) -> error::Result<DeleteResult> {
    let context = context!(id);
    context.delete(options).await
}

#[svnexus_macro::messagepack_command]
pub async fn subversion_difference(
    id: u64,
    options: ClientDifferenceOptions,
) -> error::Result<ClientDifferenceResult> {
    let context = context!(id);
    context.difference(options).await
}

#[svnexus_macro::messagepack_command]
pub async fn subversion_export(
    id: u64,
    options: ExportOptions,
) -> error::Result<Option<RevisionNumber>> {
    let context = context!(id);
    context.export(options).await
}

#[svnexus_macro::messagepack_command]
pub async fn subversion_import(
    id: u64,
    options: ImportOptions,
    filters: Option<Vec<String>>,
) -> error::Result<ImportResult> {
    let context = context!(id);
    context.import(options, filters).await
}

#[svnexus_macro::messagepack_command]
pub async fn subversion_info(id: u64, options: InfoOptions) -> error::Result<InfoResult> {
    let context = context!(id);
    context.info(options).await
}

#[svnexus_macro::messagepack_command]
pub async fn subversion_list(id: u64, options: ListOptions) -> error::Result<ListResult> {
    let context = context!(id);
    context.list(options).await
}

#[svnexus_macro::messagepack_command]
pub async fn subversion_lock(id: u64, options: LockOptions) -> error::Result<()> {
    let context = context!(id);
    context.lock(options).await
}

#[svnexus_macro::messagepack_command]
pub async fn subversion_log(id: u64, options: LogOptions) -> error::Result<LogResult> {
    let context = context!(id);
    context.log(options).await
}

#[svnexus_macro::messagepack_command]
pub async fn subversion_log_next(
    id: u64,
    options: LogOptions,
    #[channel] channel: MessagePackChannel,
) -> error::Result<()> {
    let context = context!(id);
    let cloned = channel.clone();
    context
        .log_next(options, move |entry| {
            channel.send(&entry).ok().context(error::UnexpectedSnafu {
                detail: "Failed to send message from channel",
            })?;
            Ok(())
        })
        .await?;
    cloned
        .send(&None::<LogEntry>)
        .ok()
        .context(error::UnexpectedSnafu {
            detail: "Failed to send message from channel",
        })?;
    Ok(())
}

#[svnexus_macro::messagepack_command]
pub async fn subversion_merge(id: u64, options: MergeOptions) -> error::Result<()> {
    let context = context!(id);
    context.merge(options).await
}

#[svnexus_macro::messagepack_command]
pub async fn subversion_mkdir(id: u64, options: MkdirOptions) -> error::Result<MkdirResult> {
    let context = context!(id);
    context.mkdir(options).await
}

#[svnexus_macro::messagepack_command]
pub async fn subversion_move(id: u64, options: MoveOptions) -> error::Result<MoveResult> {
    let context = context!(id);
    context.call_async(|mut ctx| ctx.move_(options)).await
}

#[svnexus_macro::messagepack_command]
pub async fn subversion_patch(id: u64, options: PatchOptions) -> error::Result<()> {
    let context = context!(id);
    context.patch(options).await
}

#[svnexus_macro::messagepack_command]
pub async fn subversion_property_get(
    id: u64,
    options: PropertyGetOptions,
) -> error::Result<PropertyGetResult> {
    let context = context!(id);
    context.property_get(options).await
}

#[svnexus_macro::messagepack_command]
pub async fn subversion_property_list(
    id: u64,
    options: PropertyListOptions,
) -> error::Result<PropertyListResult> {
    let context = context!(id);
    context.property_list(options).await
}

#[svnexus_macro::messagepack_command]
pub async fn subversion_property_set(id: u64, options: PropertySetOptions) -> error::Result<()> {
    let context = context!(id);
    context.property_set(options).await
}

#[svnexus_macro::messagepack_command]
pub async fn subversion_relocate(id: u64, options: RelocateOptions) -> error::Result<()> {
    let context = context!(id);
    context.relocate(options).await
}

#[svnexus_macro::messagepack_command]
pub async fn subversion_revert(id: u64, options: RevertOptions) -> error::Result<()> {
    let context = context!(id);
    context.revert(options).await
}

#[svnexus_macro::messagepack_command]
pub async fn subversion_revision_property_list(
    id: u64,
    options: RevisionPropertyListOptions,
) -> error::Result<RevisionPropertyListResult> {
    let context = context!(id);
    context.revision_property_list(options).await
}

#[svnexus_macro::messagepack_command]
pub async fn subversion_switch(id: u64, options: SwitchOptions) -> error::Result<RevisionNumber> {
    let context = context!(id);
    context.switch(options).await
}

#[svnexus_macro::messagepack_command]
pub async fn subversion_unlock(id: u64, options: UnlockOptions) -> error::Result<()> {
    let context = context!(id);
    context.unlock(options).await
}

#[svnexus_macro::messagepack_command]
pub async fn subversion_update(
    id: u64,
    options: UpdateOptions,
) -> error::Result<Vec<Option<RevisionNumber>>> {
    let context = context!(id);
    context.update(options).await
}

#[svnexus_macro::messagepack_command]
pub async fn subversion_blame(id: u64, options: BlameOptions) -> error::Result<BlameResult> {
    let context = context!(id);
    context.call_async(|mut ctx| ctx.blame(options)).await
}

#[svnexus_macro::messagepack_command]
pub async fn subversion_upgrade(id: u64, options: UpgradeOptions) -> error::Result<UpgradeResult> {
    let context = context!(id);
    context.call_async(|mut ctx| ctx.upgrade(options)).await
}

#[svnexus_macro::messagepack_command]
pub async fn subversion_vacuum(id: u64, options: VacuumOptions) -> error::Result<()> {
    let context = context!(id);
    context.call_async(|mut ctx| ctx.vacuum(options)).await
}

#[svnexus_macro::messagepack_command]
pub async fn subversion_url_from_path(id: u64, path: String) -> error::Result<String> {
    let context = context!(id);
    context.url_from_path(path).await
}

#[svnexus_macro::messagepack_command]
pub async fn subversion_default_wc_version(id: u64) -> error::Result<Version> {
    let context = context!(id);
    context.default_wc_version().await
}

#[svnexus_macro::messagepack_command]
pub async fn subversion_create(
    name: Option<String>,
    default_username: Option<String>,
    default_password: Option<String>,
    #[channel] channel: MessagePackChannel,
    config: subversion::context::Config,
) -> error::Result<u64> {
    let mut subversion = Subversion::instance().lock().await;

    let id = subversion.create(name, default_username, default_password, channel, config)?;

    tracing::info!("Create subversion: {}", id);

    Ok(id)
}

#[svnexus_macro::messagepack_command]
pub async fn subversion_destroy(id: u64) {
    tracing::info!("Destroy subversion: {}", id);
    let mut subversion = Subversion::instance().lock().await;
    subversion.take_context(id);
    subversion.take_notifier(id);
}

#[svnexus_macro::messagepack_command]
pub async fn subversion_get_wc_root(id: u64, path: String) -> error::Result<String> {
    let subversion = Subversion::instance().lock().await;
    let context = subversion.get(id)?;

    drop(subversion);

    let root = context.get_wc_root(path).await?;

    Ok(root)
}

#[svnexus_macro::messagepack_command]
pub async fn subversion_cancel(id: u64, msg: String) -> error::Result<()> {
    let subversion = Subversion::instance().lock().await;

    let notifier = subversion.get_notifier(id)?;

    *notifier.cancel_msg.lock().await = Some(msg);

    Ok(())
}

#[svnexus_macro::messagepack_command]
pub async fn subversion_wc_revision_status(
    id: u64,
    options: WorkingCopyRevisionStatusOptions,
) -> error::Result<WorkingCopyRevisionStatusResult> {
    let context = context!(id).working_copy_context();
    let result = context.revision_status(options).await?;
    Ok(result)
}

#[svnexus_macro::messagepack_command]
pub async fn subversion_wc_get_replaced_file(
    id: u64,
    path: String,
) -> error::Result<Option<WcReplacedNode>> {
    let context = context!(id);
    let context = context.working_copy_context();
    let result = context.get_replaced_file(path).await?;
    Ok(result)
}

#[svnexus_macro::messagepack_command]
pub async fn subversion_ra_get_location_segments(
    id: u64,
    url: String,
    peg_revision: u32,
    start_revision: u32,
    end_revision: u32,
) -> error::Result<Vec<LocationSegment>> {
    let context = context!(id);

    let context = context.open_repository_access_session(url, None).await?;

    let result = context
        .get_location_segments(String::new(), peg_revision, start_revision, end_revision)
        .await?;
    Ok(result)
}

#[svnexus_macro::messagepack_command]
pub async fn subversion_ra_get_locations(
    id: u64,
    url: String,
    revision: u32,
    location_revisions: Vec<u32>,
) -> error::Result<HashMap<u32, String>> {
    let context = context!(id);

    let context = context.open_repository_access_session(url, None).await?;

    let result = context
        .get_locations(String::new(), revision, location_revisions)
        .await?;
    Ok(result)
}

#[svnexus_macro::messagepack_command]
pub async fn subversion_ra_get_latest_revision_number(
    id: u64,
    url: String,
    path: Option<String>,
) -> error::Result<u32> {
    let context = context!(id);

    let context = context.open_repository_access_session(url, path).await?;

    let result = context.get_latest_revision_number().await?;

    Ok(result)
}

#[svnexus_macro::messagepack_command]
#[tracing::instrument]
pub async fn subversion_log_cache_reverse(
    id: u64,
    repository: String,
    peg_revision: Revision,
    url: String,
    path: String,
    start: u32,
    limit: u32,
) -> error::Result<Vec<LogEntry>> {
    let db = load_database().await?;
    let mut entries = db
        .repository_log_entry_reverse(repository.clone(), path.clone(), start, limit)
        .await?
        .unwrap_or_default();

    if entries.len() < limit as usize {
        let mut should_remove = false;
        let start = if let Some(first) = entries.first() {
            should_remove = true;
            first.revision.unwrap_or(start)
        } else {
            start
        };
        let context = context!(id);

        let options = LogOptions {
            targets: vec![url],
            peg_revision,
            limit: limit - entries.len() as u32,
            revisions: vec![RevisionRange::new(Revision::Head, Revision::Number(start))],
            discover_changed_paths: true,
            strict_node_history: false,
            include_merged_revisions: false,
            revisions_properties: None,
        };

        let mut log_entries = context.log(options).await?.entries;

        log_entries.sort_by(|a, b| b.revision.cmp(&a.revision));

        if should_remove && matches!(log_entries.last(), Some(e) if e.revision == Some(start)) {
            log_entries.remove(log_entries.len() - 1);
        }

        entries = log_entries.clone().also_apply(|e| {
            e.extend(entries);
        });

        db.update_repository_log(repository, path, log_entries)
            .await?;
    }
    Ok(entries)
}

// #[derive(Serialize, Deserialize, Debug, ts_rs::TS)]
// #[ts(export)]
// #[serde(rename_all = "camelCase")]
// pub struct LogCacheOptions {
//     repository: String,
//     peg_revision: Revision,
//     url: String,
//     path: String,
//     start: Option<u32>,
//     limit: u32,
// }

// #[derive(Serialize, Deserialize, Debug, ts_rs::TS)]
// #[ts(export)]
// #[serde(rename_all = "camelCase")]
// pub struct LogCacheReverseOptions {
//     repository: String,
//     peg_revision: Revision,
//     url: String,
//     path: String,
//     start: u32,
//     limit: u32,
// }

#[svnexus_macro::messagepack_command]
#[tracing::instrument]
pub async fn subversion_log_cache(
    id: u64,
    repository: String,
    peg_revision: Revision,
    url: String,
    path: String,
    start: Option<u32>,
    limit: u32,
) -> error::Result<Vec<LogEntry>> {
    let db = load_database().await?;
    let mut entries = db
        .repository_log_entry(
            repository.clone(),
            path.clone(),
            start.unwrap_or(u32::MAX),
            limit,
        )
        .await?
        .unwrap_or_default();

    tracing::info!("got entries from database: {}", entries.len());

    if entries.len() < limit as usize {
        let revision = entries.last().map(|v| v.revision).unwrap_or(start);
        let mut should_remove = None;
        let revision = match revision {
            Some(number) => {
                if number == 0 {
                    tracing::info!("No more revisions");
                    return Ok(entries);
                }
                should_remove = Some(number);
                Revision::Number(number)
            }
            None => Revision::Head,
        };

        tracing::info!(
            "Try to get more revision: from={:?}, peg={:?}",
            revision,
            peg_revision
        );

        let context = context!(id);

        let options = LogOptions {
            targets: vec![url],
            peg_revision,
            limit: limit - entries.len() as u32,
            revisions: vec![RevisionRange::new(revision, Revision::Number(0))],
            discover_changed_paths: true,
            strict_node_history: false,
            include_merged_revisions: false,
            revisions_properties: None,
        };

        let mut log_entries = context.log(options).await?.entries;

        if let Some(number) = should_remove {
            if let Some(first) = log_entries.first() {
                if first.revision == Some(number) {
                    log_entries.remove(0);
                }
            }
        }

        entries.extend(log_entries.clone());

        db.update_repository_log(repository, path, log_entries)
            .await?;
    }

    tracing::info!("return entries: {}", entries.len());

    Ok(entries)
}

#[svnexus_macro::messagepack_command]
pub async fn subversion_time_from_string(time: &str) -> error::Result<i64> {
    subversion::time_from_string(time)
}

#[derive(Default)]
pub struct Reply {
    next: u64,
    pending: HashMap<u64, oneshot::Sender<ReplyMessage>>,
}

#[derive(Debug, Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum ReplyMessage {
    Success(#[ts(type = "Uint8Array")] Vec<u8>),
    Failure(error::FrontendError),
}

impl Reply {
    fn instance() -> &'static Mutex<Reply> {
        static INSTANCE: OnceLock<Mutex<Reply>> = OnceLock::new();
        INSTANCE.get_or_init(|| Default::default())
    }

    fn clear(&mut self) {
        self.next = 0;
        self.pending.clear();
    }

    fn insert(&mut self, sender: oneshot::Sender<ReplyMessage>) -> u64 {
        self.next = self.next.wrapping_add(1);

        self.pending.insert(self.next, sender);

        self.next
    }

    fn reply(&mut self, id: u64, value: ReplyMessage) -> error::Result<()> {
        let Some(sender) = self.pending.remove(&id) else {
            return builder::General {
                detail: "Not found sender",
            }
            .fail();
        };

        if sender.send(value).is_err() {
            return builder::General {
                detail: "Failed to send",
            }
            .fail();
        }

        Ok(())
    }
}

#[svnexus_macro::messagepack_command]
pub async fn reply(id: u64, value: ReplyMessage) -> error::Result<()> {
    let mut instance = Reply::instance().lock().await;
    instance.reply(id, value)
}

#[svnexus_macro::messagepack_command]
pub async fn reload() {
    let mut subversion = Subversion::instance().lock().await;
    subversion.clear();

    let mut instance = Reply::instance().lock().await;
    instance.clear();
}

#[svnexus_macro::messagepack_command]
pub fn format_size(size: u64) -> String {
    humansize::format_size(size, humansize::DECIMAL)
}

#[svnexus_macro::messagepack_command]
pub fn base64_encode(data: &[u8], break_lines: bool) -> error::Result<String> {
    crate::subversion::utils::base64_encode(data, break_lines)
}

#[svnexus_macro::messagepack_command]
pub fn base64_decode(data: &str) -> serde_bytes::ByteBuf {
    serde_bytes::ByteBuf::from(crate::subversion::utils::base64_decode(data))
}

#[svnexus_macro::messagepack_command]
pub fn extended_version(verbose: bool) -> version::ExtendedVersion {
    version::extended_version(verbose)
}

#[svnexus_macro::messagepack_command]
pub async fn open_in_external_application(
    app: platform::ExternalApplication,
    path: Option<String>,
) -> error::Result<()> {
    tokio::task::spawn_blocking(move || {
        let current = platform::current();
        current.open_external_app(app, path)
    })
    .await
    .context(builder::Runtime)?
}
