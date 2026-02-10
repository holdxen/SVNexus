use super::ffi;
use super::{SVNError, svn_no_error};
use crate::apr;
use crate::apr::char_array_to_string;
use crate::error::{self, builder};
use crate::utils::Pointer;
use crate::utils::SubversionStringer;
use crate::utils::{Boxed, CStringer};
use core::panic;
use derive_new::new;
use snafu::ResultExt;
use std::collections::HashMap;
use std::ffi::{CStr, c_char, c_void};
use std::mem::ManuallyDrop;
use std::sync::Arc;

pub struct WorkingCopyContext {
    context: *mut ffi::svn_wc_context_t
}

pub type RevisionNumber = u32;

#[easy_ext::ext]
pub impl ffi::svn_revnum_t {
    fn into_optional_revision(self) -> Option<RevisionNumber> {
        if self < 0 {
            None
        } else {
            Some(self.try_into().unwrap())
        }
    }
}

#[derive(uniffi::Object)]
pub struct AsyncContext {
    context: Arc<parking_lot::Mutex<Context>>,
}

impl AsyncContext {
    async fn call_async<F, R>(&self, call: F) -> error::Result<R>
    where
        F: (FnOnce(parking_lot::MutexGuard<'_, Context>) -> error::Result<R>) + Send + 'static,
        R: Send + 'static,
    {
        let context = self.context.clone();
        let result = tokio::task::spawn_blocking(move || {
            let context = context.lock();
            call(context)
        })
        .await
        .context(builder::Runtime)??;

        Ok(result)
    }
}

#[uniffi::export(async_runtime = "tokio")]
impl AsyncContext {
    pub async fn checkout(&self, opts: CheckoutOptions) -> error::Result<RevisionNumber> {
        println!("opts: {:#?}", opts);
        println!("debug:{}:{}", file!(), line!());
        let context = self.context.clone();
        println!("debug:{}:{}", file!(), line!());
        let revision = tokio::task::spawn_blocking(move || {
            println!("debug:{}:{}", file!(), line!());
            let mut context = context.lock();
            println!("debug:{}:{}", file!(), line!());
            context.checkout(opts)
        })
        .await
        .context(builder::Runtime)??;
        println!("debug:{}:{}", file!(), line!());

        Ok(revision)
    }

    pub async fn status(&self, opts: StatusOptions) -> error::Result<StatusResult> {
        let context = self.context.clone();

        let entries = tokio::task::spawn_blocking(move || {
            let mut context = context.lock();

            context.status(opts)
        })
        .await
        .context(builder::Runtime)??;

        Ok(entries)
    }

    pub async fn add(&self, opts: AddOptions) -> error::Result<()> {
        let context = self.context.clone();

        tokio::task::spawn_blocking(move || {
            let mut context = context.lock();
            context.add(&opts)
        })
        .await
        .context(builder::Runtime)??;

        Ok(())
    }

    pub async fn commit(&self, opts: CommitOptions) -> error::Result<CommitResult> {
        let context = self.context.clone();

        let result = tokio::task::spawn_blocking(move || {
            let mut context = context.lock();

            context.commit(opts)
        })
        .await
        .context(builder::Runtime)??;

        Ok(result)
    }

    pub async fn cat(&self, opts: CatOptions) -> error::Result<CatResult> {
        self.call_async(|mut context| context.cat(opts)).await
    }

    pub async fn delete(&self, opts: DeleteOptions) -> error::Result<DeleteResult> {
        self.call_async(|mut context| context.delete(opts)).await
    }

    pub async fn revert(&self, opts: RevertOptions) -> error::Result<()> {
        self.call_async(|mut context| context.revert(opts)).await
    }

    pub async fn log(&self, opts: LogOptions) -> error::Result<LogResult> {
        self.call_async(|mut context| context.log(opts)).await
    }

    pub async fn import(&self, opts: ImportOptions) -> error::Result<ImportResult> {
        self.call_async(|mut context| context.import(opts)).await
    }

    #[uniffi::constructor]
    pub fn create(opts: CreateContextOptions) -> error::Result<Self> {
        eprintln!("debug:{}:{}", file!(), line!());
        // let context = Context::create(opts)?;
        let context = ContextFactory::instance()?.create_context(opts)?;
        eprintln!("debug:{}:{}", file!(), line!());

        let context = Arc::new(parking_lot::Mutex::new(context));
        eprintln!("debug:{}:{}", file!(), line!());

        Ok(AsyncContext { context })
    }
}

#[uniffi::export(with_foreign)]
pub trait ContextNotifier: Send + Sync + 'static {
    fn may_save_password_as_plain_text(&self, realm_string: String) -> bool;
    fn working_copy_notify(&self, notify: WorkingCopyNotify);
    fn ssl_server_trust_prompt(
        &self,
        realm: String,
        failures: u32,
        info: SslServerCertInfo,
        may_save: bool,
    ) -> Option<TrustServer>;
    fn cancel(&self) -> Option<String>;
    fn progress_notify(&self, pos: i64, total: i64);
    fn authenticate(
        &self,
        realm: String,
        username: String,
        may_save: bool,
    ) -> Option<Authentication>;
}

#[derive(new)]
pub struct ContextInner {
    #[new(default)]
    commit_message: String,
    #[new(default)]
    commit_items: Vec<CommitItem>,
    #[new(default)]
    commit_info: Option<CommitInfo>,
    // #[new(default)]
    // cancel: Arc<Mutex<Option<String>>>,
    #[new(default)]
    status_entries: Vec<StatusEntry>,
    context_notifier: Arc<dyn ContextNotifier>,
    // on_may_save_password_as_plain_text: Box<dyn Fn(&str) -> bool + Send>,
    // on_notify: Box<dyn Fn(Notify) + Send>,
    // on_ssl_server_trust_prompt:
    //     Box<dyn Fn(&str, SslFailures, SslServerCertInfo, bool) -> Option<TrustServer> + Send>,
    // on_cancel: Box<dyn Fn() -> Option<String> + Send>,
    // on_progress_notify: Box<dyn Fn(i64, i64) + Send>,
    // on_authenticate: Box<dyn Fn(String, String, bool) -> Option<Authentication> + Send>,

    // revision_statck: Vec<i32>,
    #[new(default)]
    log_entries: Vec<LogEntry>,

    #[new(default)]
    info_entries: HashMap<String, InfoEntry>,
}

// impl Default for ContextInner {
//     fn default() -> Self {
//         Self {
//             commit_message: Default::default(),
//             commit_items: Default::default(),
//             commit_info: Default::default(),
//             // cancel: Default::default(),
//             status_entries: Default::default(),
//             on_may_save_password_as_plain_text: Box::new(|_| false),
//             on_notify: Box::new(|_| ()),
//             on_ssl_server_trust_prompt: Box::new(|_, _, _, _| Default::default()),
//             on_cancel: Box::new(|| None),
//             on_progress_notify: Box::new(|_, _| ()),
//             on_authenticate: Box::new(|_, _, _| None),
//             // revision_statck: Default::default(),
//             log_entries: Default::default(),
//         }
//     }
// }

pub struct Context {
    ptr: *mut ffi::svn_client_ctx_t,
    pool: apr::Pool,
    inner: Box<ContextInner>,
}

unsafe impl Send for Context {}

// #[derive(Default)]
#[derive(uniffi::Record)]
pub struct CreateContextOptions {
    pub name: Option<String>,
    pub default_username: Option<String>,
    pub default_password: Option<String>,
    pub context_notifier: Arc<dyn ContextNotifier>,
}

#[derive(new, uniffi::Record)]
pub struct CommitResult {
    items: Vec<CommitItem>,
    info: CommitInfo,
}

#[svnexus_macro::enum_converter(repr_type=ffi::svn_node_kind_t)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, uniffi::Enum)]
pub enum NodeKind {
    None = ffi::svn_node_kind_t_svn_node_none,
    File = ffi::svn_node_kind_t_svn_node_file,
    Directory = ffi::svn_node_kind_t_svn_node_dir,
    Unknown = ffi::svn_node_kind_t_svn_node_unknown,
    Symlink = ffi::svn_node_kind_t_svn_node_symlink,
}

#[derive(uniffi::Record, Debug)]
pub struct CommitItem {
    path: String,
    kind: NodeKind,
    url: String,
    revision: RevisionNumber,
    copy_from_url: Option<String>,
    copy_from_revision: Option<RevisionNumber>,
    state: u8,
    incoming_property_changes: HashMap<String, String>, // properties: HashMap<String, String>,
    outgoing_property_changes: HashMap<String, String>,
    session_real_path: Option<String>,
    move_from_absolute_path: Option<String>,
}

#[svnexus_macro::enum_converter(repr_type=ffi::svn_wc_notify_state_t)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, uniffi::Enum)]
pub enum WorkingCopyNotifyState {
    Inapplicable = ffi::svn_wc_notify_state_t_svn_wc_notify_state_inapplicable,
    Unknown = ffi::svn_wc_notify_state_t_svn_wc_notify_state_unknown,
    Unchanged = ffi::svn_wc_notify_state_t_svn_wc_notify_state_unchanged,
    Missing = ffi::svn_wc_notify_state_t_svn_wc_notify_state_missing,
    Obstructed = ffi::svn_wc_notify_state_t_svn_wc_notify_state_obstructed,
    Changed = ffi::svn_wc_notify_state_t_svn_wc_notify_state_changed,
    Merged = ffi::svn_wc_notify_state_t_svn_wc_notify_state_merged,
    Conflicted = ffi::svn_wc_notify_state_t_svn_wc_notify_state_conflicted,
    SourceMissing = ffi::svn_wc_notify_state_t_svn_wc_notify_state_source_missing,
}

#[svnexus_macro::enum_converter(repr_type=ffi::svn_wc_notify_lock_state_t)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, uniffi::Enum)]
// #[repr(u32)]
pub enum WorkingCopyNotifyLockState {
    InApplicable = ffi::svn_wc_notify_lock_state_t_svn_wc_notify_lock_state_inapplicable,
    Unknown = ffi::svn_wc_notify_lock_state_t_svn_wc_notify_lock_state_unknown,
    Unchanged = ffi::svn_wc_notify_lock_state_t_svn_wc_notify_lock_state_unchanged,
    Locked = ffi::svn_wc_notify_lock_state_t_svn_wc_notify_lock_state_locked,
    Unlocked = ffi::svn_wc_notify_lock_state_t_svn_wc_notify_lock_state_unlocked,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, uniffi::Record)]
pub struct MergeRange {
    start: i64,
    end: i64,
    inheritable: bool,
}

impl MergeRange {
    unsafe fn from_raw(ptr: *const ffi::svn_merge_range_t) -> Self {
        unsafe {
            let ptr = &*ptr;

            Self {
                start: ptr.start.try_into().unwrap(),
                end: ptr.end.try_into().unwrap(),
                inheritable: ptr.inheritable != 0,
            }
        }
    }
}

pub type LineNumber = u32;

#[derive(Debug, Clone, uniffi::Record)]
pub struct WorkingCopyNotify {
    pub path: String,
    pub action: WorkingCopyNotifyAction,
    pub kind: NodeKind,
    pub mime_type: Option<String>,
    pub lock: Option<Lock>,
    pub err: Option<SVNError>,
    pub content_state: WorkingCopyNotifyState,
    pub property_state: WorkingCopyNotifyState,
    pub lock_state: WorkingCopyNotifyLockState,
    pub revision: Option<RevisionNumber>,
    pub changelist_name: Option<String>,
    pub merge_range: Option<MergeRange>,
    pub url: Option<String>,
    pub path_prefix: Option<String>,
    pub property_name: Option<String>,
    pub revision_properties: HashMap<String, String>,
    pub old_revision: Option<RevisionNumber>,
    pub hunk_original_start: u32,
    pub hunk_original_length: u32,
    pub hunk_modified_start: u32,
    pub hunk_modified_length: u32,

    pub hunk_matched_line: u32,

    pub hunk_fuzz: u32,
}

impl WorkingCopyNotify {
    unsafe fn from_raw(ptr: *const ffi::svn_wc_notify_t) -> Self {
        unsafe {
            let ptr = ptr.as_ref().unwrap();

            let path = apr::char_array_to_string(ptr.path).unwrap();

            let action = WorkingCopyNotifyAction::try_from(ptr.action)
                .expect(format!("Invalid notify action: {}", ptr.action).as_str());

            let kind = NodeKind::try_from(ptr.kind)
                .expect(format!("Invalid node kind: {}", ptr.kind).as_str());

            let mime_type = apr::char_array_to_string(ptr.mime_type);

            // let lock = if ptr.lock.is_null() {
            //     None
            // } else {
            //     Some(Lock::from_ptr(ptr.lock))
            // };

            let lock = ptr.lock.as_ref().map(|v| Lock::from_ptr(v));

            let err = ptr
                .err
                .as_ref()
                .map(|v| SVNError::from_not_null_ptr(ptr.err));

            // let err = if ptr.err.is_null() {
            //     None
            // } else {
            //     Some(Error::from_not_null_ptr(ptr.err))
            // };

            let content_state = WorkingCopyNotifyState::try_from(ptr.content_state).unwrap();

            let property_state = WorkingCopyNotifyState::try_from(ptr.prop_state).unwrap();

            let lock_state = WorkingCopyNotifyLockState::try_from(ptr.lock_state).unwrap();

            println!("WorkingCopyNotify revision: {}", ptr.revision);
            let revision = if ptr.revision < 0 {
                None
            } else {
                Some(ptr.revision.try_into().unwrap())
            };

            let changelist_name = apr::char_array_to_string(ptr.changelist_name);

            let merge_range = if ptr.merge_range.is_null() {
                None
            } else {
                Some(MergeRange::from_raw(ptr.merge_range))
            };

            let url = apr::char_array_to_string(ptr.url);

            let path_prefix = if ptr.path_prefix.is_null() {
                None
            } else {
                Some(apr::char_array_to_string(ptr.path_prefix).unwrap())
            };

            let property_name = if ptr.prop_name.is_null() {
                None
            } else {
                Some(apr::char_array_to_string(ptr.prop_name).unwrap())
            };

            let revision_properties = HashMap::default();

            let old_revision = if ptr.old_revision < 0 {
                None
            } else {
                Some(ptr.old_revision.try_into().unwrap())
            };

            let hunk_original_start = ptr.hunk_original_start.try_into().unwrap();

            let hunk_original_length = ptr.hunk_original_length.try_into().unwrap();

            let hunk_modified_start = ptr.hunk_modified_start.try_into().unwrap();

            let hunk_modified_length = ptr.hunk_modified_length.try_into().unwrap();

            let hunk_matched_line = ptr.hunk_matched_line.try_into().unwrap();

            let hunk_fuzz = ptr.hunk_fuzz.try_into().unwrap();

            Self {
                path,
                action,
                kind,
                mime_type,
                lock,
                err,
                content_state,
                property_state,
                lock_state,
                revision,
                changelist_name,
                merge_range,
                url,
                path_prefix,
                property_name,
                revision_properties,
                old_revision,
                hunk_original_start,
                hunk_original_length,
                hunk_modified_start,
                hunk_modified_length,
                hunk_matched_line,
                hunk_fuzz,
            }
        }
    }
}

#[svnexus_macro::enum_converter(repr_type=ffi::svn_wc_notify_action_t)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, uniffi::Enum)]
pub enum WorkingCopyNotifyAction {
    Add = ffi::svn_wc_notify_action_t_svn_wc_notify_add,
    Copy = ffi::svn_wc_notify_action_t_svn_wc_notify_copy,
    Delete = ffi::svn_wc_notify_action_t_svn_wc_notify_delete,
    Restore = ffi::svn_wc_notify_action_t_svn_wc_notify_restore,
    Revert = ffi::svn_wc_notify_action_t_svn_wc_notify_revert,
    FailedRevert = ffi::svn_wc_notify_action_t_svn_wc_notify_failed_revert,
    Resolved = ffi::svn_wc_notify_action_t_svn_wc_notify_resolved,
    Skip = ffi::svn_wc_notify_action_t_svn_wc_notify_skip,
    UpdateDelete = ffi::svn_wc_notify_action_t_svn_wc_notify_update_delete,
    UpdateAdd = ffi::svn_wc_notify_action_t_svn_wc_notify_update_add,
    UpdateUpdate = ffi::svn_wc_notify_action_t_svn_wc_notify_update_update,
    UpdateCompleted = ffi::svn_wc_notify_action_t_svn_wc_notify_update_completed,
    UpdateExternal = ffi::svn_wc_notify_action_t_svn_wc_notify_update_external,
    StatusCompleted = ffi::svn_wc_notify_action_t_svn_wc_notify_status_completed,
    StatusExternal = ffi::svn_wc_notify_action_t_svn_wc_notify_status_external,
    CommitModified = ffi::svn_wc_notify_action_t_svn_wc_notify_commit_modified,
    CommitDeleted = ffi::svn_wc_notify_action_t_svn_wc_notify_commit_deleted,
    CommitReplaced = ffi::svn_wc_notify_action_t_svn_wc_notify_commit_replaced,
    CommitPostfixTxDelta = ffi::svn_wc_notify_action_t_svn_wc_notify_commit_postfix_txdelta,
    BlameRevision = ffi::svn_wc_notify_action_t_svn_wc_notify_blame_revision,
    Locked = ffi::svn_wc_notify_action_t_svn_wc_notify_locked,
    Unlocked = ffi::svn_wc_notify_action_t_svn_wc_notify_unlocked,
    FailedLock = ffi::svn_wc_notify_action_t_svn_wc_notify_failed_lock,
    FailedUnlock = ffi::svn_wc_notify_action_t_svn_wc_notify_failed_unlock,
    Exists = ffi::svn_wc_notify_action_t_svn_wc_notify_exists,
    ChangelistSet = ffi::svn_wc_notify_action_t_svn_wc_notify_changelist_set,
    ChangelistClear = ffi::svn_wc_notify_action_t_svn_wc_notify_changelist_clear,
    ChangelistMoved = ffi::svn_wc_notify_action_t_svn_wc_notify_changelist_moved,
    MergeBegin = ffi::svn_wc_notify_action_t_svn_wc_notify_merge_begin,
    ForeignMergeBegin = ffi::svn_wc_notify_action_t_svn_wc_notify_foreign_merge_begin,
    UpdateReplace = ffi::svn_wc_notify_action_t_svn_wc_notify_update_replace,
    PropertyAdded = ffi::svn_wc_notify_action_t_svn_wc_notify_property_added,
    PropertyDeleted = ffi::svn_wc_notify_action_t_svn_wc_notify_property_deleted,
    PropertyDeletedNonexistent =
        ffi::svn_wc_notify_action_t_svn_wc_notify_property_deleted_nonexistent,
    RevisionPropertySet = ffi::svn_wc_notify_action_t_svn_wc_notify_revprop_set,
    RevisionPropertyDeleted = ffi::svn_wc_notify_action_t_svn_wc_notify_revprop_deleted,
    MergeCompleted = ffi::svn_wc_notify_action_t_svn_wc_notify_merge_completed,
    TreeConflict = ffi::svn_wc_notify_action_t_svn_wc_notify_tree_conflict,
    FailedExternal = ffi::svn_wc_notify_action_t_svn_wc_notify_failed_external,
    UpdateStarted = ffi::svn_wc_notify_action_t_svn_wc_notify_update_started,
    UpdateSkipObstruction = ffi::svn_wc_notify_action_t_svn_wc_notify_update_skip_obstruction,
    UpdateSkipWorkingOnly = ffi::svn_wc_notify_action_t_svn_wc_notify_update_skip_working_only,
    UpdateSkipAccessDenied = ffi::svn_wc_notify_action_t_svn_wc_notify_update_skip_access_denied,
    UpdateExternalRemoved = ffi::svn_wc_notify_action_t_svn_wc_notify_update_external_removed,
    UpdateShadowedAdd = ffi::svn_wc_notify_action_t_svn_wc_notify_update_shadowed_add,
    UpdateShadowedUpdate = ffi::svn_wc_notify_action_t_svn_wc_notify_update_shadowed_update,
    UpdateShadowedDelete = ffi::svn_wc_notify_action_t_svn_wc_notify_update_shadowed_delete,
    MergeRecordInfo = ffi::svn_wc_notify_action_t_svn_wc_notify_merge_record_info,
    MergeElideInfo = ffi::svn_wc_notify_action_t_svn_wc_notify_merge_elide_info,
    Patch = ffi::svn_wc_notify_action_t_svn_wc_notify_patch,
    PatchAppliedHunk = ffi::svn_wc_notify_action_t_svn_wc_notify_patch_applied_hunk,
    PatchRejectHunk = ffi::svn_wc_notify_action_t_svn_wc_notify_patch_rejected_hunk,
    PatchHunkAlreadyApplied = ffi::svn_wc_notify_action_t_svn_wc_notify_patch_hunk_already_applied,
    CommitCopied = ffi::svn_wc_notify_action_t_svn_wc_notify_commit_copied,
    CommitCopiedReplaced = ffi::svn_wc_notify_action_t_svn_wc_notify_commit_copied_replaced,
    UrlRedirect = ffi::svn_wc_notify_action_t_svn_wc_notify_url_redirect,
    PathNonexistent = ffi::svn_wc_notify_action_t_svn_wc_notify_path_nonexistent,
    Exclude = ffi::svn_wc_notify_action_t_svn_wc_notify_exclude,
    FailedConflict = ffi::svn_wc_notify_action_t_svn_wc_notify_failed_conflict,
    FailedMissing = ffi::svn_wc_notify_action_t_svn_wc_notify_failed_missing,
    FailedOutOfDate = ffi::svn_wc_notify_action_t_svn_wc_notify_failed_out_of_date,
    FailedNoParent = ffi::svn_wc_notify_action_t_svn_wc_notify_failed_no_parent,
    FailedLocked = ffi::svn_wc_notify_action_t_svn_wc_notify_failed_locked,
    FailedForbiddenByServer = ffi::svn_wc_notify_action_t_svn_wc_notify_failed_forbidden_by_server,
    SkipConflicted = ffi::svn_wc_notify_action_t_svn_wc_notify_skip_conflicted,
    UpdateBrokenLock = ffi::svn_wc_notify_action_t_svn_wc_notify_update_broken_lock,
    FailedObstruction = ffi::svn_wc_notify_action_t_svn_wc_notify_failed_obstruction,
    ConflictResolverDone = ffi::svn_wc_notify_action_t_svn_wc_notify_conflict_resolver_done,
    LeftLocalModifications = ffi::svn_wc_notify_action_t_svn_wc_notify_left_local_modifications,
    ForeignCopyBegin = ffi::svn_wc_notify_action_t_svn_wc_notify_foreign_copy_begin,
    MoveBroken = ffi::svn_wc_notify_action_t_svn_wc_notify_move_broken,
    CleanupExternal = ffi::svn_wc_notify_action_t_svn_wc_notify_cleanup_external,
    FailedRequiresTarget = ffi::svn_wc_notify_action_t_svn_wc_notify_failed_requires_target,
    InfoExternal = ffi::svn_wc_notify_action_t_svn_wc_notify_info_external,
    CommitFinalizing = ffi::svn_wc_notify_action_t_svn_wc_notify_commit_finalizing,
    ResolvedText = ffi::svn_wc_notify_action_t_svn_wc_notify_resolved_text,
    ResolvedProp = ffi::svn_wc_notify_action_t_svn_wc_notify_resolved_prop,
    ResolvedTree = ffi::svn_wc_notify_action_t_svn_wc_notify_resolved_tree,
    BeginSearchTreeConflictDetails =
        ffi::svn_wc_notify_action_t_svn_wc_notify_begin_search_tree_conflict_details,
    TreeConflictDetailsProgress =
        ffi::svn_wc_notify_action_t_svn_wc_notify_tree_conflict_details_progress,
    EndSearchTreeConflictDetails =
        ffi::svn_wc_notify_action_t_svn_wc_notify_end_search_tree_conflict_details,
    HydratingStart = ffi::svn_wc_notify_action_t_svn_wc_notify_hydrating_start,
    HydratingFile = ffi::svn_wc_notify_action_t_svn_wc_notify_hydrating_file,
    HydratingEnd = ffi::svn_wc_notify_action_t_svn_wc_notify_hydrating_end,
    Warning = ffi::svn_wc_notify_action_t_svn_wc_notify_warning,
    RevertNoAccess = ffi::svn_wc_notify_action_t_svn_wc_notify_revert_noaccess,
}

// #[repr(i32)]
#[svnexus_macro::enum_converter(repr_type=ffi::svn_depth_t)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, uniffi::Enum)]
pub enum Depth {
    Unknown = ffi::svn_depth_t_svn_depth_unknown,
    Exclude = ffi::svn_depth_t_svn_depth_exclude,
    Empty = ffi::svn_depth_t_svn_depth_empty,
    Files = ffi::svn_depth_t_svn_depth_files,
    Immediate = ffi::svn_depth_t_svn_depth_immediates,
    Infinity = ffi::svn_depth_t_svn_depth_infinity,
}

#[derive(uniffi::Record, Debug)]
pub struct AddOptions {
    path: String,
    depth: Depth,
    force: bool,
    no_ignore: bool,
    no_auto_properties: bool,
    add_parents: bool,
}

#[derive(Debug, uniffi::Record)]
pub struct StatusEntry {
    path: String,
    node_kind: NodeKind,
    local_absolute_path: String,
    file_size: Option<u64>,
    versioned: bool,
    conflicted: bool,
    node_status: NodeStatus,
    text_status: NodeStatus,
    prop_status: NodeStatus,
    wc_is_locked: bool,
    copied: bool,
    repos_root_url: Option<String>,
    repos_uuid: Option<String>,
    repos_relpath: Option<String>,
    revision: RevisionNumber,
    last_changed_revision: RevisionNumber,
    last_changed_date: i64,
    last_changed_author: Option<String>,
    switched: bool,
    file_external: bool,
    lock: Option<Lock>,
    changelist: Option<String>,
    depth: Depth,
    out_of_date_kind: NodeKind,
    repos_node_status: NodeStatus,
    repos_text_status: NodeStatus,
    repos_prop_status: NodeStatus,
    repos_lock: Option<Lock>,
    out_of_date_changed_revision: Option<RevisionNumber>,
    out_of_date_changed_date: Option<i64>,
    out_of_date_changed_author: Option<String>,
    moved_from_absolute_path: Option<String>,
    moved_to_absolute_path: Option<String>,
}

#[derive(uniffi::Record, Debug)]
pub struct CommitOptions {
    targets: Vec<String>,
    depth: Depth,
    keep_locks: bool,
    keep_changelist: bool,
    commit_as_operations: bool,
    include_file_externals: bool,
    include_dir_externals: bool,
    changelists: Vec<String>,
    revision_property_table: HashMap<String, String>,
    commit_message: String,
}

#[derive(Default, Debug, uniffi::Record)]
pub struct CommitInfo {
    revision: RevisionNumber,
    date: String,
    author: String,
    post_commit_err: Option<String>,
    repos_root: Option<String>,
}

// #[repr(u32)]
// #[derive(Debug, TryFromPrimitive)]
#[svnexus_macro::enum_converter(repr_type=ffi::svn_wc_status_kind)]
#[derive(Debug, uniffi::Enum)]
pub enum NodeStatus {
    None = ffi::svn_wc_status_kind_svn_wc_status_none,
    Unversioned = ffi::svn_wc_status_kind_svn_wc_status_unversioned,
    Normal = ffi::svn_wc_status_kind_svn_wc_status_normal,
    Added = ffi::svn_wc_status_kind_svn_wc_status_added,
    Missing = ffi::svn_wc_status_kind_svn_wc_status_missing,
    Deleted = ffi::svn_wc_status_kind_svn_wc_status_deleted,
    Replaced = ffi::svn_wc_status_kind_svn_wc_status_replaced,
    Modified = ffi::svn_wc_status_kind_svn_wc_status_modified,
    Merged = ffi::svn_wc_status_kind_svn_wc_status_merged,
    Conflicted = ffi::svn_wc_status_kind_svn_wc_status_conflicted,
    Ignored = ffi::svn_wc_status_kind_svn_wc_status_ignored,
    Obstructed = ffi::svn_wc_status_kind_svn_wc_status_obstructed,
    External = ffi::svn_wc_status_kind_svn_wc_status_external,
    Incomplete = ffi::svn_wc_status_kind_svn_wc_status_incomplete,
}

#[derive(Clone, Copy, Debug, uniffi::Enum)]
pub enum Revision {
    Unspecified,
    Number(RevisionNumber),
    Date(i64),
    Committed,
    Previous,
    Base,
    Working,
    Head,
}

#[derive(new, uniffi::Record)]
pub struct DeleteResult {
    info: Option<CommitInfo>,
}

#[derive(Debug, uniffi::Record)]
pub struct DeleteOptions {
    path: Vec<String>,
    force: bool,
    keep_local: bool,
    revprop_table: HashMap<String, String>,
}

#[derive(Debug, uniffi::Record)]
pub struct StatusOptions {
    path: String,
    revision: Revision,
    depth: Depth,
    get_all: bool,
    update: bool,
    check_out_of_date: bool,
    check_working_copy: bool,
    no_ignore: bool,
    ignore_externals: bool,
    depth_as_sticky: bool,
    changelist: Vec<String>,
}

#[derive(new, Debug, uniffi::Record)]
pub struct CheckoutOptions {
    url: String,
    path: String,
    peg_revision: Revision,
    revision: Revision,
    depth: Depth,
    ignore_externals: bool,
    allow_unversioned_obstructions: bool,
    store_pristine: Option<bool>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct Lock {
    path: String,
    token: String,
    owner: String,
    comment: String,
    is_dav_comment: bool,
    creation_date: i64,
    expiration_date: i64,
}

impl Default for Revision {
    fn default() -> Self {
        Self::Unspecified
    }
}

impl Revision {
    fn to_opt_revision(&self) -> ffi::svn_opt_revision_t {
        let mut revision = ffi::svn_opt_revision_t::default();

        match self {
            Revision::Unspecified => {
                revision.kind = ffi::svn_opt_revision_kind_svn_opt_revision_unspecified;
            }
            Revision::Number(number) => {
                revision.kind = ffi::svn_opt_revision_kind_svn_opt_revision_number;
                unsafe {
                    *revision.value.number.as_mut() = (*number).try_into().unwrap();
                }
            }
            Revision::Date(time) => {
                revision.kind = ffi::svn_opt_revision_kind_svn_opt_revision_date;
                unsafe {
                    *revision.value.date.as_mut() = (*time).try_into().unwrap();
                }
            }
            Revision::Committed => {
                revision.kind = ffi::svn_opt_revision_kind_svn_opt_revision_committed;
            }
            Revision::Previous => {
                revision.kind = ffi::svn_opt_revision_kind_svn_opt_revision_previous;
            }
            Revision::Base => {
                revision.kind = ffi::svn_opt_revision_kind_svn_opt_revision_base;
            }
            Revision::Working => {
                revision.kind = ffi::svn_opt_revision_kind_svn_opt_revision_working;
            }
            Revision::Head => {
                revision.kind = ffi::svn_opt_revision_kind_svn_opt_revision_head;
            }
        }

        revision
    }
}

impl CommitInfo {
    unsafe fn from_ptr(info: *const ffi::svn_commit_info_t) -> Self {
        unsafe {
            let info = &*info;
            Self {
                revision: info.revision.try_into().unwrap(),
                date: apr::char_array_to_string(info.date).unwrap(),
                author: apr::char_array_to_string(info.author).unwrap(),
                post_commit_err: apr::char_array_to_string(info.post_commit_err),
                repos_root: apr::char_array_to_string(info.repos_root),
            }
        }
    }
}

unsafe extern "C" fn commit_callback(
    commit_info: *const ffi::svn_commit_info_t,
    baton: *mut c_void,
    pool: *mut ffi::apr_pool_t,
) -> *mut ffi::svn_error_t {
    unsafe {
        let error = on_cancel(baton);
        if !error.is_null() {
            return error;
        }
        let commit_info = CommitInfo::from_ptr(commit_info);

        let this = &mut *(baton as *mut ContextInner);

        this.commit_info = Some(commit_info);

        svn_no_error()
    }
}

impl Lock {
    fn from_ptr(ptr: *const ffi::svn_lock_t) -> Self {
        unsafe {
            let lock = ptr.as_ref().unwrap();
            let path = apr::char_array_to_string(lock.path).unwrap();

            let token = apr::char_array_to_string(lock.token).unwrap();

            let owner = apr::char_array_to_string(lock.owner).unwrap();

            let comment = apr::char_array_to_string(lock.comment).unwrap();

            let is_dav_comment = lock.is_dav_comment != 0;

            let creation_date: i64 = lock.creation_date.try_into().unwrap();

            let expiration_date: i64 = lock.expiration_date.try_into().unwrap();
            Self {
                path,
                token,
                owner,
                comment,
                is_dav_comment,
                creation_date,
                expiration_date,
            }
        }
    }
}

impl StatusEntry {
    unsafe fn from_path_and_ptr(
        path: *const c_char,
        entry: *const ffi::svn_client_status_t,
    ) -> Self {
        unsafe {
            let path = apr::char_array_to_string(path).unwrap();

            let entry = &*entry;

            let node_kind = NodeKind::try_from(entry.kind).unwrap();

            let local_absolute_path = apr::char_array_to_string(entry.local_abspath).unwrap();

            // let file_size: u64 = entry.filesize.try_into().unwrap();
            let file_size = if entry.filesize == -1 {
                None
            } else {
                Some(entry.filesize.try_into().unwrap())
            };

            let versioned = entry.versioned != 0;

            let conflicted = entry.conflicted != 0;

            let node_status = NodeStatus::try_from(entry.node_status).unwrap();

            let text_status = NodeStatus::try_from(entry.text_status).unwrap();

            let prop_status = NodeStatus::try_from(entry.prop_status).unwrap();

            let wc_is_locked = entry.wc_is_locked != 0;

            let copied = entry.copied != 0;

            let repos_root_url = apr::char_array_to_string(entry.repos_root_url);

            let repos_uuid = apr::char_array_to_string(entry.repos_uuid);

            let repos_relpath = apr::char_array_to_string(entry.repos_relpath);

            let revision = RevisionNumber::try_from(entry.revision).unwrap();

            let last_changed_revision: RevisionNumber = entry.changed_rev.try_into().unwrap();

            let last_changed_date: i64 = entry.changed_date.try_into().unwrap();

            let last_changed_author = apr::char_array_to_string(entry.changed_author);

            let switched = entry.switched != 0;

            let file_external = entry.file_external != 0;

            let lock = if entry.lock.is_null() {
                None
            } else {
                Some(Lock::from_ptr(entry.lock))
            };

            let changelist = apr::char_array_to_string(entry.changelist);

            let depth = Depth::try_from(entry.depth).unwrap();

            let out_of_date_kind = NodeKind::try_from(entry.ood_kind).unwrap();

            let repos_node_status = NodeStatus::try_from(entry.repos_node_status).unwrap();

            let repos_text_status = NodeStatus::try_from(entry.repos_text_status).unwrap();

            let repos_prop_status = NodeStatus::try_from(entry.repos_prop_status).unwrap();

            let repos_lock = if entry.repos_lock.is_null() {
                None
            } else {
                Some(Lock::from_ptr(entry.repos_lock))
            };

            let out_of_date_changed_revision = if entry.ood_changed_rev == -1 {
                None
            } else {
                Some(entry.ood_changed_rev.try_into().unwrap())
            };

            let out_of_date_changed_date = if entry.ood_changed_date == 0 {
                None
            } else {
                Some(entry.ood_changed_date.try_into().unwrap())
            };

            let out_of_date_changed_author = apr::char_array_to_string(entry.ood_changed_author);

            let moved_from_absolute_path = apr::char_array_to_string(entry.moved_from_abspath);

            let moved_to_absolute_path = apr::char_array_to_string(entry.moved_to_abspath);

            Self {
                path,
                node_kind,
                local_absolute_path,
                file_size,
                versioned,
                conflicted,
                node_status,
                text_status,
                prop_status,
                wc_is_locked,
                copied,
                repos_root_url,
                repos_uuid,
                repos_relpath,
                revision,
                last_changed_revision,
                last_changed_date,
                last_changed_author,
                switched,
                file_external,
                lock,
                changelist,
                depth,
                out_of_date_kind,
                repos_node_status,
                repos_text_status,
                repos_prop_status,
                repos_lock,
                out_of_date_changed_revision,
                out_of_date_changed_date,
                out_of_date_changed_author,
                moved_from_absolute_path,
                moved_to_absolute_path,
            }
        }
    }
}

impl Default for NodeStatus {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, new, uniffi::Record)]
pub struct StatusResult {
    entries: Vec<StatusEntry>,
}

#[derive(new, Debug, uniffi::Record)]
pub struct LogResult {
    log_entries: Vec<LogEntry>,
}

impl Default for NodeKind {
    fn default() -> Self {
        NodeKind::None
    }
}

impl CommitItem {
    unsafe fn from_items(items: *const ffi::apr_array_header_t) -> Vec<Self> {
        let items = unsafe { &*items };

        let mut ret = Vec::with_capacity(items.nelts.try_into().unwrap());

        for i in 0..items.nelts {
            let elements = items.elts as *const *const ffi::svn_client_commit_item3_t;

            let element = unsafe {
                let offset = elements.offset(i as _);
                if (*offset).is_null() {
                    continue;
                }
                *offset
            };

            let item = unsafe { Self::from_item(element) };

            ret.push(item);
        }

        ret
    }

    unsafe fn from_item(item: *const ffi::svn_client_commit_item3_t) -> Self {
        unsafe {
            let item = &*item;

            let path = apr::char_array_to_string(item.path).unwrap();

            let kind = NodeKind::try_from(item.kind).unwrap();

            let url = apr::char_array_to_string(item.url).unwrap();

            let copy_from_url = apr::char_array_to_string(item.copyfrom_url);

            let mut incoming_property_changes = HashMap::new();
            if !item.incoming_prop_changes.is_null() {
                let prop = &*item.incoming_prop_changes;

                incoming_property_changes.reserve(prop.nelts as _);

                for i in 0..prop.nelts {
                    let elements = prop.elts as *const *const ffi::svn_prop_t;

                    let offset = elements.offset(i as _);
                    if (*offset).is_null() {
                        continue;
                    }
                    let element = &**offset;

                    let key = apr::char_array_to_string(element.name).unwrap();

                    let slice = std::slice::from_raw_parts(
                        (*element.value).data as *const u8,
                        (*element.value).len,
                    );
                    let value = String::from_utf8(slice.to_vec()).unwrap();

                    incoming_property_changes.insert(key, value);
                }
            }

            let mut outgoing_property_changes = HashMap::new();
            if !item.outgoing_prop_changes.is_null() {
                let prop = &*item.outgoing_prop_changes;

                outgoing_property_changes.reserve(prop.nelts as _);

                for i in 0..prop.nelts {
                    let elements = prop.elts as *const *const ffi::svn_prop_t;

                    let offset = elements.offset(i as _);
                    if (*offset).is_null() {
                        continue;
                    }
                    let element = &**offset;

                    let key = apr::char_array_to_string(element.name).unwrap();

                    let slice = std::slice::from_raw_parts(
                        (*element.value).data as *const u8,
                        (*element.value).len,
                    );
                    let value = String::from_utf8(slice.to_vec()).unwrap();

                    outgoing_property_changes.insert(key, value);
                }
            }

            let revision = item.revision.try_into().unwrap();
            let state = item.state_flags;

            let session_real_path = apr::char_array_to_string(item.session_relpath);

            let copy_from_revision = if item.copyfrom_url.is_null() || item.copyfrom_rev < 0 {
                None
            } else {
                Some(item.copyfrom_rev.try_into().unwrap())
            };

            let move_from_absolute_path = apr::char_array_to_string(item.moved_from_abspath);

            Self {
                path,
                kind,
                url,
                revision,
                copy_from_url,
                copy_from_revision,
                state,
                incoming_property_changes,
                outgoing_property_changes,
                session_real_path,
                move_from_absolute_path,
            }
        }
    }
}

unsafe fn char_from_pool(string: &str, pool: *mut ffi::apr_pool_t) -> *const c_char {
    let mut bytes = vec![0; string.len() + 1];
    bytes[..string.len()].copy_from_slice(string.as_bytes());

    unsafe { ffi::apr_pstrndup(pool, bytes.as_ptr() as _, bytes.len()) }
}

unsafe extern "C" fn on_progress_notify(
    progress: ffi::apr_off_t,
    total: ffi::apr_off_t,
    baton: *mut c_void,
    pool: *mut ffi::apr_pool_t,
) {
    unsafe {
        let this = &*(baton as *mut ContextInner);
        this.context_notifier
            .progress_notify(progress.try_into().unwrap(), total.try_into().unwrap());
    }
}

unsafe extern "C" fn on_cancel(baton: *mut c_void) -> *mut ffi::svn_error_t {
    unsafe {
        let ctx = &mut *(baton as *mut ContextInner);
        if let Some(msg) = ctx.context_notifier.cancel() {
            let mut pool = apr::Pool::create();
            return ffi::svn_error_create(
                ffi::svn_errno_t_SVN_ERR_CANCELLED as _,
                std::ptr::null_mut(),
                pool.string(msg.as_str()).unwrap_or(std::ptr::null_mut()) as _,
            );
        }
    }
    svn_no_error()
}

unsafe extern "C" fn on_get_commit_message(
    log_msg: *mut *const c_char,
    tmp_file: *mut *const c_char,
    commit_items: *const ffi::apr_array_header_t,
    baton: *mut c_void,
    pool: *mut ffi::apr_pool_t,
) -> *mut ffi::svn_error_t {
    if baton.is_null() {
        tracing::error!("baton is null");
        return svn_no_error();
    }

    unsafe {
        let this = &mut *(baton as *mut ContextInner);

        *log_msg = char_from_pool(&this.commit_message, pool);

        this.commit_message.clear();

        *tmp_file = std::ptr::null();

        let items = CommitItem::from_items(commit_items);

        this.commit_items.extend(items);
    }

    svn_no_error()
}

unsafe extern "C" fn on_notify(
    baton: *mut c_void,
    notify: *const ffi::svn_wc_notify_t,
    pool: *mut ffi::apr_pool_t,
) {
    unsafe {
        let this = &mut *(baton as *mut ContextInner);

        this.context_notifier
            .working_copy_notify(WorkingCopyNotify::from_raw(notify));

        // (this.on_notify)(Notify::from_raw(notify));
    }
}

// struct Pointer<T>(pub T);

impl Default for Depth {
    fn default() -> Self {
        Depth::Unknown
    }
}

unsafe extern "C" fn status_callback(
    baton: *mut c_void,
    path: *const c_char,
    status: *const ffi::svn_client_status_t,
    pool: *mut ffi::apr_pool_t,
) -> *mut ffi::svn_error_t {
    let this = unsafe { &mut *(baton as *mut ContextInner) };

    tracing::info!("Add entry");
    let entry = unsafe { StatusEntry::from_path_and_ptr(path, status) };
    tracing::info!("translated entry");

    this.status_entries.push(entry);

    svn_no_error()
}

#[uniffi::export]
fn svn_err_general_into_u32(err: SvnErrnoGeneral) -> u32 {
    err.into()
}

#[svnexus_macro::enum_converter(repr_type=ffi::svn_errno_t)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, uniffi::Enum)]
pub enum SvnErrnoGeneral {
    Warning = ffi::svn_errno_t_SVN_WARNING,
    BadContainingPool = ffi::svn_errno_t_SVN_ERR_BAD_CONTAINING_POOL,
    BadFilename = ffi::svn_errno_t_SVN_ERR_BAD_FILENAME,
    BadUrl = ffi::svn_errno_t_SVN_ERR_BAD_URL,
    BadDate = ffi::svn_errno_t_SVN_ERR_BAD_DATE,
    BadMimeType = ffi::svn_errno_t_SVN_ERR_BAD_MIME_TYPE,
    BadPropertyValue = ffi::svn_errno_t_SVN_ERR_BAD_PROPERTY_VALUE,
    BadVersionFileFormat = ffi::svn_errno_t_SVN_ERR_BAD_VERSION_FILE_FORMAT,
    BadRelativePath = ffi::svn_errno_t_SVN_ERR_BAD_RELATIVE_PATH,
    BadUuid = ffi::svn_errno_t_SVN_ERR_BAD_UUID,
    BadConfigValue = ffi::svn_errno_t_SVN_ERR_BAD_CONFIG_VALUE,
    BadServerSpecification = ffi::svn_errno_t_SVN_ERR_BAD_SERVER_SPECIFICATION,
    BadChecksumKind = ffi::svn_errno_t_SVN_ERR_BAD_CHECKSUM_KIND,
    BadChecksumParse = ffi::svn_errno_t_SVN_ERR_BAD_CHECKSUM_PARSE,
    BadToken = ffi::svn_errno_t_SVN_ERR_BAD_TOKEN,
    BadChangelistName = ffi::svn_errno_t_SVN_ERR_BAD_CHANGELIST_NAME,
    BadAtomic = ffi::svn_errno_t_SVN_ERR_BAD_ATOMIC,
    BadCompressionMethod = ffi::svn_errno_t_SVN_ERR_BAD_COMPRESSION_METHOD,
    BadPropertyValueEol = ffi::svn_errno_t_SVN_ERR_BAD_PROPERTY_VALUE_EOL,
    XmlAttribNotFound = ffi::svn_errno_t_SVN_ERR_XML_ATTRIB_NOT_FOUND,
    XmlMissingAncestry = ffi::svn_errno_t_SVN_ERR_XML_MISSING_ANCESTRY,
    XmlUnknownEncoding = ffi::svn_errno_t_SVN_ERR_XML_UNKNOWN_ENCODING,
    XmlMalformed = ffi::svn_errno_t_SVN_ERR_XML_MALFORMED,
    XmlUnescapableData = ffi::svn_errno_t_SVN_ERR_XML_UNESCAPABLE_DATA,
    XmlUnexpectedElement = ffi::svn_errno_t_SVN_ERR_XML_UNEXPECTED_ELEMENT,
    IoInconsistentEol = ffi::svn_errno_t_SVN_ERR_IO_INCONSISTENT_EOL,
    IoUnknownEol = ffi::svn_errno_t_SVN_ERR_IO_UNKNOWN_EOL,
    IoCorruptEol = ffi::svn_errno_t_SVN_ERR_IO_CORRUPT_EOL,
    IoUniqueNamesExhausted = ffi::svn_errno_t_SVN_ERR_IO_UNIQUE_NAMES_EXHAUSTED,
    IoPipeFrameError = ffi::svn_errno_t_SVN_ERR_IO_PIPE_FRAME_ERROR,
    IoPipeReadError = ffi::svn_errno_t_SVN_ERR_IO_PIPE_READ_ERROR,
    IoWriteError = ffi::svn_errno_t_SVN_ERR_IO_WRITE_ERROR,
    IoPipeWriteError = ffi::svn_errno_t_SVN_ERR_IO_PIPE_WRITE_ERROR,
    StreamUnexpectedEof = ffi::svn_errno_t_SVN_ERR_STREAM_UNEXPECTED_EOF,
    StreamMalformedData = ffi::svn_errno_t_SVN_ERR_STREAM_MALFORMED_DATA,
    StreamUnrecognizedData = ffi::svn_errno_t_SVN_ERR_STREAM_UNRECOGNIZED_DATA,
    StreamSeekNotSupported = ffi::svn_errno_t_SVN_ERR_STREAM_SEEK_NOT_SUPPORTED,
    StreamNotSupported = ffi::svn_errno_t_SVN_ERR_STREAM_NOT_SUPPORTED,
    NodeUnknownKind = ffi::svn_errno_t_SVN_ERR_NODE_UNKNOWN_KIND,
    NodeUnexpectedKind = ffi::svn_errno_t_SVN_ERR_NODE_UNEXPECTED_KIND,
    EntryNotFound = ffi::svn_errno_t_SVN_ERR_ENTRY_NOT_FOUND,
    EntryExists = ffi::svn_errno_t_SVN_ERR_ENTRY_EXISTS,
    EntryMissingRevision = ffi::svn_errno_t_SVN_ERR_ENTRY_MISSING_REVISION,
    EntryMissingUrl = ffi::svn_errno_t_SVN_ERR_ENTRY_MISSING_URL,
    EntryAttributeInvalid = ffi::svn_errno_t_SVN_ERR_ENTRY_ATTRIBUTE_INVALID,
    EntryForbidden = ffi::svn_errno_t_SVN_ERR_ENTRY_FORBIDDEN,
    WcObstructedUpdate = ffi::svn_errno_t_SVN_ERR_WC_OBSTRUCTED_UPDATE,
    WcUnwindMismatch = ffi::svn_errno_t_SVN_ERR_WC_UNWIND_MISMATCH,
    WcUnwindEmpty = ffi::svn_errno_t_SVN_ERR_WC_UNWIND_EMPTY,
    WcUnwindNotEmpty = ffi::svn_errno_t_SVN_ERR_WC_UNWIND_NOT_EMPTY,
    WcLocked = ffi::svn_errno_t_SVN_ERR_WC_LOCKED,
    WcNotLocked = ffi::svn_errno_t_SVN_ERR_WC_NOT_LOCKED,
    WcInvalidLock = ffi::svn_errno_t_SVN_ERR_WC_INVALID_LOCK,
    WcNotWorkingCopy = ffi::svn_errno_t_SVN_ERR_WC_NOT_WORKING_COPY,
    // WcNotDirectory = ffi::svn_errno_t_SVN_ERR_WC_NOT_DIRECTORY,
    WcNotFile = ffi::svn_errno_t_SVN_ERR_WC_NOT_FILE,
    WcBadAdmLog = ffi::svn_errno_t_SVN_ERR_WC_BAD_ADM_LOG,
    WcPathNotFound = ffi::svn_errno_t_SVN_ERR_WC_PATH_NOT_FOUND,
    WcNotUpToDate = ffi::svn_errno_t_SVN_ERR_WC_NOT_UP_TO_DATE,
    WcLeftLocalMod = ffi::svn_errno_t_SVN_ERR_WC_LEFT_LOCAL_MOD,
    WcScheduleConflict = ffi::svn_errno_t_SVN_ERR_WC_SCHEDULE_CONFLICT,
    WcPathFound = ffi::svn_errno_t_SVN_ERR_WC_PATH_FOUND,
    WcFoundConflict = ffi::svn_errno_t_SVN_ERR_WC_FOUND_CONFLICT,
    WcCorrupt = ffi::svn_errno_t_SVN_ERR_WC_CORRUPT,
    WcCorruptTextBase = ffi::svn_errno_t_SVN_ERR_WC_CORRUPT_TEXT_BASE,
    WcNodeKindChange = ffi::svn_errno_t_SVN_ERR_WC_NODE_KIND_CHANGE,
    WcInvalidOpOnCwd = ffi::svn_errno_t_SVN_ERR_WC_INVALID_OP_ON_CWD,
    WcBadAdmLogStart = ffi::svn_errno_t_SVN_ERR_WC_BAD_ADM_LOG_START,
    WcUnsupportedFormat = ffi::svn_errno_t_SVN_ERR_WC_UNSUPPORTED_FORMAT,
    WcBadPath = ffi::svn_errno_t_SVN_ERR_WC_BAD_PATH,
    WcInvalidSchedule = ffi::svn_errno_t_SVN_ERR_WC_INVALID_SCHEDULE,
    WcInvalidRelocation = ffi::svn_errno_t_SVN_ERR_WC_INVALID_RELOCATION,
    WcInvalidSwitch = ffi::svn_errno_t_SVN_ERR_WC_INVALID_SWITCH,
}

#[uniffi::export]
fn svn_err_fs_reposra_into_u32(err: SvnErrnoFsReposRa) -> u32 {
    err.into()
}

#[svnexus_macro::enum_converter(repr_type=ffi::svn_errno_t)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, uniffi::Enum)]
pub enum SvnErrnoFsReposRa {
    FsGeneral = ffi::svn_errno_t_SVN_ERR_FS_GENERAL,
    FsCleanup = ffi::svn_errno_t_SVN_ERR_FS_CLEANUP,
    FsAlreadyOpen = ffi::svn_errno_t_SVN_ERR_FS_ALREADY_OPEN,
    FsNotOpen = ffi::svn_errno_t_SVN_ERR_FS_NOT_OPEN,
    FsCorrupt = ffi::svn_errno_t_SVN_ERR_FS_CORRUPT,
    FsPathSyntax = ffi::svn_errno_t_SVN_ERR_FS_PATH_SYNTAX,
    FsNoSuchRevision = ffi::svn_errno_t_SVN_ERR_FS_NO_SUCH_REVISION,
    FsNoSuchTransaction = ffi::svn_errno_t_SVN_ERR_FS_NO_SUCH_TRANSACTION,
    FsNoSuchEntry = ffi::svn_errno_t_SVN_ERR_FS_NO_SUCH_ENTRY,
    FsNoSuchRepresentation = ffi::svn_errno_t_SVN_ERR_FS_NO_SUCH_REPRESENTATION,
    FsNoSuchString = ffi::svn_errno_t_SVN_ERR_FS_NO_SUCH_STRING,
    FsNoSuchCopy = ffi::svn_errno_t_SVN_ERR_FS_NO_SUCH_COPY,
    FsTransactionNotMutable = ffi::svn_errno_t_SVN_ERR_FS_TRANSACTION_NOT_MUTABLE,
    FsNotFound = ffi::svn_errno_t_SVN_ERR_FS_NOT_FOUND,
    FsIdNotFound = ffi::svn_errno_t_SVN_ERR_FS_ID_NOT_FOUND,
    FsNotId = ffi::svn_errno_t_SVN_ERR_FS_NOT_ID,
    FsNotDirectory = ffi::svn_errno_t_SVN_ERR_FS_NOT_DIRECTORY,
    FsNotFile = ffi::svn_errno_t_SVN_ERR_FS_NOT_FILE,
    FsNotSinglePathComponent = ffi::svn_errno_t_SVN_ERR_FS_NOT_SINGLE_PATH_COMPONENT,
    FsNotMutable = ffi::svn_errno_t_SVN_ERR_FS_NOT_MUTABLE,
    FsAlreadyExists = ffi::svn_errno_t_SVN_ERR_FS_ALREADY_EXISTS,
    FsRootDir = ffi::svn_errno_t_SVN_ERR_FS_ROOT_DIR,
    FsNotTxnRoot = ffi::svn_errno_t_SVN_ERR_FS_NOT_TXN_ROOT,
    FsNotRevisionRoot = ffi::svn_errno_t_SVN_ERR_FS_NOT_REVISION_ROOT,
    FsConflict = ffi::svn_errno_t_SVN_ERR_FS_CONFLICT,
    FsRepChanged = ffi::svn_errno_t_SVN_ERR_FS_REP_CHANGED,
    FsRepNotMutable = ffi::svn_errno_t_SVN_ERR_FS_REP_NOT_MUTABLE,
    FsMalformedSkel = ffi::svn_errno_t_SVN_ERR_FS_MALFORMED_SKEL,
    FsTxnOutOfDate = ffi::svn_errno_t_SVN_ERR_FS_TXN_OUT_OF_DATE,
    FsBerkeleyDb = ffi::svn_errno_t_SVN_ERR_FS_BERKELEY_DB,
    FsBerkeleyDbDeadlock = ffi::svn_errno_t_SVN_ERR_FS_BERKELEY_DB_DEADLOCK,
    FsTransactionDead = ffi::svn_errno_t_SVN_ERR_FS_TRANSACTION_DEAD,
    FsTransactionNotDead = ffi::svn_errno_t_SVN_ERR_FS_TRANSACTION_NOT_DEAD,
    FsUnknownFsType = ffi::svn_errno_t_SVN_ERR_FS_UNKNOWN_FS_TYPE,
    FsNoUser = ffi::svn_errno_t_SVN_ERR_FS_NO_USER,
    FsPathAlreadyLocked = ffi::svn_errno_t_SVN_ERR_FS_PATH_ALREADY_LOCKED,
    FsPathNotLocked = ffi::svn_errno_t_SVN_ERR_FS_PATH_NOT_LOCKED,
    FsBadLockToken = ffi::svn_errno_t_SVN_ERR_FS_BAD_LOCK_TOKEN,
    FsNoLockToken = ffi::svn_errno_t_SVN_ERR_FS_NO_LOCK_TOKEN,
    FsLockOwnerMismatch = ffi::svn_errno_t_SVN_ERR_FS_LOCK_OWNER_MISMATCH,
    FsNoSuchLock = ffi::svn_errno_t_SVN_ERR_FS_NO_SUCH_LOCK,
    FsLockExpired = ffi::svn_errno_t_SVN_ERR_FS_LOCK_EXPIRED,
    FsOutOfDate = ffi::svn_errno_t_SVN_ERR_FS_OUT_OF_DATE,
    FsUnsupportedFormat = ffi::svn_errno_t_SVN_ERR_FS_UNSUPPORTED_FORMAT,
    FsRepBeingWritten = ffi::svn_errno_t_SVN_ERR_FS_REP_BEING_WRITTEN,
    FsTxnNameTooLong = ffi::svn_errno_t_SVN_ERR_FS_TXN_NAME_TOO_LONG,
    FsNoSuchNodeOrigin = ffi::svn_errno_t_SVN_ERR_FS_NO_SUCH_NODE_ORIGIN,
    FsUnsupportedUpgrade = ffi::svn_errno_t_SVN_ERR_FS_UNSUPPORTED_UPGRADE,
    FsNoSuchChecksumRep = ffi::svn_errno_t_SVN_ERR_FS_NO_SUCH_CHECKSUM_REP,
    FsPropBasevalueMismatch = ffi::svn_errno_t_SVN_ERR_FS_PROP_BASEVALUE_MISMATCH,
    FsIncorrectEditorCompletion = ffi::svn_errno_t_SVN_ERR_FS_INCORRECT_EDITOR_COMPLETION,
    FsPackedRevpropReadFailure = ffi::svn_errno_t_SVN_ERR_FS_PACKED_REVPROP_READ_FAILURE,
    FsRevpropCacheInitFailure = ffi::svn_errno_t_SVN_ERR_FS_REVPROP_CACHE_INIT_FAILURE,
    FsMalformedTxnId = ffi::svn_errno_t_SVN_ERR_FS_MALFORMED_TXN_ID,
}

#[uniffi::export]
fn svn_err_reposra_into_u32(err: SvnErrnoReposRa) -> u32 {
    err.into()
}

#[svnexus_macro::enum_converter(repr_type=ffi::svn_errno_t)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, uniffi::Enum)]
pub enum SvnErrnoReposRa {
    ReposLocked = ffi::svn_errno_t_SVN_ERR_REPOS_LOCKED,
    ReposHookFailure = ffi::svn_errno_t_SVN_ERR_REPOS_HOOK_FAILURE,
    ReposBadArgs = ffi::svn_errno_t_SVN_ERR_REPOS_BAD_ARGS,
    ReposNoDataForReport = ffi::svn_errno_t_SVN_ERR_REPOS_NO_DATA_FOR_REPORT,
    ReposBadRevisionReport = ffi::svn_errno_t_SVN_ERR_REPOS_BAD_REVISION_REPORT,
    ReposUnsupportedVersion = ffi::svn_errno_t_SVN_ERR_REPOS_UNSUPPORTED_VERSION,
    ReposDisabledFeature = ffi::svn_errno_t_SVN_ERR_REPOS_DISABLED_FEATURE,
    ReposPostCommitHookFailed = ffi::svn_errno_t_SVN_ERR_REPOS_POST_COMMIT_HOOK_FAILED,
    ReposPostLockHookFailed = ffi::svn_errno_t_SVN_ERR_REPOS_POST_LOCK_HOOK_FAILED,
    ReposPostUnlockHookFailed = ffi::svn_errno_t_SVN_ERR_REPOS_POST_UNLOCK_HOOK_FAILED,
    ReposUnsupportedUpgrade = ffi::svn_errno_t_SVN_ERR_REPOS_UNSUPPORTED_UPGRADE,
    RaIllegalUrl = ffi::svn_errno_t_SVN_ERR_RA_ILLEGAL_URL,
    RaNotAuthorized = ffi::svn_errno_t_SVN_ERR_RA_NOT_AUTHORIZED,
    RaUnknownAuth = ffi::svn_errno_t_SVN_ERR_RA_UNKNOWN_AUTH,
    RaNotImplemented = ffi::svn_errno_t_SVN_ERR_RA_NOT_IMPLEMENTED,
    RaOutOfDate = ffi::svn_errno_t_SVN_ERR_RA_OUT_OF_DATE,
    RaNoReposUuid = ffi::svn_errno_t_SVN_ERR_RA_NO_REPOS_UUID,
    RaUnsupportedAbiVersion = ffi::svn_errno_t_SVN_ERR_RA_UNSUPPORTED_ABI_VERSION,
    RaNotLocked = ffi::svn_errno_t_SVN_ERR_RA_NOT_LOCKED,
    RaPartialReplayNotSupported = ffi::svn_errno_t_SVN_ERR_RA_PARTIAL_REPLAY_NOT_SUPPORTED,
    RaUuidMismatch = ffi::svn_errno_t_SVN_ERR_RA_UUID_MISMATCH,
    RaReposRootUrlMismatch = ffi::svn_errno_t_SVN_ERR_RA_REPOS_ROOT_URL_MISMATCH,
    RaSessionUrlMismatch = ffi::svn_errno_t_SVN_ERR_RA_SESSION_URL_MISMATCH,
    RaCannotCreateTunnel = ffi::svn_errno_t_SVN_ERR_RA_CANNOT_CREATE_TUNNEL,
    RaCannotCreateSession = ffi::svn_errno_t_SVN_ERR_RA_CANNOT_CREATE_SESSION,
    RaDavSockInit = ffi::svn_errno_t_SVN_ERR_RA_DAV_SOCK_INIT,
    RaDavCreatingRequest = ffi::svn_errno_t_SVN_ERR_RA_DAV_CREATING_REQUEST,
    RaDavRequestFailed = ffi::svn_errno_t_SVN_ERR_RA_DAV_REQUEST_FAILED,
    RaDavOptionsReqFailed = ffi::svn_errno_t_SVN_ERR_RA_DAV_OPTIONS_REQ_FAILED,
    RaDavPropsNotFound = ffi::svn_errno_t_SVN_ERR_RA_DAV_PROPS_NOT_FOUND,
}

#[derive(Debug)]
pub struct SvnErrnoConstants {
    pub warning: i32,
    pub bad_containing_pool: i32,
    pub bad_filename: i32,
    pub bad_url: i32,
    pub bad_date: i32,
    pub bad_mime_type: i32,
    pub bad_property_value: i32,
    pub bad_version_file_format: i32,
    pub bad_relative_path: i32,
    pub bad_uuid: i32,
    pub bad_config_value: i32,
    pub bad_server_specification: i32,
    pub bad_checksum_kind: i32,
    pub bad_checksum_parse: i32,
    pub bad_token: i32,
    pub bad_changelist_name: i32,
    pub bad_atomic: i32,
    pub bad_compression_method: i32,
    pub bad_property_value_eol: i32,
    pub xml_attrib_not_found: i32,
    pub xml_missing_ancestry: i32,
    pub xml_unknown_encoding: i32,
    pub xml_malformed: i32,
    pub xml_unescapable_data: i32,
    pub xml_unexpected_element: i32,
    pub io_inconsistent_eol: i32,
    pub io_unknown_eol: i32,
    pub io_corrupt_eol: i32,
    pub io_unique_names_exhausted: i32,
    pub io_pipe_frame_error: i32,
    pub io_pipe_read_error: i32,
    pub io_write_error: i32,
    pub io_pipe_write_error: i32,
    pub stream_unexpected_eof: i32,
    pub stream_malformed_data: i32,
    pub stream_unrecognized_data: i32,
    pub stream_seek_not_supported: i32,
    pub stream_not_supported: i32,
    pub node_unknown_kind: i32,
    pub node_unexpected_kind: i32,
    pub entry_not_found: i32,
    pub entry_exists: i32,
    pub entry_missing_revision: i32,
    pub entry_missing_url: i32,
    pub entry_attribute_invalid: i32,
    pub entry_forbidden: i32,
    pub wc_obstructed_update: i32,
    pub wc_unwind_mismatch: i32,
    pub wc_unwind_empty: i32,
    pub wc_unwind_not_empty: i32,
    pub wc_locked: i32,
    pub wc_not_locked: i32,
    pub wc_invalid_lock: i32,
    pub wc_not_working_copy: i32,
    pub wc_not_directory: i32,
    pub wc_not_file: i32,
    pub wc_bad_adm_log: i32,
    pub wc_path_not_found: i32,
    pub wc_not_up_to_date: i32,
    pub wc_left_local_mod: i32,
    pub wc_schedule_conflict: i32,
    pub wc_path_found: i32,
    pub wc_found_conflict: i32,
    pub wc_corrupt: i32,
    pub wc_corrupt_text_base: i32,
    pub wc_node_kind_change: i32,
    pub wc_invalid_op_on_cwd: i32,
    pub wc_bad_adm_log_start: i32,
    pub wc_unsupported_format: i32,
    pub wc_bad_path: i32,
    pub wc_invalid_schedule: i32,
    pub wc_invalid_relocation: i32,
    pub wc_invalid_switch: i32,
    pub wc_mismatched_changelist: i32,
    pub wc_conflict_resolver_failure: i32,
    pub wc_copyfrom_path_not_found: i32,
    pub wc_changelist_move: i32,
    pub wc_cannot_delete_file_external: i32,
    pub wc_cannot_move_file_external: i32,
    pub wc_db_error: i32,
    pub wc_missing: i32,
    pub wc_not_symlink: i32,
    pub wc_path_unexpected_status: i32,
    pub wc_upgrade_required: i32,
    pub wc_cleanup_required: i32,
    pub wc_invalid_operation_depth: i32,
    pub wc_path_access_denied: i32,
    pub wc_mixed_revisions: i32,
    pub wc_duplicate_externals_target: i32,
    pub wc_incompatible_settings: i32,
    pub wc_deprecated_api_store_pristine: i32,
    pub wc_pristine_dehydrated: i32,
    pub fs_general: i32,
    pub fs_cleanup: i32,
    pub fs_already_open: i32,
    pub fs_not_open: i32,
    pub fs_corrupt: i32,
    pub fs_path_syntax: i32,
    pub fs_no_such_revision: i32,
    pub fs_no_such_transaction: i32,
    pub fs_no_such_entry: i32,
    pub fs_no_such_representation: i32,
    pub fs_no_such_string: i32,
    pub fs_no_such_copy: i32,
    pub fs_transaction_not_mutable: i32,
    pub fs_not_found: i32,
    pub fs_id_not_found: i32,
    pub fs_not_id: i32,
    pub fs_not_directory: i32,
    pub fs_not_file: i32,
    pub fs_not_single_path_component: i32,
    pub fs_not_mutable: i32,
    pub fs_already_exists: i32,
    pub fs_root_dir: i32,
    pub fs_not_txn_root: i32,
    pub fs_not_revision_root: i32,
    pub fs_conflict: i32,
    pub fs_rep_changed: i32,
    pub fs_rep_not_mutable: i32,
    pub fs_malformed_skel: i32,
    pub fs_txn_out_of_date: i32,
    pub fs_berkeley_db: i32,
    pub fs_berkeley_db_deadlock: i32,
    pub fs_transaction_dead: i32,
    pub fs_transaction_not_dead: i32,
    pub fs_unknown_fs_type: i32,
    pub fs_no_user: i32,
    pub fs_path_already_locked: i32,
    pub fs_path_not_locked: i32,
    pub fs_bad_lock_token: i32,
    pub fs_no_lock_token: i32,
    pub fs_lock_owner_mismatch: i32,
    pub fs_no_such_lock: i32,
    pub fs_lock_expired: i32,
    pub fs_out_of_date: i32,
    pub fs_unsupported_format: i32,
    pub fs_rep_being_written: i32,
    pub fs_txn_name_too_long: i32,
    pub fs_no_such_node_origin: i32,
    pub fs_unsupported_upgrade: i32,
    pub fs_no_such_checksum_rep: i32,
    pub fs_prop_basevalue_mismatch: i32,
    pub fs_incorrect_editor_completion: i32,
    pub fs_packed_revprop_read_failure: i32,
    pub fs_revprop_cache_init_failure: i32,
    pub fs_malformed_txn_id: i32,
    pub fs_index_corruption: i32,
    pub fs_index_revision: i32,
    pub fs_index_overflow: i32,
    pub fs_container_index: i32,
    pub fs_index_inconsistent: i32,
    pub fs_lock_operation_failed: i32,
    pub fs_unsupported_type: i32,
    pub fs_container_size: i32,
    pub fs_malformed_noderev_id: i32,
    pub fs_invalid_generation: i32,
    pub fs_corrupt_revprop_manifest: i32,
    pub fs_corrupt_proplist: i32,
    pub fs_ambiguous_checksum_rep: i32,
    pub fs_unrecognized_ioctl_code: i32,
    pub fs_rep_sharing_not_allowed: i32,
    pub fs_rep_sharing_not_supported: i32,
    pub repos_locked: i32,
    pub repos_hook_failure: i32,
    pub repos_bad_args: i32,
    pub repos_no_data_for_report: i32,
    pub repos_bad_revision_report: i32,
    pub repos_unsupported_version: i32,
    pub repos_disabled_feature: i32,
    pub repos_post_commit_hook_failed: i32,
    pub repos_post_lock_hook_failed: i32,
    pub repos_post_unlock_hook_failed: i32,
    pub repos_unsupported_upgrade: i32,
    pub ra_illegal_url: i32,
    pub ra_not_authorized: i32,
    pub ra_unknown_auth: i32,
    pub ra_not_implemented: i32,
    pub ra_out_of_date: i32,
    pub ra_no_repos_uuid: i32,
    pub ra_unsupported_abi_version: i32,
    pub ra_not_locked: i32,
    pub ra_partial_replay_not_supported: i32,
    pub ra_uuid_mismatch: i32,
    pub ra_repos_root_url_mismatch: i32,
    pub ra_session_url_mismatch: i32,
    pub ra_cannot_create_tunnel: i32,
    pub ra_cannot_create_session: i32,
    pub ra_dav_sock_init: i32,
    pub ra_dav_creating_request: i32,
    pub ra_dav_request_failed: i32,
    pub ra_dav_options_req_failed: i32,
    pub ra_dav_props_not_found: i32,
    pub ra_dav_already_exists: i32,
    pub ra_dav_invalid_config_value: i32,
    pub ra_dav_path_not_found: i32,
    pub ra_dav_proppatch_failed: i32,
    pub ra_dav_malformed_data: i32,
    pub ra_dav_response_header_badness: i32,
    pub ra_dav_relocated: i32,
    pub ra_dav_conn_timeout: i32,
    pub ra_dav_forbidden: i32,
    pub ra_dav_precondition_failed: i32,
    pub ra_dav_method_not_allowed: i32,
    pub ra_local_repos_not_found: i32,
    pub ra_local_repos_open_failed: i32,
    pub svndiff_invalid_header: i32,
    pub svndiff_corrupt_window: i32,
    pub svndiff_backward_view: i32,
    pub svndiff_invalid_ops: i32,
    pub svndiff_unexpected_end: i32,
    pub svndiff_invalid_compressed_data: i32,
    pub apmod_missing_path_to_fs: i32,
    pub apmod_malformed_uri: i32,
    pub apmod_activity_not_found: i32,
    pub apmod_bad_baseline: i32,
    pub apmod_connection_aborted: i32,
    pub client_versioned_path_required: i32,
    pub client_ra_access_required: i32,
    pub client_bad_revision: i32,
    pub client_duplicate_commit_url: i32,
    pub client_is_binary_file: i32,
    pub client_invalid_externals_description: i32,
    pub client_modified: i32,
    pub client_is_directory: i32,
    pub client_revision_range: i32,
    pub client_invalid_relocation: i32,
    pub client_revision_author_contains_newline: i32,
    pub client_property_name: i32,
    pub client_unrelated_resources: i32,
    pub client_missing_lock_token: i32,
    pub client_multiple_sources_disallowed: i32,
    pub client_no_versioned_parent: i32,
    pub client_not_ready_to_merge: i32,
    pub client_file_external_overwrite_versioned: i32,
    pub client_patch_bad_strip_count: i32,
    pub client_cycle_detected: i32,
    pub client_merge_update_required: i32,
    pub client_invalid_mergeinfo_no_mergetracking: i32,
    pub client_no_lock_token: i32,
    pub client_forbidden_by_server: i32,
    pub client_conflict_option_not_applicable: i32,
    pub base: i32,
    pub plugin_load_failure: i32,
    pub malformed_file: i32,
    pub incomplete_data: i32,
    pub incorrect_params: i32,
    pub unversioned_resource: i32,
    pub test_failed: i32,
    pub unsupported_feature: i32,
    pub bad_prop_kind: i32,
    pub illegal_target: i32,
    pub delta_md5_checksum_absent: i32,
    pub dir_not_empty: i32,
    pub external_program: i32,
    pub swig_py_exception_set: i32,
    pub checksum_mismatch: i32,
    pub cancelled: i32,
    pub invalid_diff_option: i32,
    pub property_not_found: i32,
    pub no_auth_file_path: i32,
    pub version_mismatch: i32,
    pub mergeinfo_parse_error: i32,
    pub cease_invocation: i32,
    pub revnum_parse_failure: i32,
    pub iter_break: i32,
    pub unknown_changelist: i32,
    pub reserved_filename_specified: i32,
    pub unknown_capability: i32,
    pub test_skipped: i32,
    pub no_apr_memcache: i32,
    pub atomic_init_failure: i32,
    pub sqlite_error: i32,
    pub sqlite_readonly: i32,
    pub sqlite_unsupported_schema: i32,
    pub sqlite_busy: i32,
    pub sqlite_resetting_for_rollback: i32,
    pub sqlite_constraint: i32,
    pub too_many_memcached_servers: i32,
    pub malformed_version_string: i32,
    pub corrupted_atomic_storage: i32,
    pub utf8proc_error: i32,
    pub utf8_glob: i32,
    pub corrupt_packed_data: i32,
    pub composed_error: i32,
    pub invalid_input: i32,
    pub sqlite_rollback_failed: i32,
    pub lz4_compression_failed: i32,
    pub lz4_decompression_failed: i32,
    pub canonicalization_failed: i32,
    pub cl_arg_parsing_error: i32,
    pub cl_insufficient_args: i32,
    pub cl_mutually_exclusive_args: i32,
    pub cl_adm_dir_reserved: i32,
    pub cl_log_message_is_versioned_file: i32,
    pub cl_log_message_is_pathname: i32,
    pub cl_commit_in_added_dir: i32,
    pub cl_no_external_editor: i32,
    pub cl_bad_log_message: i32,
    pub cl_unnecessary_log_message: i32,
    pub cl_no_external_merge_tool: i32,
    pub cl_error_processing_externals: i32,
    pub cl_repos_verify_failed: i32,
    pub ra_svn_cmd_err: i32,
    pub ra_svn_unknown_cmd: i32,
    pub ra_svn_connection_closed: i32,
    pub ra_svn_io_error: i32,
    pub ra_svn_malformed_data: i32,
    pub ra_svn_repos_not_found: i32,
    pub ra_svn_bad_version: i32,
    pub ra_svn_no_mechanisms: i32,
    pub ra_svn_edit_aborted: i32,
    pub ra_svn_request_size: i32,
    pub ra_svn_response_size: i32,
    pub authn_creds_unavailable: i32,
    pub authn_no_provider: i32,
    pub authn_providers_exhausted: i32,
    pub authn_creds_not_saved: i32,
    pub authn_failed: i32,
    pub authz_root_unreadable: i32,
    pub authz_unreadable: i32,
    pub authz_partially_readable: i32,
    pub authz_invalid_config: i32,
    pub authz_unwritable: i32,
    pub diff_datasource_modified: i32,
    pub diff_unexpected_data: i32,
    pub ra_serf_sspi_initialisation_failed: i32,
    pub ra_serf_ssl_cert_untrusted: i32,
    pub ra_serf_gssapi_initialisation_failed: i32,
    pub ra_serf_wrapped_error: i32,
    pub ra_serf_stream_bucket_read_error: i32,
    pub assertion_fail: i32,
    pub assertion_only_tracing_links: i32,
    pub asn1_out_of_data: i32,
    pub asn1_unexpected_tag: i32,
    pub asn1_invalid_length: i32,
    pub asn1_length_mismatch: i32,
    pub asn1_invalid_data: i32,
    pub x509_feature_unavailable: i32,
    pub x509_cert_invalid_pem: i32,
    pub x509_cert_invalid_format: i32,
    pub x509_cert_invalid_version: i32,
    pub x509_cert_invalid_serial: i32,
    pub x509_cert_invalid_alg: i32,
    pub x509_cert_invalid_name: i32,
    pub x509_cert_invalid_date: i32,
    pub x509_cert_invalid_pubkey: i32,
    pub x509_cert_invalid_signature: i32,
    pub x509_cert_invalid_extensions: i32,
    pub x509_cert_unknown_version: i32,
    pub x509_cert_unknown_pk_alg: i32,
    pub x509_cert_sig_mismatch: i32,
    pub x509_cert_verify_failed: i32,
}

impl SvnErrnoConstants {
    /// Creates a new instance of SvnErrnoConstants,
    /// initialized with values from the ffi module.

    pub fn new() -> Self {
        SvnErrnoConstants {
            warning: ffi::svn_errno_t_SVN_WARNING as _,
            bad_containing_pool: ffi::svn_errno_t_SVN_ERR_BAD_CONTAINING_POOL as _,
            bad_filename: ffi::svn_errno_t_SVN_ERR_BAD_FILENAME as _,
            bad_url: ffi::svn_errno_t_SVN_ERR_BAD_URL as _,
            bad_date: ffi::svn_errno_t_SVN_ERR_BAD_DATE as _,
            bad_mime_type: ffi::svn_errno_t_SVN_ERR_BAD_MIME_TYPE as _,
            bad_property_value: ffi::svn_errno_t_SVN_ERR_BAD_PROPERTY_VALUE as _,
            bad_version_file_format: ffi::svn_errno_t_SVN_ERR_BAD_VERSION_FILE_FORMAT as _,
            bad_relative_path: ffi::svn_errno_t_SVN_ERR_BAD_RELATIVE_PATH as _,
            bad_uuid: ffi::svn_errno_t_SVN_ERR_BAD_UUID as _,
            bad_config_value: ffi::svn_errno_t_SVN_ERR_BAD_CONFIG_VALUE as _,
            bad_server_specification: ffi::svn_errno_t_SVN_ERR_BAD_SERVER_SPECIFICATION as _,
            bad_checksum_kind: ffi::svn_errno_t_SVN_ERR_BAD_CHECKSUM_KIND as _,
            bad_checksum_parse: ffi::svn_errno_t_SVN_ERR_BAD_CHECKSUM_PARSE as _,
            bad_token: ffi::svn_errno_t_SVN_ERR_BAD_TOKEN as _,
            bad_changelist_name: ffi::svn_errno_t_SVN_ERR_BAD_CHANGELIST_NAME as _,
            bad_atomic: ffi::svn_errno_t_SVN_ERR_BAD_ATOMIC as _,
            bad_compression_method: ffi::svn_errno_t_SVN_ERR_BAD_COMPRESSION_METHOD as _,
            bad_property_value_eol: ffi::svn_errno_t_SVN_ERR_BAD_PROPERTY_VALUE_EOL as _,
            xml_attrib_not_found: ffi::svn_errno_t_SVN_ERR_XML_ATTRIB_NOT_FOUND as _,
            xml_missing_ancestry: ffi::svn_errno_t_SVN_ERR_XML_MISSING_ANCESTRY as _,
            xml_unknown_encoding: ffi::svn_errno_t_SVN_ERR_XML_UNKNOWN_ENCODING as _,
            xml_malformed: ffi::svn_errno_t_SVN_ERR_XML_MALFORMED as _,
            xml_unescapable_data: ffi::svn_errno_t_SVN_ERR_XML_UNESCAPABLE_DATA as _,
            xml_unexpected_element: ffi::svn_errno_t_SVN_ERR_XML_UNEXPECTED_ELEMENT as _,
            io_inconsistent_eol: ffi::svn_errno_t_SVN_ERR_IO_INCONSISTENT_EOL as _,
            io_unknown_eol: ffi::svn_errno_t_SVN_ERR_IO_UNKNOWN_EOL as _,
            io_corrupt_eol: ffi::svn_errno_t_SVN_ERR_IO_CORRUPT_EOL as _,
            io_unique_names_exhausted: ffi::svn_errno_t_SVN_ERR_IO_UNIQUE_NAMES_EXHAUSTED as _,
            io_pipe_frame_error: ffi::svn_errno_t_SVN_ERR_IO_PIPE_FRAME_ERROR as _,
            io_pipe_read_error: ffi::svn_errno_t_SVN_ERR_IO_PIPE_READ_ERROR as _,
            io_write_error: ffi::svn_errno_t_SVN_ERR_IO_WRITE_ERROR as _,
            io_pipe_write_error: ffi::svn_errno_t_SVN_ERR_IO_PIPE_WRITE_ERROR as _,
            stream_unexpected_eof: ffi::svn_errno_t_SVN_ERR_STREAM_UNEXPECTED_EOF as _,
            stream_malformed_data: ffi::svn_errno_t_SVN_ERR_STREAM_MALFORMED_DATA as _,
            stream_unrecognized_data: ffi::svn_errno_t_SVN_ERR_STREAM_UNRECOGNIZED_DATA as _,
            stream_seek_not_supported: ffi::svn_errno_t_SVN_ERR_STREAM_SEEK_NOT_SUPPORTED as _,
            stream_not_supported: ffi::svn_errno_t_SVN_ERR_STREAM_NOT_SUPPORTED as _,
            node_unknown_kind: ffi::svn_errno_t_SVN_ERR_NODE_UNKNOWN_KIND as _,
            node_unexpected_kind: ffi::svn_errno_t_SVN_ERR_NODE_UNEXPECTED_KIND as _,
            entry_not_found: ffi::svn_errno_t_SVN_ERR_ENTRY_NOT_FOUND as _,
            entry_exists: ffi::svn_errno_t_SVN_ERR_ENTRY_EXISTS as _,
            entry_missing_revision: ffi::svn_errno_t_SVN_ERR_ENTRY_MISSING_REVISION as _,
            entry_missing_url: ffi::svn_errno_t_SVN_ERR_ENTRY_MISSING_URL as _,
            entry_attribute_invalid: ffi::svn_errno_t_SVN_ERR_ENTRY_ATTRIBUTE_INVALID as _,
            entry_forbidden: ffi::svn_errno_t_SVN_ERR_ENTRY_FORBIDDEN as _,
            wc_obstructed_update: ffi::svn_errno_t_SVN_ERR_WC_OBSTRUCTED_UPDATE as _,
            wc_unwind_mismatch: ffi::svn_errno_t_SVN_ERR_WC_UNWIND_MISMATCH as _,
            wc_unwind_empty: ffi::svn_errno_t_SVN_ERR_WC_UNWIND_EMPTY as _,
            wc_unwind_not_empty: ffi::svn_errno_t_SVN_ERR_WC_UNWIND_NOT_EMPTY as _,
            wc_locked: ffi::svn_errno_t_SVN_ERR_WC_LOCKED as _,
            wc_not_locked: ffi::svn_errno_t_SVN_ERR_WC_NOT_LOCKED as _,
            wc_invalid_lock: ffi::svn_errno_t_SVN_ERR_WC_INVALID_LOCK as _,
            wc_not_working_copy: ffi::svn_errno_t_SVN_ERR_WC_NOT_WORKING_COPY as _,
            wc_not_directory: ffi::svn_errno_t_SVN_ERR_WC_NOT_DIRECTORY as _,
            wc_not_file: ffi::svn_errno_t_SVN_ERR_WC_NOT_FILE as _,
            wc_bad_adm_log: ffi::svn_errno_t_SVN_ERR_WC_BAD_ADM_LOG as _,
            wc_path_not_found: ffi::svn_errno_t_SVN_ERR_WC_PATH_NOT_FOUND as _,
            wc_not_up_to_date: ffi::svn_errno_t_SVN_ERR_WC_NOT_UP_TO_DATE as _,
            wc_left_local_mod: ffi::svn_errno_t_SVN_ERR_WC_LEFT_LOCAL_MOD as _,
            wc_schedule_conflict: ffi::svn_errno_t_SVN_ERR_WC_SCHEDULE_CONFLICT as _,
            wc_path_found: ffi::svn_errno_t_SVN_ERR_WC_PATH_FOUND as _,
            wc_found_conflict: ffi::svn_errno_t_SVN_ERR_WC_FOUND_CONFLICT as _,
            wc_corrupt: ffi::svn_errno_t_SVN_ERR_WC_CORRUPT as _,
            wc_corrupt_text_base: ffi::svn_errno_t_SVN_ERR_WC_CORRUPT_TEXT_BASE as _,
            wc_node_kind_change: ffi::svn_errno_t_SVN_ERR_WC_NODE_KIND_CHANGE as _,
            wc_invalid_op_on_cwd: ffi::svn_errno_t_SVN_ERR_WC_INVALID_OP_ON_CWD as _,
            wc_bad_adm_log_start: ffi::svn_errno_t_SVN_ERR_WC_BAD_ADM_LOG_START as _,
            wc_unsupported_format: ffi::svn_errno_t_SVN_ERR_WC_UNSUPPORTED_FORMAT as _,
            wc_bad_path: ffi::svn_errno_t_SVN_ERR_WC_BAD_PATH as _,
            wc_invalid_schedule: ffi::svn_errno_t_SVN_ERR_WC_INVALID_SCHEDULE as _,
            wc_invalid_relocation: ffi::svn_errno_t_SVN_ERR_WC_INVALID_RELOCATION as _,
            wc_invalid_switch: ffi::svn_errno_t_SVN_ERR_WC_INVALID_SWITCH as _,
            wc_mismatched_changelist: ffi::svn_errno_t_SVN_ERR_WC_MISMATCHED_CHANGELIST as _,
            wc_conflict_resolver_failure: ffi::svn_errno_t_SVN_ERR_WC_CONFLICT_RESOLVER_FAILURE
                as _,
            wc_copyfrom_path_not_found: ffi::svn_errno_t_SVN_ERR_WC_COPYFROM_PATH_NOT_FOUND as _,
            wc_changelist_move: ffi::svn_errno_t_SVN_ERR_WC_CHANGELIST_MOVE as _,
            wc_cannot_delete_file_external: ffi::svn_errno_t_SVN_ERR_WC_CANNOT_DELETE_FILE_EXTERNAL
                as _,
            wc_cannot_move_file_external: ffi::svn_errno_t_SVN_ERR_WC_CANNOT_MOVE_FILE_EXTERNAL
                as _,
            wc_db_error: ffi::svn_errno_t_SVN_ERR_WC_DB_ERROR as _,
            wc_missing: ffi::svn_errno_t_SVN_ERR_WC_MISSING as _,
            wc_not_symlink: ffi::svn_errno_t_SVN_ERR_WC_NOT_SYMLINK as _,
            wc_path_unexpected_status: ffi::svn_errno_t_SVN_ERR_WC_PATH_UNEXPECTED_STATUS as _,
            wc_upgrade_required: ffi::svn_errno_t_SVN_ERR_WC_UPGRADE_REQUIRED as _,
            wc_cleanup_required: ffi::svn_errno_t_SVN_ERR_WC_CLEANUP_REQUIRED as _,
            wc_invalid_operation_depth: ffi::svn_errno_t_SVN_ERR_WC_INVALID_OPERATION_DEPTH as _,
            wc_path_access_denied: ffi::svn_errno_t_SVN_ERR_WC_PATH_ACCESS_DENIED as _,
            wc_mixed_revisions: ffi::svn_errno_t_SVN_ERR_WC_MIXED_REVISIONS as _,
            wc_duplicate_externals_target: ffi::svn_errno_t_SVN_ERR_WC_DUPLICATE_EXTERNALS_TARGET
                as _,
            wc_incompatible_settings: ffi::svn_errno_t_SVN_ERR_WC_INCOMPATIBLE_SETTINGS as _,
            wc_deprecated_api_store_pristine:
                ffi::svn_errno_t_SVN_ERR_WC_DEPRECATED_API_STORE_PRISTINE as _,
            wc_pristine_dehydrated: ffi::svn_errno_t_SVN_ERR_WC_PRISTINE_DEHYDRATED as _,
            fs_general: ffi::svn_errno_t_SVN_ERR_FS_GENERAL as _,
            fs_cleanup: ffi::svn_errno_t_SVN_ERR_FS_CLEANUP as _,
            fs_already_open: ffi::svn_errno_t_SVN_ERR_FS_ALREADY_OPEN as _,
            fs_not_open: ffi::svn_errno_t_SVN_ERR_FS_NOT_OPEN as _,
            fs_corrupt: ffi::svn_errno_t_SVN_ERR_FS_CORRUPT as _,
            fs_path_syntax: ffi::svn_errno_t_SVN_ERR_FS_PATH_SYNTAX as _,
            fs_no_such_revision: ffi::svn_errno_t_SVN_ERR_FS_NO_SUCH_REVISION as _,
            fs_no_such_transaction: ffi::svn_errno_t_SVN_ERR_FS_NO_SUCH_TRANSACTION as _,
            fs_no_such_entry: ffi::svn_errno_t_SVN_ERR_FS_NO_SUCH_ENTRY as _,
            fs_no_such_representation: ffi::svn_errno_t_SVN_ERR_FS_NO_SUCH_REPRESENTATION as _,
            fs_no_such_string: ffi::svn_errno_t_SVN_ERR_FS_NO_SUCH_STRING as _,
            fs_no_such_copy: ffi::svn_errno_t_SVN_ERR_FS_NO_SUCH_COPY as _,
            fs_transaction_not_mutable: ffi::svn_errno_t_SVN_ERR_FS_TRANSACTION_NOT_MUTABLE as _,
            fs_not_found: ffi::svn_errno_t_SVN_ERR_FS_NOT_FOUND as _,
            fs_id_not_found: ffi::svn_errno_t_SVN_ERR_FS_ID_NOT_FOUND as _,
            fs_not_id: ffi::svn_errno_t_SVN_ERR_FS_NOT_ID as _,
            fs_not_directory: ffi::svn_errno_t_SVN_ERR_FS_NOT_DIRECTORY as _,
            fs_not_file: ffi::svn_errno_t_SVN_ERR_FS_NOT_FILE as _,
            fs_not_single_path_component: ffi::svn_errno_t_SVN_ERR_FS_NOT_SINGLE_PATH_COMPONENT
                as _,
            fs_not_mutable: ffi::svn_errno_t_SVN_ERR_FS_NOT_MUTABLE as _,
            fs_already_exists: ffi::svn_errno_t_SVN_ERR_FS_ALREADY_EXISTS as _,
            fs_root_dir: ffi::svn_errno_t_SVN_ERR_FS_ROOT_DIR as _,
            fs_not_txn_root: ffi::svn_errno_t_SVN_ERR_FS_NOT_TXN_ROOT as _,
            fs_not_revision_root: ffi::svn_errno_t_SVN_ERR_FS_NOT_REVISION_ROOT as _,
            fs_conflict: ffi::svn_errno_t_SVN_ERR_FS_CONFLICT as _,
            fs_rep_changed: ffi::svn_errno_t_SVN_ERR_FS_REP_CHANGED as _,
            fs_rep_not_mutable: ffi::svn_errno_t_SVN_ERR_FS_REP_NOT_MUTABLE as _,
            fs_malformed_skel: ffi::svn_errno_t_SVN_ERR_FS_MALFORMED_SKEL as _,
            fs_txn_out_of_date: ffi::svn_errno_t_SVN_ERR_FS_TXN_OUT_OF_DATE as _,
            fs_berkeley_db: ffi::svn_errno_t_SVN_ERR_FS_BERKELEY_DB as _,
            fs_berkeley_db_deadlock: ffi::svn_errno_t_SVN_ERR_FS_BERKELEY_DB_DEADLOCK as _,
            fs_transaction_dead: ffi::svn_errno_t_SVN_ERR_FS_TRANSACTION_DEAD as _,
            fs_transaction_not_dead: ffi::svn_errno_t_SVN_ERR_FS_TRANSACTION_NOT_DEAD as _,
            fs_unknown_fs_type: ffi::svn_errno_t_SVN_ERR_FS_UNKNOWN_FS_TYPE as _,
            fs_no_user: ffi::svn_errno_t_SVN_ERR_FS_NO_USER as _,
            fs_path_already_locked: ffi::svn_errno_t_SVN_ERR_FS_PATH_ALREADY_LOCKED as _,
            fs_path_not_locked: ffi::svn_errno_t_SVN_ERR_FS_PATH_NOT_LOCKED as _,
            fs_bad_lock_token: ffi::svn_errno_t_SVN_ERR_FS_BAD_LOCK_TOKEN as _,
            fs_no_lock_token: ffi::svn_errno_t_SVN_ERR_FS_NO_LOCK_TOKEN as _,
            fs_lock_owner_mismatch: ffi::svn_errno_t_SVN_ERR_FS_LOCK_OWNER_MISMATCH as _,
            fs_no_such_lock: ffi::svn_errno_t_SVN_ERR_FS_NO_SUCH_LOCK as _,
            fs_lock_expired: ffi::svn_errno_t_SVN_ERR_FS_LOCK_EXPIRED as _,
            fs_out_of_date: ffi::svn_errno_t_SVN_ERR_FS_OUT_OF_DATE as _,
            fs_unsupported_format: ffi::svn_errno_t_SVN_ERR_FS_UNSUPPORTED_FORMAT as _,
            fs_rep_being_written: ffi::svn_errno_t_SVN_ERR_FS_REP_BEING_WRITTEN as _,
            fs_txn_name_too_long: ffi::svn_errno_t_SVN_ERR_FS_TXN_NAME_TOO_LONG as _,
            fs_no_such_node_origin: ffi::svn_errno_t_SVN_ERR_FS_NO_SUCH_NODE_ORIGIN as _,
            fs_unsupported_upgrade: ffi::svn_errno_t_SVN_ERR_FS_UNSUPPORTED_UPGRADE as _,
            fs_no_such_checksum_rep: ffi::svn_errno_t_SVN_ERR_FS_NO_SUCH_CHECKSUM_REP as _,
            fs_prop_basevalue_mismatch: ffi::svn_errno_t_SVN_ERR_FS_PROP_BASEVALUE_MISMATCH as _,
            fs_incorrect_editor_completion: ffi::svn_errno_t_SVN_ERR_FS_INCORRECT_EDITOR_COMPLETION
                as _,
            fs_packed_revprop_read_failure: ffi::svn_errno_t_SVN_ERR_FS_PACKED_REVPROP_READ_FAILURE
                as _,
            fs_revprop_cache_init_failure: ffi::svn_errno_t_SVN_ERR_FS_REVPROP_CACHE_INIT_FAILURE
                as _,
            fs_malformed_txn_id: ffi::svn_errno_t_SVN_ERR_FS_MALFORMED_TXN_ID as _,
            fs_index_corruption: ffi::svn_errno_t_SVN_ERR_FS_INDEX_CORRUPTION as _,
            fs_index_revision: ffi::svn_errno_t_SVN_ERR_FS_INDEX_REVISION as _,
            fs_index_overflow: ffi::svn_errno_t_SVN_ERR_FS_INDEX_OVERFLOW as _,
            fs_container_index: ffi::svn_errno_t_SVN_ERR_FS_CONTAINER_INDEX as _,
            fs_index_inconsistent: ffi::svn_errno_t_SVN_ERR_FS_INDEX_INCONSISTENT as _,
            fs_lock_operation_failed: ffi::svn_errno_t_SVN_ERR_FS_LOCK_OPERATION_FAILED as _,
            fs_unsupported_type: ffi::svn_errno_t_SVN_ERR_FS_UNSUPPORTED_TYPE as _,
            fs_container_size: ffi::svn_errno_t_SVN_ERR_FS_CONTAINER_SIZE as _,
            fs_malformed_noderev_id: ffi::svn_errno_t_SVN_ERR_FS_MALFORMED_NODEREV_ID as _,
            fs_invalid_generation: ffi::svn_errno_t_SVN_ERR_FS_INVALID_GENERATION as _,
            fs_corrupt_revprop_manifest: ffi::svn_errno_t_SVN_ERR_FS_CORRUPT_REVPROP_MANIFEST as _,
            fs_corrupt_proplist: ffi::svn_errno_t_SVN_ERR_FS_CORRUPT_PROPLIST as _,
            fs_ambiguous_checksum_rep: ffi::svn_errno_t_SVN_ERR_FS_AMBIGUOUS_CHECKSUM_REP as _,
            fs_unrecognized_ioctl_code: ffi::svn_errno_t_SVN_ERR_FS_UNRECOGNIZED_IOCTL_CODE as _,
            fs_rep_sharing_not_allowed: ffi::svn_errno_t_SVN_ERR_FS_REP_SHARING_NOT_ALLOWED as _,
            fs_rep_sharing_not_supported: ffi::svn_errno_t_SVN_ERR_FS_REP_SHARING_NOT_SUPPORTED
                as _,
            repos_locked: ffi::svn_errno_t_SVN_ERR_REPOS_LOCKED as _,
            repos_hook_failure: ffi::svn_errno_t_SVN_ERR_REPOS_HOOK_FAILURE as _,
            repos_bad_args: ffi::svn_errno_t_SVN_ERR_REPOS_BAD_ARGS as _,
            repos_no_data_for_report: ffi::svn_errno_t_SVN_ERR_REPOS_NO_DATA_FOR_REPORT as _,
            repos_bad_revision_report: ffi::svn_errno_t_SVN_ERR_REPOS_BAD_REVISION_REPORT as _,
            repos_unsupported_version: ffi::svn_errno_t_SVN_ERR_REPOS_UNSUPPORTED_VERSION as _,
            repos_disabled_feature: ffi::svn_errno_t_SVN_ERR_REPOS_DISABLED_FEATURE as _,
            repos_post_commit_hook_failed: ffi::svn_errno_t_SVN_ERR_REPOS_POST_COMMIT_HOOK_FAILED
                as _,
            repos_post_lock_hook_failed: ffi::svn_errno_t_SVN_ERR_REPOS_POST_LOCK_HOOK_FAILED as _,
            repos_post_unlock_hook_failed: ffi::svn_errno_t_SVN_ERR_REPOS_POST_UNLOCK_HOOK_FAILED
                as _,
            repos_unsupported_upgrade: ffi::svn_errno_t_SVN_ERR_REPOS_UNSUPPORTED_UPGRADE as _,
            ra_illegal_url: ffi::svn_errno_t_SVN_ERR_RA_ILLEGAL_URL as _,
            ra_not_authorized: ffi::svn_errno_t_SVN_ERR_RA_NOT_AUTHORIZED as _,
            ra_unknown_auth: ffi::svn_errno_t_SVN_ERR_RA_UNKNOWN_AUTH as _,
            ra_not_implemented: ffi::svn_errno_t_SVN_ERR_RA_NOT_IMPLEMENTED as _,
            ra_out_of_date: ffi::svn_errno_t_SVN_ERR_RA_OUT_OF_DATE as _,
            ra_no_repos_uuid: ffi::svn_errno_t_SVN_ERR_RA_NO_REPOS_UUID as _,
            ra_unsupported_abi_version: ffi::svn_errno_t_SVN_ERR_RA_UNSUPPORTED_ABI_VERSION as _,
            ra_not_locked: ffi::svn_errno_t_SVN_ERR_RA_NOT_LOCKED as _,
            ra_partial_replay_not_supported:
                ffi::svn_errno_t_SVN_ERR_RA_PARTIAL_REPLAY_NOT_SUPPORTED as _,
            ra_uuid_mismatch: ffi::svn_errno_t_SVN_ERR_RA_UUID_MISMATCH as _,
            ra_repos_root_url_mismatch: ffi::svn_errno_t_SVN_ERR_RA_REPOS_ROOT_URL_MISMATCH as _,
            ra_session_url_mismatch: ffi::svn_errno_t_SVN_ERR_RA_SESSION_URL_MISMATCH as _,
            ra_cannot_create_tunnel: ffi::svn_errno_t_SVN_ERR_RA_CANNOT_CREATE_TUNNEL as _,
            ra_cannot_create_session: ffi::svn_errno_t_SVN_ERR_RA_CANNOT_CREATE_SESSION as _,
            ra_dav_sock_init: ffi::svn_errno_t_SVN_ERR_RA_DAV_SOCK_INIT as _,
            ra_dav_creating_request: ffi::svn_errno_t_SVN_ERR_RA_DAV_CREATING_REQUEST as _,
            ra_dav_request_failed: ffi::svn_errno_t_SVN_ERR_RA_DAV_REQUEST_FAILED as _,
            ra_dav_options_req_failed: ffi::svn_errno_t_SVN_ERR_RA_DAV_OPTIONS_REQ_FAILED as _,
            ra_dav_props_not_found: ffi::svn_errno_t_SVN_ERR_RA_DAV_PROPS_NOT_FOUND as _,
            ra_dav_already_exists: ffi::svn_errno_t_SVN_ERR_RA_DAV_ALREADY_EXISTS as _,
            ra_dav_invalid_config_value: ffi::svn_errno_t_SVN_ERR_RA_DAV_INVALID_CONFIG_VALUE as _,
            ra_dav_path_not_found: ffi::svn_errno_t_SVN_ERR_RA_DAV_PATH_NOT_FOUND as _,
            ra_dav_proppatch_failed: ffi::svn_errno_t_SVN_ERR_RA_DAV_PROPPATCH_FAILED as _,
            ra_dav_malformed_data: ffi::svn_errno_t_SVN_ERR_RA_DAV_MALFORMED_DATA as _,
            ra_dav_response_header_badness: ffi::svn_errno_t_SVN_ERR_RA_DAV_RESPONSE_HEADER_BADNESS
                as _,
            ra_dav_relocated: ffi::svn_errno_t_SVN_ERR_RA_DAV_RELOCATED as _,
            ra_dav_conn_timeout: ffi::svn_errno_t_SVN_ERR_RA_DAV_CONN_TIMEOUT as _,
            ra_dav_forbidden: ffi::svn_errno_t_SVN_ERR_RA_DAV_FORBIDDEN as _,
            ra_dav_precondition_failed: ffi::svn_errno_t_SVN_ERR_RA_DAV_PRECONDITION_FAILED as _,
            ra_dav_method_not_allowed: ffi::svn_errno_t_SVN_ERR_RA_DAV_METHOD_NOT_ALLOWED as _,
            ra_local_repos_not_found: ffi::svn_errno_t_SVN_ERR_RA_LOCAL_REPOS_NOT_FOUND as _,
            ra_local_repos_open_failed: ffi::svn_errno_t_SVN_ERR_RA_LOCAL_REPOS_OPEN_FAILED as _,
            svndiff_invalid_header: ffi::svn_errno_t_SVN_ERR_SVNDIFF_INVALID_HEADER as _,
            svndiff_corrupt_window: ffi::svn_errno_t_SVN_ERR_SVNDIFF_CORRUPT_WINDOW as _,
            svndiff_backward_view: ffi::svn_errno_t_SVN_ERR_SVNDIFF_BACKWARD_VIEW as _,
            svndiff_invalid_ops: ffi::svn_errno_t_SVN_ERR_SVNDIFF_INVALID_OPS as _,
            svndiff_unexpected_end: ffi::svn_errno_t_SVN_ERR_SVNDIFF_UNEXPECTED_END as _,
            svndiff_invalid_compressed_data:
                ffi::svn_errno_t_SVN_ERR_SVNDIFF_INVALID_COMPRESSED_DATA as _,
            apmod_missing_path_to_fs: ffi::svn_errno_t_SVN_ERR_APMOD_MISSING_PATH_TO_FS as _,
            apmod_malformed_uri: ffi::svn_errno_t_SVN_ERR_APMOD_MALFORMED_URI as _,
            apmod_activity_not_found: ffi::svn_errno_t_SVN_ERR_APMOD_ACTIVITY_NOT_FOUND as _,
            apmod_bad_baseline: ffi::svn_errno_t_SVN_ERR_APMOD_BAD_BASELINE as _,
            apmod_connection_aborted: ffi::svn_errno_t_SVN_ERR_APMOD_CONNECTION_ABORTED as _,
            client_versioned_path_required: ffi::svn_errno_t_SVN_ERR_CLIENT_VERSIONED_PATH_REQUIRED
                as _,
            client_ra_access_required: ffi::svn_errno_t_SVN_ERR_CLIENT_RA_ACCESS_REQUIRED as _,
            client_bad_revision: ffi::svn_errno_t_SVN_ERR_CLIENT_BAD_REVISION as _,
            client_duplicate_commit_url: ffi::svn_errno_t_SVN_ERR_CLIENT_DUPLICATE_COMMIT_URL as _,
            client_is_binary_file: ffi::svn_errno_t_SVN_ERR_CLIENT_IS_BINARY_FILE as _,
            client_invalid_externals_description:
                ffi::svn_errno_t_SVN_ERR_CLIENT_INVALID_EXTERNALS_DESCRIPTION as _,
            client_modified: ffi::svn_errno_t_SVN_ERR_CLIENT_MODIFIED as _,
            client_is_directory: ffi::svn_errno_t_SVN_ERR_CLIENT_IS_DIRECTORY as _,
            client_revision_range: ffi::svn_errno_t_SVN_ERR_CLIENT_REVISION_RANGE as _,
            client_invalid_relocation: ffi::svn_errno_t_SVN_ERR_CLIENT_INVALID_RELOCATION as _,
            client_revision_author_contains_newline:
                ffi::svn_errno_t_SVN_ERR_CLIENT_REVISION_AUTHOR_CONTAINS_NEWLINE as _,
            client_property_name: ffi::svn_errno_t_SVN_ERR_CLIENT_PROPERTY_NAME as _,
            client_unrelated_resources: ffi::svn_errno_t_SVN_ERR_CLIENT_UNRELATED_RESOURCES as _,
            client_missing_lock_token: ffi::svn_errno_t_SVN_ERR_CLIENT_MISSING_LOCK_TOKEN as _,
            client_multiple_sources_disallowed:
                ffi::svn_errno_t_SVN_ERR_CLIENT_MULTIPLE_SOURCES_DISALLOWED as _,
            client_no_versioned_parent: ffi::svn_errno_t_SVN_ERR_CLIENT_NO_VERSIONED_PARENT as _,
            client_not_ready_to_merge: ffi::svn_errno_t_SVN_ERR_CLIENT_NOT_READY_TO_MERGE as _,
            client_file_external_overwrite_versioned:
                ffi::svn_errno_t_SVN_ERR_CLIENT_FILE_EXTERNAL_OVERWRITE_VERSIONED as _,
            client_patch_bad_strip_count: ffi::svn_errno_t_SVN_ERR_CLIENT_PATCH_BAD_STRIP_COUNT
                as _,
            client_cycle_detected: ffi::svn_errno_t_SVN_ERR_CLIENT_CYCLE_DETECTED as _,
            client_merge_update_required: ffi::svn_errno_t_SVN_ERR_CLIENT_MERGE_UPDATE_REQUIRED
                as _,
            client_invalid_mergeinfo_no_mergetracking:
                ffi::svn_errno_t_SVN_ERR_CLIENT_INVALID_MERGEINFO_NO_MERGETRACKING as _,
            client_no_lock_token: ffi::svn_errno_t_SVN_ERR_CLIENT_NO_LOCK_TOKEN as _,
            client_forbidden_by_server: ffi::svn_errno_t_SVN_ERR_CLIENT_FORBIDDEN_BY_SERVER as _,
            client_conflict_option_not_applicable:
                ffi::svn_errno_t_SVN_ERR_CLIENT_CONFLICT_OPTION_NOT_APPLICABLE as _,
            base: ffi::svn_errno_t_SVN_ERR_BASE as _,
            plugin_load_failure: ffi::svn_errno_t_SVN_ERR_PLUGIN_LOAD_FAILURE as _,
            malformed_file: ffi::svn_errno_t_SVN_ERR_MALFORMED_FILE as _,
            incomplete_data: ffi::svn_errno_t_SVN_ERR_INCOMPLETE_DATA as _,
            incorrect_params: ffi::svn_errno_t_SVN_ERR_INCORRECT_PARAMS as _,
            unversioned_resource: ffi::svn_errno_t_SVN_ERR_UNVERSIONED_RESOURCE as _,
            test_failed: ffi::svn_errno_t_SVN_ERR_TEST_FAILED as _,
            unsupported_feature: ffi::svn_errno_t_SVN_ERR_UNSUPPORTED_FEATURE as _,
            bad_prop_kind: ffi::svn_errno_t_SVN_ERR_BAD_PROP_KIND as _,
            illegal_target: ffi::svn_errno_t_SVN_ERR_ILLEGAL_TARGET as _,
            delta_md5_checksum_absent: ffi::svn_errno_t_SVN_ERR_DELTA_MD5_CHECKSUM_ABSENT as _,
            dir_not_empty: ffi::svn_errno_t_SVN_ERR_DIR_NOT_EMPTY as _,
            external_program: ffi::svn_errno_t_SVN_ERR_EXTERNAL_PROGRAM as _,
            swig_py_exception_set: ffi::svn_errno_t_SVN_ERR_SWIG_PY_EXCEPTION_SET as _,
            checksum_mismatch: ffi::svn_errno_t_SVN_ERR_CHECKSUM_MISMATCH as _,
            cancelled: ffi::svn_errno_t_SVN_ERR_CANCELLED as _,
            invalid_diff_option: ffi::svn_errno_t_SVN_ERR_INVALID_DIFF_OPTION as _,
            property_not_found: ffi::svn_errno_t_SVN_ERR_PROPERTY_NOT_FOUND as _,
            no_auth_file_path: ffi::svn_errno_t_SVN_ERR_NO_AUTH_FILE_PATH as _,
            version_mismatch: ffi::svn_errno_t_SVN_ERR_VERSION_MISMATCH as _,
            mergeinfo_parse_error: ffi::svn_errno_t_SVN_ERR_MERGEINFO_PARSE_ERROR as _,
            cease_invocation: ffi::svn_errno_t_SVN_ERR_CEASE_INVOCATION as _,
            revnum_parse_failure: ffi::svn_errno_t_SVN_ERR_REVNUM_PARSE_FAILURE as _,
            iter_break: ffi::svn_errno_t_SVN_ERR_ITER_BREAK as _,
            unknown_changelist: ffi::svn_errno_t_SVN_ERR_UNKNOWN_CHANGELIST as _,
            reserved_filename_specified: ffi::svn_errno_t_SVN_ERR_RESERVED_FILENAME_SPECIFIED as _,
            unknown_capability: ffi::svn_errno_t_SVN_ERR_UNKNOWN_CAPABILITY as _,
            test_skipped: ffi::svn_errno_t_SVN_ERR_TEST_SKIPPED as _,
            no_apr_memcache: ffi::svn_errno_t_SVN_ERR_NO_APR_MEMCACHE as _,
            atomic_init_failure: ffi::svn_errno_t_SVN_ERR_ATOMIC_INIT_FAILURE as _,
            sqlite_error: ffi::svn_errno_t_SVN_ERR_SQLITE_ERROR as _,
            sqlite_readonly: ffi::svn_errno_t_SVN_ERR_SQLITE_READONLY as _,
            sqlite_unsupported_schema: ffi::svn_errno_t_SVN_ERR_SQLITE_UNSUPPORTED_SCHEMA as _,
            sqlite_busy: ffi::svn_errno_t_SVN_ERR_SQLITE_BUSY as _,
            sqlite_resetting_for_rollback: ffi::svn_errno_t_SVN_ERR_SQLITE_RESETTING_FOR_ROLLBACK
                as _,
            sqlite_constraint: ffi::svn_errno_t_SVN_ERR_SQLITE_CONSTRAINT as _,
            too_many_memcached_servers: ffi::svn_errno_t_SVN_ERR_TOO_MANY_MEMCACHED_SERVERS as _,
            malformed_version_string: ffi::svn_errno_t_SVN_ERR_MALFORMED_VERSION_STRING as _,
            corrupted_atomic_storage: ffi::svn_errno_t_SVN_ERR_CORRUPTED_ATOMIC_STORAGE as _,
            utf8proc_error: ffi::svn_errno_t_SVN_ERR_UTF8PROC_ERROR as _,
            utf8_glob: ffi::svn_errno_t_SVN_ERR_UTF8_GLOB as _,
            corrupt_packed_data: ffi::svn_errno_t_SVN_ERR_CORRUPT_PACKED_DATA as _,
            composed_error: ffi::svn_errno_t_SVN_ERR_COMPOSED_ERROR as _,
            invalid_input: ffi::svn_errno_t_SVN_ERR_INVALID_INPUT as _,
            sqlite_rollback_failed: ffi::svn_errno_t_SVN_ERR_SQLITE_ROLLBACK_FAILED as _,
            lz4_compression_failed: ffi::svn_errno_t_SVN_ERR_LZ4_COMPRESSION_FAILED as _,
            lz4_decompression_failed: ffi::svn_errno_t_SVN_ERR_LZ4_DECOMPRESSION_FAILED as _,
            canonicalization_failed: ffi::svn_errno_t_SVN_ERR_CANONICALIZATION_FAILED as _,
            cl_arg_parsing_error: ffi::svn_errno_t_SVN_ERR_CL_ARG_PARSING_ERROR as _,
            cl_insufficient_args: ffi::svn_errno_t_SVN_ERR_CL_INSUFFICIENT_ARGS as _,
            cl_mutually_exclusive_args: ffi::svn_errno_t_SVN_ERR_CL_MUTUALLY_EXCLUSIVE_ARGS as _,
            cl_adm_dir_reserved: ffi::svn_errno_t_SVN_ERR_CL_ADM_DIR_RESERVED as _,
            cl_log_message_is_versioned_file:
                ffi::svn_errno_t_SVN_ERR_CL_LOG_MESSAGE_IS_VERSIONED_FILE as _,
            cl_log_message_is_pathname: ffi::svn_errno_t_SVN_ERR_CL_LOG_MESSAGE_IS_PATHNAME as _,
            cl_commit_in_added_dir: ffi::svn_errno_t_SVN_ERR_CL_COMMIT_IN_ADDED_DIR as _,
            cl_no_external_editor: ffi::svn_errno_t_SVN_ERR_CL_NO_EXTERNAL_EDITOR as _,
            cl_bad_log_message: ffi::svn_errno_t_SVN_ERR_CL_BAD_LOG_MESSAGE as _,
            cl_unnecessary_log_message: ffi::svn_errno_t_SVN_ERR_CL_UNNECESSARY_LOG_MESSAGE as _,
            cl_no_external_merge_tool: ffi::svn_errno_t_SVN_ERR_CL_NO_EXTERNAL_MERGE_TOOL as _,
            cl_error_processing_externals: ffi::svn_errno_t_SVN_ERR_CL_ERROR_PROCESSING_EXTERNALS
                as _,
            cl_repos_verify_failed: ffi::svn_errno_t_SVN_ERR_CL_REPOS_VERIFY_FAILED as _,
            ra_svn_cmd_err: ffi::svn_errno_t_SVN_ERR_RA_SVN_CMD_ERR as _,
            ra_svn_unknown_cmd: ffi::svn_errno_t_SVN_ERR_RA_SVN_UNKNOWN_CMD as _,
            ra_svn_connection_closed: ffi::svn_errno_t_SVN_ERR_RA_SVN_CONNECTION_CLOSED as _,
            ra_svn_io_error: ffi::svn_errno_t_SVN_ERR_RA_SVN_IO_ERROR as _,
            ra_svn_malformed_data: ffi::svn_errno_t_SVN_ERR_RA_SVN_MALFORMED_DATA as _,
            ra_svn_repos_not_found: ffi::svn_errno_t_SVN_ERR_RA_SVN_REPOS_NOT_FOUND as _,
            ra_svn_bad_version: ffi::svn_errno_t_SVN_ERR_RA_SVN_BAD_VERSION as _,
            ra_svn_no_mechanisms: ffi::svn_errno_t_SVN_ERR_RA_SVN_NO_MECHANISMS as _,
            ra_svn_edit_aborted: ffi::svn_errno_t_SVN_ERR_RA_SVN_EDIT_ABORTED as _,
            ra_svn_request_size: ffi::svn_errno_t_SVN_ERR_RA_SVN_REQUEST_SIZE as _,
            ra_svn_response_size: ffi::svn_errno_t_SVN_ERR_RA_SVN_RESPONSE_SIZE as _,
            authn_creds_unavailable: ffi::svn_errno_t_SVN_ERR_AUTHN_CREDS_UNAVAILABLE as _,
            authn_no_provider: ffi::svn_errno_t_SVN_ERR_AUTHN_NO_PROVIDER as _,
            authn_providers_exhausted: ffi::svn_errno_t_SVN_ERR_AUTHN_PROVIDERS_EXHAUSTED as _,
            authn_creds_not_saved: ffi::svn_errno_t_SVN_ERR_AUTHN_CREDS_NOT_SAVED as _,
            authn_failed: ffi::svn_errno_t_SVN_ERR_AUTHN_FAILED as _,
            authz_root_unreadable: ffi::svn_errno_t_SVN_ERR_AUTHZ_ROOT_UNREADABLE as _,
            authz_unreadable: ffi::svn_errno_t_SVN_ERR_AUTHZ_UNREADABLE as _,
            authz_partially_readable: ffi::svn_errno_t_SVN_ERR_AUTHZ_PARTIALLY_READABLE as _,
            authz_invalid_config: ffi::svn_errno_t_SVN_ERR_AUTHZ_INVALID_CONFIG as _,
            authz_unwritable: ffi::svn_errno_t_SVN_ERR_AUTHZ_UNWRITABLE as _,
            diff_datasource_modified: ffi::svn_errno_t_SVN_ERR_DIFF_DATASOURCE_MODIFIED as _,
            diff_unexpected_data: ffi::svn_errno_t_SVN_ERR_DIFF_UNEXPECTED_DATA as _,
            ra_serf_sspi_initialisation_failed:
                ffi::svn_errno_t_SVN_ERR_RA_SERF_SSPI_INITIALISATION_FAILED as _,
            ra_serf_ssl_cert_untrusted: ffi::svn_errno_t_SVN_ERR_RA_SERF_SSL_CERT_UNTRUSTED as _,
            ra_serf_gssapi_initialisation_failed:
                ffi::svn_errno_t_SVN_ERR_RA_SERF_GSSAPI_INITIALISATION_FAILED as _,
            ra_serf_wrapped_error: ffi::svn_errno_t_SVN_ERR_RA_SERF_WRAPPED_ERROR as _,
            ra_serf_stream_bucket_read_error:
                ffi::svn_errno_t_SVN_ERR_RA_SERF_STREAM_BUCKET_READ_ERROR as _,
            assertion_fail: ffi::svn_errno_t_SVN_ERR_ASSERTION_FAIL as _,
            assertion_only_tracing_links: ffi::svn_errno_t_SVN_ERR_ASSERTION_ONLY_TRACING_LINKS
                as _,
            asn1_out_of_data: ffi::svn_errno_t_SVN_ERR_ASN1_OUT_OF_DATA as _,
            asn1_unexpected_tag: ffi::svn_errno_t_SVN_ERR_ASN1_UNEXPECTED_TAG as _,
            asn1_invalid_length: ffi::svn_errno_t_SVN_ERR_ASN1_INVALID_LENGTH as _,
            asn1_length_mismatch: ffi::svn_errno_t_SVN_ERR_ASN1_LENGTH_MISMATCH as _,
            asn1_invalid_data: ffi::svn_errno_t_SVN_ERR_ASN1_INVALID_DATA as _,
            x509_feature_unavailable: ffi::svn_errno_t_SVN_ERR_X509_FEATURE_UNAVAILABLE as _,
            x509_cert_invalid_pem: ffi::svn_errno_t_SVN_ERR_X509_CERT_INVALID_PEM as _,
            x509_cert_invalid_format: ffi::svn_errno_t_SVN_ERR_X509_CERT_INVALID_FORMAT as _,
            x509_cert_invalid_version: ffi::svn_errno_t_SVN_ERR_X509_CERT_INVALID_VERSION as _,
            x509_cert_invalid_serial: ffi::svn_errno_t_SVN_ERR_X509_CERT_INVALID_SERIAL as _,
            x509_cert_invalid_alg: ffi::svn_errno_t_SVN_ERR_X509_CERT_INVALID_ALG as _,
            x509_cert_invalid_name: ffi::svn_errno_t_SVN_ERR_X509_CERT_INVALID_NAME as _,
            x509_cert_invalid_date: ffi::svn_errno_t_SVN_ERR_X509_CERT_INVALID_DATE as _,
            x509_cert_invalid_pubkey: ffi::svn_errno_t_SVN_ERR_X509_CERT_INVALID_PUBKEY as _,
            x509_cert_invalid_signature: ffi::svn_errno_t_SVN_ERR_X509_CERT_INVALID_SIGNATURE as _,
            x509_cert_invalid_extensions: ffi::svn_errno_t_SVN_ERR_X509_CERT_INVALID_EXTENSIONS
                as _,
            x509_cert_unknown_version: ffi::svn_errno_t_SVN_ERR_X509_CERT_UNKNOWN_VERSION as _,
            x509_cert_unknown_pk_alg: ffi::svn_errno_t_SVN_ERR_X509_CERT_UNKNOWN_PK_ALG as _,
            x509_cert_sig_mismatch: ffi::svn_errno_t_SVN_ERR_X509_CERT_SIG_MISMATCH as _,
            x509_cert_verify_failed: ffi::svn_errno_t_SVN_ERR_X509_CERT_VERIFY_FAILED as _,
        }
    }
}

unsafe extern "C" fn may_save_password_as_plain_text(
    save: *mut ffi::svn_boolean_t,
    realm_string: *const c_char,
    baton: *mut c_void,
    pool: *mut ffi::apr_pool_t,
) -> *mut ffi::svn_error_t {
    let ctx = unsafe { &mut *(baton as *mut ContextInner) };

    unsafe {
        let realm_string = CStr::from_ptr(realm_string).to_str().unwrap();
        // *save = (ctx.on_may_save_password_as_plain_text)(realm_string).into();
        *save = ctx
            .context_notifier
            .may_save_password_as_plain_text(realm_string.to_string())
            .into();
    }

    std::ptr::null_mut()
}

unsafe extern "C" fn first_ssl_client_cert_pw() -> *mut ffi::svn_error_t {
    svn_no_error()
}

unsafe extern "C" fn info_receiver(
    baton: *mut c_void,
    path: *const c_char,
    info: *const ffi::svn_client_info2_t,
    scratch_pool: *mut apr::ffi::apr_pool_t,
) -> *mut ffi::svn_error_t {
    unsafe {
        let context = (baton as *mut ContextInner).as_mut().unwrap();
    }

    svn_no_error()
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SslServerCertInfo {
    hostname: String,
    fingerprint: String,
    valid_from: String,
    valid_until: String,
    issuer: String,
    ascii_cert: String,
}

#[derive(Debug, Clone, Copy)]
struct CredSslServerTrust {
    accepted_failures: SslFailures,
    may_save: bool,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, Default)]
    pub struct SslFailures: u32 {
        const NOTYETVALID = ffi::SVN_AUTH_SSL_NOTYETVALID;
        const EXPIRED = ffi::SVN_AUTH_SSL_EXPIRED;
        const CNMISMATCH = ffi::SVN_AUTH_SSL_CNMISMATCH;
        const UNKNOWNCA = ffi::SVN_AUTH_SSL_UNKNOWNCA;
        const OTHER = ffi::SVN_AUTH_SSL_OTHER;
    }
}

impl SslServerCertInfo {
    unsafe fn from_raw(info: *const ffi::svn_auth_ssl_server_cert_info_t) -> Self {
        let info = unsafe { info.as_ref().unwrap() };
        unsafe {
            let hostname = apr::char_array_to_string(info.hostname).unwrap();
            let fingerprint = apr::char_array_to_string(info.fingerprint).unwrap();
            let valid_from = apr::char_array_to_string(info.valid_from).unwrap();
            let valid_until = apr::char_array_to_string(info.valid_until).unwrap();
            let issuer = apr::char_array_to_string(info.issuer_dname).unwrap();
            let ascii_cert = apr::char_array_to_string(info.ascii_cert).unwrap();

            Self {
                hostname,
                fingerprint,
                valid_from,
                valid_until,
                issuer,
                ascii_cert,
            }
        }
    }
}

#[derive(Debug, Clone, Copy, new, Default, uniffi::Record)]
pub struct TrustServer {
    accept_failures: u32,
    save: bool,
}

#[derive(Debug, Clone, new, Default, uniffi::Record)]
pub struct Authentication {
    pub username: String,
    pub password: String,
    pub save: bool,
}

unsafe extern "C" fn ssl_server_trust_prompt(
    cred: *mut *mut ffi::svn_auth_cred_ssl_server_trust_t,
    baton: *mut c_void,
    realm: *const c_char,
    failures: ffi::apr_uint32_t,
    info: *const ffi::svn_auth_ssl_server_cert_info_t,
    may_save: ffi::svn_boolean_t,
    pool: *mut ffi::apr_pool_t,
) -> *mut ffi::svn_error_t {
    unsafe {
        let realm = CStr::from_ptr(realm).to_str().unwrap();
        let info = SslServerCertInfo::from_raw(info);
        tracing::info!(
            "Server info: realm={} failures={} info={:?} may_save={}",
            realm,
            failures,
            info,
            may_save
        );

        let this = &mut *(baton as *mut ContextInner);

        let trust = this.context_notifier.ssl_server_trust_prompt(
            realm.to_string(),
            failures,
            // SslFailures::from_bits_retain(failures),
            info,
            may_save != 0,
        );

        match trust {
            Some(trust) => {
                let mut pool = ManuallyDrop::new(apr::Pool::from_raw(pool));

                *cred = pool.malloc::<ffi::svn_auth_cred_ssl_server_trust_t>();
                let cred = &mut **cred;

                cred.accepted_failures = trust.accept_failures;
                cred.may_save = trust.save as _;
            }
            None => {
                *cred = std::ptr::null_mut();
            }
        }

        svn_no_error()
    }
}

// #[derive(TryFromPrimitive)]
// #[repr(u8)]
#[svnexus_macro::enum_converter(repr_type=u8)]
#[derive(uniffi::Enum, Debug)]
enum LogChangedPathAction {
    Add = b'A',
    Delete = b'D',
    Replace = b'R',
    Modify = b'M',
}
impl LogChangedPathAction {
    fn from_i8(value: i8) -> Self {
        Self::try_from(u8::try_from(value).unwrap()).unwrap()
    }
}

#[derive(new, Debug, uniffi::Record)]
pub struct LogChangedPathEntry {
    action: LogChangedPathAction,
    copy_from_path: String,
    copy_from_revision: RevisionNumber,
    node_kind: NodeKind,
    text_modified: Option<bool>,
    props_modified: Option<bool>,
}

fn from_svn_stristate(value: u32) -> Option<bool> {
    match value {
        ffi::svn_tristate_t_svn_tristate_true => Some(true),
        ffi::svn_tristate_t_svn_tristate_false => Some(false),
        ffi::svn_tristate_t_svn_tristate_unknown => None,
        _ => panic!("Invalid svn tristate value: {}", value),
    }
}

impl LogChangedPathEntry {
    unsafe fn from_raw(ptr: *const ffi::svn_log_changed_path2_t) -> Self {
        unsafe {
            let ptr = &*ptr;

            let action = LogChangedPathAction::from_i8(ptr.action);
            let copy_from_path = apr::char_array_to_string(ptr.copyfrom_path).unwrap();
            let copy_from_revision = ptr.copyfrom_rev.try_into().unwrap();

            let node_kind = NodeKind::try_from(ptr.node_kind).unwrap();

            let text_modified = from_svn_stristate(ptr.text_modified);

            let props_modified = from_svn_stristate(ptr.props_modified);

            Self {
                action,
                copy_from_path,
                copy_from_revision,
                node_kind,
                text_modified,
                props_modified,
            }
        }
    }
}

#[derive(Default, new, Debug, uniffi::Record)]
pub struct LogEntry {
    revision: RevisionNumber,
    date: Option<i64>,
    author: Option<String>,
    message: Option<String>,
    changed_path_entries: HashMap<String, LogChangedPathEntry>,
    has_children: bool,
    non_inheritable: bool,
    subtractive_merge: bool, // merged_in_revsions: Vec<RevisionNumber>,
}

impl LogEntry {
    fn from_raw(ptr: *mut ffi::svn_log_entry_t) -> Self {
        unsafe {
            let log_entry = &*ptr;

            let mut author: *const c_char = std::ptr::null();
            let mut date: *const c_char = std::ptr::null();
            let mut message: *const c_char = std::ptr::null();

            ffi::svn_compat_log_revprops_out(
                &mut author as _,
                &mut date as _,
                &mut message as _,
                log_entry.revprops,
            );

            let revision = log_entry.revision.try_into().unwrap();

            let mut pool = apr::Pool::create();

            let mut time_is_valid = false;
            let mut time: ffi::apr_time_t = 0;

            if !date.is_null() {
                let error =
                    ffi::svn_time_from_cstring(&mut time as *mut _, date, pool.as_mut_ptr());
                if error.is_null() {
                    time_is_valid = true;
                } else {
                    ffi::svn_error_clear(error);
                }
            }

            let author = apr::char_array_to_string(author);
            let message = apr::char_array_to_string(message);
            let date = if time_is_valid { Some(time) } else { None };

            let mut changed_path_entries = HashMap::new();

            if !log_entry.changed_paths2.is_null() {
                let mut it = ffi::apr_hash_first(pool.as_mut_ptr(), log_entry.changed_paths2);

                while !it.is_null() {
                    let mut key = std::ptr::null();
                    let mut value = std::ptr::null_mut();

                    ffi::apr_hash_this(
                        it,
                        &mut key as *mut _,
                        std::ptr::null_mut(),
                        &mut value as *mut _,
                    );

                    let path = apr::char_array_to_string(key as _).unwrap();

                    // let item = &*(value as *const ffi::svn_log_changed_path2_t);
                    let log_changed_path_entry = LogChangedPathEntry::from_raw(value as _);

                    changed_path_entries.insert(path, log_changed_path_entry);

                    it = ffi::apr_hash_next(it);
                }
            }

            let has_children = log_entry.has_children != 0;
            let non_inheritable = log_entry.non_inheritable != 0;
            let subtractive_merge = log_entry.subtractive_merge != 0;

            Self {
                revision,
                date,
                author,
                message,
                changed_path_entries,
                has_children,
                non_inheritable,
                subtractive_merge,
            }
        }
    }
}

unsafe extern "C" fn on_import_filter(
    baton: *mut c_void,
    filtered: *mut ffi::svn_boolean_t,
    local_abspath: *const c_char,
    direct: *const ffi::svn_io_dirent2_t,
    pool: *mut ffi::apr_pool_t,
) -> *mut ffi::svn_error_t {
    svn_no_error()
}

unsafe extern "C" fn on_receive_log_entry(
    baton: *mut c_void,
    log_entry: *mut ffi::svn_log_entry_t,
    pool: *mut ffi::apr_pool_t,
) -> *mut ffi::svn_error_t {
    unsafe {
        let error = on_cancel(baton);
        if !error.is_null() {
            return error;
        }
        // let log_entry = &*log_entry;

        let this = &mut *(baton as *mut ContextInner);

        // if log_entry.revision < 0 {
        // this.revision_statck.pop();
        // return svn_no_error();
        // }
        //
        //
        this.log_entries.push(LogEntry::from_raw(log_entry));
    }

    svn_no_error()
}

unsafe extern "C" fn on_authenticate(
    cred: *mut *mut ffi::svn_auth_cred_simple_t,
    baton: *mut c_void,
    realm: *const c_char,
    username: *const c_char,
    may_save: ffi::svn_boolean_t,
    pool: *mut ffi::apr_pool_t,
) -> *mut ffi::svn_error_t {
    unsafe {
        let cancel = on_cancel(baton);
        if !cancel.is_null() {
            return cancel;
        }

        let context = (baton as *mut ContextInner).as_mut().unwrap();
        if let Some(result) = context.context_notifier.authenticate(
            char_array_to_string(realm).unwrap(),
            char_array_to_string(username).unwrap(),
            may_save != 0,
        ) {
            let mut pool = ManuallyDrop::new(apr::Pool::from_raw(pool));

            *cred = pool.malloc();

            let cred = (*cred).as_mut().unwrap();

            cred.may_save = result.save.into();
            cred.password = pool.string(result.password).unwrap();
            cred.username = pool.string(result.username).unwrap();

            svn_no_error()
        } else {
            on_cancel(baton)
        }
    }
}

pub struct ContextFactory;

impl Drop for ContextFactory {
    fn drop(&mut self) {
        unsafe {
            ffi::apr_terminate2();
        }
    }
}

impl ContextFactory {
    pub fn create_context(&self, opts: CreateContextOptions) -> error::Result<Context> {
        Context::create(opts)
    }

    pub fn instance() -> error::Result<&'static ContextFactory> {
        static SELF: once_cell::sync::OnceCell<ContextFactory> = once_cell::sync::OnceCell::new();

        SELF.get_or_try_init(|| unsafe {
            apr::initialize()?;
            // let status = ffi::apr_initialize();
            // if status != 0 {
            //     return builder::General {
            //         detail: format!("Failed to initialize apache portable runtime: {}", status),
            //     }
            //     .fail();
            // }

            let error = ffi::svn_dso_initialize2();

            SVNError::from_nullable_ptr(error).context(builder::Svn)?;

            let pool = ffi::svn_pool_create_ex(std::ptr::null_mut(), std::ptr::null_mut());

            ffi::svn_utf_initialize2(0, pool);

            let error = ffi::svn_nls_init();

            SVNError::from_nullable_ptr(error).context(builder::Svn)?;

            Ok(ContextFactory {})
        })
    }
}

#[derive(Debug, uniffi::Record)]
pub struct RevertOptions {
    paths: Vec<String>,
    depth: Depth,
    changelists: Vec<String>,
    clear_changelists: bool,
    metadata_only: bool,
    added_keep_local: bool,
}

#[derive(uniffi::Record, Debug, new)]
pub struct RevisionRange {
    start: Revision,
    end: Revision,
}

#[derive(Debug, uniffi::Record)]
pub struct LogOptions {
    targets: Vec<String>,
    peg_revision: Revision,
    limit: u32,
    revsions: Vec<RevisionRange>,
    discover_changed_paths: bool,
    strict_node_history: bool,
    include_merged_revisions: bool,
    revisions_properties: Vec<String>,
}

#[derive(new, Debug, uniffi::Record)]
pub struct ImportResult {
    items: Vec<CommitItem>,
    info: CommitInfo,
}

#[derive(new, Debug, uniffi::Record)]
pub struct ImportOptions {
    path: String,
    url: String,
    depth: Depth,
    no_ignore: bool,
    no_autoprops: bool,
    ignore_unknown_node_types: bool,
    revision_property_table: HashMap<String, String>,
    commit_message: String,
}

impl apr::Pool {
    unsafe fn revision_range(
        &mut self,
        revsions: &[RevisionRange], // start: Revision,
                                    // end: Revision,
    ) -> *mut ffi::apr_array_header_t {
        unsafe {
            let ranges = ffi::apr_array_make(
                self.as_mut_ptr(),
                revsions.len().try_into().unwrap(),
                size_of::<usize>().try_into().unwrap(),
            );

            for i in revsions {
                let range: *mut ffi::svn_opt_revision_range_t = self.malloc();

                let ptr = ffi::apr_array_push(ranges) as *mut *mut ffi::svn_opt_revision_range_t;
                *ptr = range;

                let range = &mut *range;

                range.start = i.start.to_opt_revision();
                range.end = i.end.to_opt_revision();
            }

            ranges
        }
    }
}

#[derive(Debug, new, uniffi::Record)]
pub struct CatOptions {
    path: String,
    peg_revision: Revision,
    revision: Revision,
    expand_keywords: bool,
}

#[derive(Debug, new, uniffi::Record)]
pub struct CatResult {
    content: Vec<u8>,
    properties: HashMap<String, String>,
}

#[derive(Debug, uniffi::Record)]
pub struct InfoOptions {
    path: String,
    peg_revision: Revision,
    revision: Revision,
    depth: Depth,
    fetch_excluded: bool,
    fetch_actual_only: bool,
    include_externals: bool,
    changelists: Vec<String>,
}

#[derive(Debug, uniffi::Record)]
pub struct InfoResult {
    entries: HashMap<String, InfoEntry>,
}

#[derive(uniffi::Enum, Debug)]
pub enum CheckSum {
    Md5(Vec<u8>),
    Sha1(Vec<u8>),
    Fnv1a32(Vec<u8>),
    Fnv1a32x4(Vec<u8>),
}

#[easy_ext::ext]
impl *const ffi::svn_checksum_t {
    fn to_check_sum(self) -> CheckSum {
        unsafe {
            let ptr = self.as_ref().unwrap();

            match ptr.kind {
                ffi::svn_checksum_kind_t_svn_checksum_md5 => {
                    let digest = std::slice::from_raw_parts(ptr.digest, 16);

                    CheckSum::Md5(digest.to_vec())
                }
                ffi::svn_checksum_kind_t_svn_checksum_sha1 => {
                    let digest = std::slice::from_raw_parts(ptr.digest, 20);

                    CheckSum::Sha1(digest.to_vec())
                }
                ffi::svn_checksum_kind_t_svn_checksum_fnv1a_32 => {
                    let digest = std::slice::from_raw_parts(ptr.digest, 4);
                    CheckSum::Fnv1a32(digest.to_vec())
                }
                ffi::svn_checksum_kind_t_svn_checksum_fnv1a_32x4 => {
                    let digest = std::slice::from_raw_parts(ptr.digest, 4);
                    CheckSum::Fnv1a32x4(digest.to_vec())
                }
                kind => panic!("Invalid checksum type: {}", kind),
            }
        }
    }
}

#[svnexus_macro::enum_converter(repr_type=ffi::svn_wc_conflict_kind_t)]
#[derive(uniffi::Enum, Debug)]
pub enum WorkingCopyConflictKind {
    Text = ffi::svn_wc_conflict_kind_t_svn_wc_conflict_kind_text,
    Property = ffi::svn_wc_conflict_kind_t_svn_wc_conflict_kind_property,
    Tree = ffi::svn_wc_conflict_kind_t_svn_wc_conflict_kind_tree,
}

#[svnexus_macro::enum_converter(repr_type=ffi::svn_wc_conflict_kind_t)]
#[derive(uniffi::Enum, Debug)]
pub enum WorkingCopyConflictAction {
    Edit = ffi::svn_wc_conflict_action_t_svn_wc_conflict_action_edit,
    Add = ffi::svn_wc_conflict_action_t_svn_wc_conflict_action_add,
    Delete = ffi::svn_wc_conflict_action_t_svn_wc_conflict_action_delete,
    Replace = ffi::svn_wc_conflict_action_t_svn_wc_conflict_action_replace,
}

#[svnexus_macro::enum_converter(repr_type=ffi::svn_wc_conflict_reason_t)]
#[derive(uniffi::Enum, Debug)]
pub enum WorkingCopyConflictReason {
    Edited = ffi::svn_wc_conflict_reason_t_svn_wc_conflict_reason_edited,
    Obstructed = ffi::svn_wc_conflict_reason_t_svn_wc_conflict_reason_obstructed,
    Deleted = ffi::svn_wc_conflict_reason_t_svn_wc_conflict_reason_deleted,
    Missing = ffi::svn_wc_conflict_reason_t_svn_wc_conflict_reason_missing,
    Unversioned = ffi::svn_wc_conflict_reason_t_svn_wc_conflict_reason_unversioned,
    Added = ffi::svn_wc_conflict_reason_t_svn_wc_conflict_reason_added,
    Replaced = ffi::svn_wc_conflict_reason_t_svn_wc_conflict_reason_replaced,
    MovedAway = ffi::svn_wc_conflict_reason_t_svn_wc_conflict_reason_moved_away,
    MovedHere = ffi::svn_wc_conflict_reason_t_svn_wc_conflict_reason_moved_here,
}

#[svnexus_macro::enum_converter(repr_type=ffi::svn_wc_operation_t)]
#[derive(uniffi::Enum, Debug)]
pub enum WorkingCopyOperation {
    None = ffi::svn_wc_operation_t_svn_wc_operation_none,
    Update = ffi::svn_wc_operation_t_svn_wc_operation_update,
    Switch = ffi::svn_wc_operation_t_svn_wc_operation_switch,
    Merge = ffi::svn_wc_operation_t_svn_wc_operation_merge,
}

#[derive(uniffi::Record, Debug, new)]
pub struct WorkingCopyInfo {
    schedule: WorkingCopySchedule,
    copy_from_revision: Option<RevisionNumber>,
    check_sum: CheckSum,
    changelist: String,
    depth: Depth,
    recorded_size: Option<u64>,
    recorded_time: i64,
    working_copy_absolute_path: String,
    moved_from_absolute_path: Option<String>,
    moved_to_absolute_path: Option<String>,
    working_copy_format: i32,
    store_pristine: bool,
}

#[easy_ext::ext]
impl *const ffi::svn_wc_info_t {
    fn to_working_copy_info(self) -> WorkingCopyInfo {
        unsafe {
            let ptr = self.as_ref().unwrap();
            let schedule = ptr.schedule.try_into().unwrap();
            let copy_from_revision = ptr.copyfrom_rev.into_optional_revision();
            let check_sum = ptr.checksum.to_check_sum();
            let changelist = ptr.changelist.to_str().to_string();
            let depth = ptr.depth.try_into().unwrap();
            let recorded_size = if ptr.recorded_size < 0 {
                None
            } else {
                Some(ptr.recorded_size.try_into().unwrap())
            };

            let recorded_time = ptr.recorded_time.try_into().unwrap();

            let working_copy_absolute_path = ptr.wcroot_abspath.to_str().to_string();

            let moved_from_absolute_path = ptr
                .moved_from_abspath
                .to_nullable_str()
                .map(|v| v.to_string());

            let moved_to_absolute_path = ptr
                .moved_to_abspath
                .to_nullable_str()
                .map(|v| v.to_string());

            let working_copy_format = ptr.wc_format.try_into().unwrap();

            let store_pristine = ptr.store_pristine != 0;

            WorkingCopyInfo {
                schedule,
                copy_from_revision,
                check_sum,
                changelist,
                depth,
                recorded_size,
                recorded_time,
                working_copy_absolute_path,
                moved_from_absolute_path,
                moved_to_absolute_path,
                working_copy_format,
                store_pristine,
            }
        }
    }
}

#[svnexus_macro::enum_converter(repr_type=ffi::svn_wc_schedule_t)]
#[derive(uniffi::Enum, Debug)]
pub enum WorkingCopySchedule {
    Normal = ffi::svn_wc_schedule_t_svn_wc_schedule_normal,
    Add = ffi::svn_wc_schedule_t_svn_wc_schedule_add,
    Delete = ffi::svn_wc_schedule_t_svn_wc_schedule_delete,
    Replace = ffi::svn_wc_schedule_t_svn_wc_schedule_replace,
}

#[derive(new, Debug, uniffi::Record)]
pub struct WorkingCopyConflictDescription {
    local_absolute_path: String,
    node_kind: NodeKind,
    kind: WorkingCopyConflictKind,
    property_name: Option<String>,
    is_binary: bool,
    mime_type: Option<String>,
    action: WorkingCopyConflictAction,
    base_absolute_path: Option<String>,
    their_absolute_path: String,
    my_absolute_path: String,
    merged_file: Option<String>,
    src_left_version: WorkingCopyConflictVersion,
    src_right_version: WorkingCopyConflictVersion,
    property_value_base: Option<Vec<u8>>,
    property_value_working: Option<Vec<u8>>,
    property_value_incoming_old: Option<Vec<u8>>,
    property_value_incoming_new: Option<Vec<u8>>,
}

impl WorkingCopyConflictDescription {
    fn from_raw(ptr: *const ffi::svn_wc_conflict_description2_t) -> Self {
        unsafe {
            let ptr = ptr.as_ref().unwrap();

            let local_absolute_path = ptr.local_abspath.to_str().to_string();

            let node_kind = ptr.node_kind.try_into().unwrap();

            let kind = ptr.kind.try_into().unwrap();

            let property_name = ptr.property_name.to_nullable_str().map(|v| v.to_string());

            let is_binary = ptr.is_binary != 0;

            let mime_type = ptr.mime_type.to_nullable_str().map(|v| v.to_string());

            let action = ptr.action.try_into().unwrap();

            let base_absolute_path = ptr.base_abspath.to_nullable_str().map(|v| v.to_string());

            let their_absolute_path = ptr.their_abspath.to_str().to_string();

            let my_absolute_path = ptr.my_abspath.to_str().to_string();

            let merged_file = ptr.merged_file.to_nullable_str().map(|v| v.to_string());

            let src_left_version = ptr.src_left_version.to_working_copy_conflict_version();

            let src_right_version = ptr.src_right_version.to_working_copy_conflict_version();

            let property_value_base = ptr.prop_value_base.to_nullable_slice().map(|v| v.to_vec());

            let property_value_working = ptr
                .prop_value_working
                .to_nullable_slice()
                .map(|v| v.to_vec());

            let property_value_incoming_old = ptr
                .prop_value_incoming_old
                .to_nullable_slice()
                .map(|v| v.to_vec());

            let property_value_incoming_new = ptr
                .prop_value_incoming_new
                .to_nullable_slice()
                .map(|v| v.to_vec());

            Self {
                local_absolute_path,
                node_kind,
                kind,
                property_name,
                is_binary,
                mime_type,
                action,
                base_absolute_path,
                their_absolute_path,
                my_absolute_path,
                merged_file,
                src_left_version,
                src_right_version,
                property_value_base,
                property_value_working,
                property_value_incoming_new,
                property_value_incoming_old,
            }
        }
    }
}

#[derive(uniffi::Record, Debug, new)]
pub struct WorkingCopyConflictVersion {
    repos_url: String,
    peg_revision: RevisionNumber,
    path_in_repos: String,
    node_kind: NodeKind,
    repos_uuid: String,
}

#[easy_ext::ext]
impl *const ffi::svn_wc_conflict_version_t {
    fn to_working_copy_conflict_version(self) -> WorkingCopyConflictVersion {
        assert!(!self.is_null());

        unsafe {
            let ptr = self.as_ref().unwrap();

            let repos_url = ptr.repos_url.to_str().to_string();

            let peg_revision = ptr.peg_rev.try_into().unwrap();

            let path_in_repos = ptr.path_in_repos.to_str().to_string();

            let node_kind = ptr.node_kind.try_into().unwrap();

            let repos_uuid = ptr.repos_uuid.to_str().to_string();

            WorkingCopyConflictVersion {
                repos_url,
                peg_revision,
                path_in_repos,
                node_kind,
                repos_uuid,
            }
        }
    }
}

#[derive(uniffi::Record, Debug, new)]
pub struct InfoEntry {
    url: String,
    revision: RevisionNumber,
    repos_root_url: String,
    repos_uuid: String,
    kind: NodeKind,
    size: Option<u64>,
    last_changed_revision: RevisionNumber,
    last_changed_date: i64,
    last_changed_author: String,
    lock: Lock,
    working_copy_info: Option<WorkingCopyInfo>,
}

#[easy_ext::ext]
impl *const ffi::svn_client_info2_t {
    fn to_info_entry(self) -> InfoEntry {
        unsafe {
            let ptr = self.as_ref().unwrap();

            let url = ptr.URL.to_str().to_string();

            let revision = ptr.rev.try_into().unwrap();

            let repos_root_url = ptr.repos_root_URL.to_str().to_string();

            let repos_uuid = ptr.repos_UUID.to_str().to_string();

            let kind = ptr.kind.try_into().unwrap();

            let size = if ptr.size < 0 {
                None
            } else {
                Some(ptr.size.try_into().unwrap())
            };

            let last_changed_revision = ptr.last_changed_rev.try_into().unwrap();

            let last_changed_date = ptr.last_changed_date.try_into().unwrap();

            let last_changed_author = ptr.last_changed_author.to_str().to_string();

            let lock = Lock::from_ptr(ptr.lock);

            let working_copy_info = if ptr.wc_info.is_null() {
                None
            } else {
                Some(ptr.wc_info.to_working_copy_info())
            };

            InfoEntry {
                url,
                revision,
                repos_root_url,
                repos_uuid,
                kind,
                size,
                last_changed_revision,
                last_changed_date,
                last_changed_author,
                lock,
                working_copy_info,
            }
        }
    }
}

impl Context {
    fn ctx(&self) -> &ffi::svn_client_ctx_t {
        unsafe { &*self.ptr }
    }

    fn ctx_mut(&mut self) -> &mut ffi::svn_client_ctx_t {
        unsafe { &mut *self.ptr }
    }

    pub fn status(&mut self, opts: StatusOptions) -> error::Result<StatusResult> {
        let mut result_revision: ffi::svn_revnum_t = 0;

        let revision = opts.revision.to_opt_revision();

        // let mut changelist =
        // unsafe { apr::Array::from_string_list(opts.changelist.len(), opts.changelist)? };

        self.inner.status_entries.clear();

        unsafe {
            let mut pool = apr::Pool::create();
            let path = pool.string(opts.path.as_str())?;
            let changelist = pool.string_array(opts.changelist.len(), opts.changelist.iter())?;

            let error = ffi::svn_client_status6(
                &mut result_revision as *mut _,
                self.ptr,
                path,
                &revision as *const _,
                opts.depth.into(),
                opts.get_all.into(),
                opts.check_out_of_date.into(),
                opts.check_working_copy.into(),
                opts.no_ignore.into(),
                opts.ignore_externals.into(),
                opts.depth_as_sticky.into(),
                changelist,
                Some(status_callback),
                &mut *self.inner as *mut ContextInner as *mut c_void,
                pool.as_mut_ptr(),
            );

            SVNError::from_nullable_ptr(error).context(builder::Svn)?;
        };

        let entries = std::mem::take(&mut self.inner.status_entries);

        Ok(StatusResult::new(entries))
    }

    fn take_commit_result(&mut self) -> CommitResult {
        let items = std::mem::take(&mut self.inner.commit_items);

        let info = std::mem::take(&mut self.inner.commit_info).unwrap();

        CommitResult::new(items, info)
    }

    pub fn checkout(&mut self, opts: CheckoutOptions) -> error::Result<RevisionNumber> {
        if opts.url.is_empty() {
            return error::builder::InvalidArgument {
                detail: "Empty url",
            }
            .fail();
        }
        if opts.path.is_empty() {
            return error::builder::InvalidArgument {
                detail: "Empty path",
            }
            .fail();
        }
        let revision = opts.revision.to_opt_revision();
        let peg_revision = opts.peg_revision.to_opt_revision();

        let mut result_revision: ffi::svn_revnum_t = 0;

        let mut pool = unsafe { apr::Pool::create() };

        let url = unsafe { pool.string(opts.url.as_str()) }?;

        let path = unsafe { pool.string(opts.path.as_str()) }?;

        // let url = CString::new(opts.url)
        //     .ok()
        //     .context(builder::InvalidArgument {
        //         detail: "Invalid url",
        //     })?;

        // let path = CString::new(opts.path).whatever_context::<_, error::Error>("Invalid path")?;

        let store_pristine = opts
            .store_pristine
            .and_then(|v| {
                Some(if v {
                    ffi::svn_tristate_t_svn_tristate_true
                } else {
                    ffi::svn_tristate_t_svn_tristate_false
                })
            })
            .unwrap_or(ffi::svn_tristate_t_svn_tristate_unknown);

        tracing::info!("Start checking using c function");
        let error = unsafe {
            if ffi::svn_path_is_url(url) == 0 {
                return builder::InvalidArgument {
                    detail: "Invalid url",
                }
                .fail();
            }
            if ffi::svn_uri_is_canonical(url, pool.as_mut_ptr()) == 0 {
                return builder::InvalidArgument {
                    detail: "Invalid url",
                }
                .fail();
            }

            // if ffi::svn_dirent_is_absolute(path) == 0 {
            //     return builder::InvalidArgument {
            //         detail: "Invalid local path",
            //     }
            //     .fail();
            // }

            ffi::svn_client_checkout4(
                &mut result_revision as *mut _,
                url,
                path,
                &revision as *const _,
                &peg_revision as *const _,
                opts.depth.into(),
                opts.ignore_externals.into(),
                opts.allow_unversioned_obstructions.into(),
                std::ptr::null(),
                store_pristine,
                self.ptr,
                pool.as_mut_ptr(),
            )
        };
        tracing::info!("Finish checking using c function");

        SVNError::from_nullable_ptr(error).context(builder::Svn)?;

        Ok(result_revision.try_into().unwrap())
    }

    pub fn add(&mut self, opts: &AddOptions) -> error::Result<()> {
        // let path = CString::from_str(opts.path.as_str())
        //     .whatever_context::<_, error::Error>("Invalid path")?;
        //
        unsafe {
            let mut pool = apr::Pool::create();

            let path = pool.string(opts.path.as_str())?;

            let error = ffi::svn_client_add5(
                path,
                opts.depth.into(),
                opts.force.into(),
                opts.no_ignore.into(),
                opts.no_auto_properties.into(),
                opts.add_parents.into(),
                self.ctx_mut(),
                pool.as_mut_ptr(),
            );
            SVNError::from_nullable_ptr(error).context(builder::Svn)
        }
    }

    pub fn commit(&mut self, opts: CommitOptions) -> error::Result<CommitResult> {
        let result = unsafe {
            if opts.commit_message.as_bytes().contains(&b'\0') {
                return builder::InvalidArgument {
                    detail: "Invalid commit message",
                }
                .fail();
            }
            let mut pool = apr::Pool::create();
            // let mut targets = Vec::with_capacity(opts.targets.len());
            // let mut targets = apr::Array::with_capacity(opts.targets.len());
            // for i in opts.targets {
            //     let target = CString::new(i.clone())
            //         .ok()
            //         .context(builder::InvalidArgument {
            //             detail: format!("Invalid target: {}", i),
            //         })?;
            //     targets.push(target.as_c_str().as_ptr());
            // }
            let targets = pool.string_array(opts.targets.len(), opts.targets.iter())?;

            // let mut changelists = apr::Array::with_capacity(opts.changelists.len());
            //
            // for i in opts.changelists {
            //     let target = CString::new(i.clone())
            //         .ok()
            //         .context(builder::InvalidArgument {
            //             detail: format!("Invalid changelists: {}", i),
            //         })?;
            //     changelists.push(target.as_c_str().as_ptr());
            // }

            let changelists = pool.string_array(opts.changelists.len(), opts.changelists.iter())?;

            // let revision_properties_table = ffi::apr_hash_make(pool.ptr);
            //
            // for (k, v) in opts.revision_properties_table {
            //     let key = CString::new(k).ok().context(builder::InvalidArgument {
            //         detail: format!("Invalid revprop table key: {}", v),
            //     })?;
            //
            //     let value = CString::new(v.clone())
            //         .ok()
            //         .context(builder::InvalidArgument {
            //             detail: format!("Invalid revprop table value: {}", v),
            //         })?;
            //
            //     ffi::apr_hash_set(
            //         revision_properties_table,
            //         key.as_c_str().as_ptr() as _,
            //         ffi::APR_HASH_KEY_STRING.try_into().unwrap(),
            //         value.as_c_str().as_ptr() as _,
            //     );
            // }

            let revision_properties_table =
                pool.string_hash_map(opts.revision_property_table.iter())?;

            self.inner.commit_items.clear();
            self.inner.commit_message = opts.commit_message;

            let error = ffi::svn_client_commit6(
                targets,
                opts.depth.into(),
                opts.keep_locks.into(),
                opts.keep_changelist.into(),
                opts.commit_as_operations.into(),
                opts.include_file_externals.into(),
                opts.include_dir_externals.into(),
                changelists,
                revision_properties_table,
                Some(commit_callback),
                &mut *self.inner as *mut _ as *mut c_void,
                self.ctx_mut(),
                pool.as_mut_ptr(),
            );

            // *self.inner.cancel.lock() = None;

            self.inner.commit_message.clear();

            SVNError::from_nullable_ptr(error).context(builder::Svn)?;

            self.take_commit_result()
        };

        Ok(result)
    }

    pub fn delete(&mut self, opts: DeleteOptions) -> error::Result<DeleteResult> {
        unsafe {
            let mut pool = apr::Pool::create();

            let path = pool.string_array(opts.path.len(), opts.path.iter())?;

            let table = pool.string_hash_map(opts.revprop_table.iter())?;

            let error = ffi::svn_client_delete4(
                path,
                opts.force.into(),
                opts.keep_local.into(),
                table,
                Some(commit_callback),
                &mut *self.inner as *mut _ as *mut c_void,
                self.ctx_mut(),
                pool.as_mut_ptr(),
            );

            SVNError::from_nullable_ptr(error).context(builder::Svn)?;
        }

        Ok(DeleteResult::new(std::mem::take(
            &mut self.inner.commit_info,
        )))
    }

    pub fn revert(&mut self, opts: RevertOptions) -> error::Result<()> {
        unsafe {
            let mut pool = apr::Pool::create();
            let error = ffi::svn_client_revert4(
                pool.string_array(opts.paths.len(), opts.paths.iter())?,
                opts.depth.into(),
                pool.string_array(opts.changelists.len(), opts.changelists.iter())?,
                opts.clear_changelists.into(),
                opts.metadata_only.into(),
                opts.added_keep_local.into(),
                self.ctx_mut(),
                pool.as_mut_ptr(),
            );
            SVNError::from_nullable_ptr(error).context(builder::Svn)?;
        }

        Ok(())
    }

    pub fn log(&mut self, opts: LogOptions) -> error::Result<LogResult> {
        unsafe {
            let mut pool = apr::Pool::create();
            let targets = pool.string_array(opts.targets.len(), opts.targets.iter())?;

            let peg_revision = opts.peg_revision.to_opt_revision();

            self.inner.log_entries.clear();

            let error = ffi::svn_client_log5(
                targets,
                &peg_revision as *const _,
                pool.revision_range(&opts.revsions),
                opts.limit.try_into().unwrap_or(std::ffi::c_int::MAX),
                opts.discover_changed_paths.into(),
                opts.strict_node_history.into(),
                opts.include_merged_revisions.into(),
                pool.string_array(
                    opts.revisions_properties.len(),
                    opts.revisions_properties.iter(),
                )?,
                Some(on_receive_log_entry),
                &mut (*self.inner) as *mut _ as *mut c_void,
                self.ctx_mut(),
                pool.as_mut_ptr(),
            );

            SVNError::from_nullable_ptr(error).context(builder::Svn)?;
        }
        Ok(LogResult::new(std::mem::take(&mut self.inner.log_entries)))
    }

    pub fn import(&mut self, opts: ImportOptions) -> error::Result<ImportResult> {
        unsafe {
            let mut pool = apr::Pool::create();
            if opts.commit_message.as_bytes().contains(&b'\0') {
                return builder::General {
                    detail: "Invalid commit message",
                }
                .fail();
            }
            self.inner.commit_message = opts.commit_message;
            let error = ffi::svn_client_import5(
                pool.string(opts.path)?,
                pool.string(opts.url)?,
                opts.depth.into(),
                opts.no_ignore.into(),
                opts.no_autoprops.into(),
                opts.ignore_unknown_node_types.into(),
                pool.string_hash_map(opts.revision_property_table.iter())?,
                Some(on_import_filter),
                &mut *self.inner as *mut _ as *mut c_void,
                Some(commit_callback),
                &mut *self.inner as *mut _ as *mut c_void,
                self.ctx_mut(),
                pool.as_mut_ptr(),
            );

            SVNError::from_nullable_ptr(error).context(builder::Svn)?;
        }
        Ok(ImportResult::new(
            std::mem::take(&mut self.inner.commit_items),
            std::mem::take(&mut self.inner.commit_info).unwrap_or_default(),
        ))
    }

    pub fn cat(&mut self, opts: CatOptions) -> error::Result<CatResult> {
        unsafe {
            let mut properties = std::ptr::null_mut();

            let mut pool = apr::Pool::create();

            let mut stream = super::stream::Stream::create(Default::default());

            let path = pool.string(opts.path)?;

            let peg_revision = opts.peg_revision.to_opt_revision();

            let revision = opts.revision.to_opt_revision();

            println!("Call cat3");
            let error = ffi::svn_client_cat3(
                &mut properties as *mut _,
                stream.stream(),
                path,
                &peg_revision as *const _,
                &revision as *const _,
                opts.expand_keywords.into(),
                self.ptr,
                pool.as_mut_ptr(),
                pool.as_mut_ptr(),
            );
            println!("Call finished cat3");

            SVNError::from_nullable_ptr(error).context(builder::Svn)?;
            let content = stream.take_write_buffer();

            let properties = pool.apr_hash_to_hash_map(properties);

            Ok(CatResult {
                content,
                properties,
            })
        }
    }

    pub fn info(&mut self, opts: InfoOptions) -> error::Result<InfoResult> {
        unsafe {
            let mut pool = apr::Pool::create();

            let path = pool.string(opts.path.as_str())?;

            let peg_revision = opts.peg_revision.to_opt_revision();

            let revision = opts.revision.to_opt_revision();

            let depth = opts.depth.into();

            let fetch_excluded = opts.fetch_excluded.into();

            let fetch_actual_only = opts.fetch_actual_only.into();

            let include_externals = opts.include_externals.into();

            let changelists = pool.string_array(opts.changelists.len(), opts.changelists.iter())?;

            let error = ffi::svn_client_info4(
                path,
                peg_revision.pointer(),
                revision.pointer(),
                depth,
                fetch_excluded,
                fetch_actual_only,
                include_externals,
                changelists,
                Some(info_receiver),
                self.inner.inner_void_pointer_mut(),
                self.ctx_mut(),
                pool.as_mut_ptr(),
            );

            SVNError::from_nullable_ptr(error).context(builder::Svn)?;
        }

        let entries = std::mem::take(&mut self.inner.info_entries);

        Ok(InfoResult { entries })
    }

    fn create(opts: CreateContextOptions) -> error::Result<Self> {
        let mut pool = unsafe { apr::Pool::create() };

        let mut ptr: *mut ffi::svn_client_ctx_t = std::ptr::null_mut();

        unsafe {
            let error = ffi::svn_client_create_context2(
                &mut ptr as *mut _,
                std::ptr::null_mut(),
                pool.as_mut_ptr(),
            );
            SVNError::from_nullable_ptr(error).context(builder::Svn)?;
        }

        assert!(!ptr.is_null());

        let mut inner = Box::new(ContextInner::new(opts.context_notifier));

        // if let Some(on_notify) = opts.on_notify {
        //     inner.on_notify = on_notify;
        // }
        // if let Some(func) = opts.on_may_save_password_as_plain_text {
        //     inner.on_may_save_password_as_plain_text = func;
        // }
        // if let Some(func) = opts.on_ssl_server_trust_prompt {
        //     inner.on_ssl_server_trust_prompt = func;
        // }
        // if let Some(func) = opts.on_progress_notify {
        //     inner.on_progress_notify = func;
        // }
        // if let Some(func) = opts.on_cancel {
        //     inner.on_cancel = func;
        // }
        // if let Some(func) = opts.on_authenticate {
        //     inner.on_authenticate = func;
        // }

        let array = unsafe {
            ffi::apr_array_make(
                pool.as_mut_ptr(),
                11,
                size_of::<usize>().try_into().unwrap(),
            )
        };

        let mut provider = std::ptr::null_mut();

        #[cfg(target_os = "windows")]
        {
            unsafe {
                let error = ffi::svn_auth_get_platform_specific_provider(
                    &mut provider as *mut _,
                    pool.string("windows")?,
                    pool.string("simple")?,
                    pool.as_mut_ptr(),
                );
                SVNError::from_error_ptr(error).context(builder::Svn)?;
                let ptr = ffi::apr_array_push(array) as *mut *mut ffi::svn_auth_provider_object_t;
                *ptr = provider;
            };
        }

        unsafe {
            ffi::svn_auth_get_username_provider(&mut provider as *mut _, pool.as_mut_ptr());
            let ptr = ffi::apr_array_push(array) as *mut *mut ffi::svn_auth_provider_object_t;
            *ptr = provider;

            ffi::svn_auth_get_simple_provider2(
                &mut provider as *mut _,
                Some(may_save_password_as_plain_text),
                &mut *inner as *mut _ as *mut c_void,
                pool.as_mut_ptr(),
            );
            let ptr = ffi::apr_array_push(array) as *mut *mut ffi::svn_auth_provider_object_t;
            *ptr = provider;

            // ssl
            ffi::svn_auth_get_ssl_server_trust_file_provider(
                &mut provider as *mut _,
                pool.as_mut_ptr(),
            );
            let ptr = ffi::apr_array_push(array) as *mut *mut ffi::svn_auth_provider_object_t;
            *ptr = provider;

            ffi::svn_auth_get_ssl_client_cert_file_provider(
                &mut provider as *mut _,
                pool.as_mut_ptr(),
            );
            let ptr = ffi::apr_array_push(array) as *mut *mut ffi::svn_auth_provider_object_t;
            *ptr = provider;

            ffi::svn_auth_get_ssl_client_cert_pw_file_provider2(
                &mut provider as *mut _,
                Some(may_save_password_as_plain_text),
                &mut *inner as *mut _ as *mut c_void,
                pool.as_mut_ptr(),
            );
            let ptr = ffi::apr_array_push(array) as *mut *mut ffi::svn_auth_provider_object_t;
            *ptr = provider;

            ffi::svn_auth_get_ssl_server_trust_prompt_provider(
                &mut provider as *mut _,
                Some(ssl_server_trust_prompt),
                &mut *inner as *mut _ as *mut c_void,
                pool.as_mut_ptr(),
            );
            let ptr = ffi::apr_array_push(array) as *mut *mut ffi::svn_auth_provider_object_t;
            *ptr = provider;

            ffi::svn_auth_get_simple_prompt_provider(
                &mut provider as *mut _,
                Some(on_authenticate),
                &mut *inner as *mut _ as *mut c_void,
                -1,
                pool.as_mut_ptr(),
            );
        }

        let mut auth_baton = std::ptr::null_mut();
        unsafe {
            ffi::svn_auth_open(&mut auth_baton as *mut _, array, pool.as_mut_ptr());
            if let (Some(username), Some(password)) = (opts.default_username, opts.default_password)
            {
                ffi::svn_auth_set_parameter(
                    auth_baton,
                    ffi::SVN_AUTH_PARAM_DEFAULT_USERNAME.as_ptr() as _,
                    pool.string(username)? as *const c_void,
                );

                ffi::svn_auth_set_parameter(
                    auth_baton,
                    ffi::SVN_AUTH_PARAM_DEFAULT_PASSWORD.as_ptr() as _,
                    pool.string(password)? as *const c_void,
                );
            }
        }

        unsafe {
            let ctx = ptr.as_mut().unwrap();
            ctx.auth_baton = auth_baton;
            ctx.log_msg_func3 = Some(on_get_commit_message);
            ctx.log_msg_baton3 = &mut *inner as *mut _ as *mut c_void;
            ctx.notify_func2 = Some(on_notify);
            ctx.notify_baton2 = &mut *inner as *mut _ as *mut c_void;
            ctx.cancel_func = Some(on_cancel);
            ctx.cancel_baton = &mut *inner as *mut _ as *mut c_void;

            ctx.progress_func = Some(on_progress_notify);
            ctx.progress_baton = (&mut *inner) as *mut _ as *mut c_void;

            if let Some(ref name) = opts.name {
                ctx.client_name = pool.string(name)?;
            }
        }

        Ok(Self { ptr, pool, inner })
    }
}
