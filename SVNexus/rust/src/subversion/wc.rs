use derive_new::new;
use snafu::ResultExt;

use std::collections::HashMap;
use std::sync::Arc;
use super::context;
use super::ffi;
use crate::apr;
use crate::error;
use crate::error::builder;
use crate::utils::Pointer;
use crate::utils::{CStringer, SubversionStringer, PointerMapper};
use super::SVNError;
use super::version::Version;

pub struct WorkingCopyContext {
    ptr: *mut ffi::svn_wc_context_t,
    pool: apr::Pool,
}
impl Drop for WorkingCopyContext {
    fn drop(&mut self) {
        unsafe {
            let error = ffi::svn_wc_context_destroy(self.ptr);
            if !error.is_null() {
                tracing::info!("Failed to destroy wc context: {:?}", SVNError::from_nullable_ptr(error))
            }

        }
    }
}
unsafe impl Send for WorkingCopyContext {}

#[derive(uniffi::Record, Debug)]
pub struct WorkingCopyCreateContextOptions {

}

impl WorkingCopyContext {
    fn create(_: WorkingCopyCreateContextOptions) -> error::Result<Self> {
        unsafe {
            let mut pool = apr::Pool::create();

            let mut ptr: *mut ffi::svn_wc_context_t = std::ptr::null_mut();
            let error = ffi::svn_wc_context_create(ptr.pointer_mut(), std::ptr::null_mut(), pool.as_mut_ptr(), pool.as_mut_ptr());
            SVNError::from_nullable_ptr(error).context(builder::Svn)?;

            assert!(!ptr.is_null(), "wc context ptr should not be null");

            Ok(Self {
                ptr,
                pool
            })


        }
    }
}

#[derive(uniffi::Record, Debug)]
pub struct CheckRootResult {
    is_wc_root: bool,
    is_switched: bool,
    kind: context::NodeKind,
}

pub struct WorkingCopyWalkStatusOptions {
    local_absolute_path: String,
    depth: context::Depth,
    get_all: bool,
    no_ignore: bool,
    ignore_text_modified: bool,

}


#[derive(uniffi::Object, Clone)]
pub enum AsyncWorkingCopyContext {
    Context(Arc<parking_lot::Mutex<context::Context>>),
    Raw(Arc<parking_lot::Mutex<WorkingCopyContext>>)
}

#[uniffi::export(async_runtime = "tokio")]
impl AsyncWorkingCopyContext {

    #[uniffi::constructor]
    pub fn create(opts: WorkingCopyCreateContextOptions) -> error::Result<Self> {
        Ok(Self::Raw(Arc::new(parking_lot::Mutex::new(WorkingCopyContext::create(opts)?))))
    }

    pub async fn check_root(&self, path: String) -> error::Result<CheckRootResult> {
        self.call_async(move |ctx| unsafe {
            let mut pool = apr::Pool::create();
            let path = pool.string(path)?;

            let mut is_wc_root: ffi::svn_boolean_t = 0;
            let mut is_switched: ffi::svn_boolean_t = 0;
            let mut kind: ffi::svn_node_kind_t = ffi::svn_node_kind_t_svn_node_unknown;
            let error = ffi::svn_wc_check_root(is_wc_root.pointer_mut(), is_switched.pointer_mut(), kind.pointer_mut(), ctx, path, pool.as_mut_ptr());
            SVNError::from_nullable_ptr(error).context(builder::Svn)?;

            let is_wc_root = is_wc_root != 0;
            let is_switched = is_switched != 0;
            let kind = context::NodeKind::try_from(kind).unwrap();
            Ok(CheckRootResult { is_wc_root, is_switched, kind })
        }).await
    }

    pub async fn wc_version(&self, path: String) -> error::Result<Version> {
        self.call_async(move |ctx| unsafe {
            let mut pool = apr::Pool::create();
            let path = pool.string(path)?;

            let mut format: std::ffi::c_int = 0;

            let error = ffi::svn_wc_check_wc2(format.pointer_mut(), ctx, path, pool.as_mut_ptr());

            SVNError::from_nullable_ptr(error).context(builder::Svn)?;

            let version = ffi::svn_client_wc_version_from_format(format, pool.as_mut_ptr());

            Ok(Version::from(version))

            // Ok(version)
        }).await
    }
}

impl AsyncWorkingCopyContext {
    fn call<T>(&self, f: impl FnOnce(*mut ffi::svn_wc_context_t) -> error::Result<T>) -> error::Result<T> {
        match self {
            AsyncWorkingCopyContext::Context(mutex) => {
                let mut lock = mutex.lock();
                let ctx = unsafe { lock.ctx().as_mut().unwrap().wc_ctx };
                f(ctx)
            },
            AsyncWorkingCopyContext::Raw(mutex) => {
                let lock = mutex.lock();
                f(lock.ptr)
            },
        }
    }

    async fn call_async<F, R>(&self, call: F) -> error::Result<R>
    where
        F: (FnOnce(*mut ffi::svn_wc_context_t)-> error::Result<R>) + Send + 'static,
        R: Send + 'static,
    {
        let context = self.clone();
        let result = tokio::task::spawn_blocking(move || {
            context.call(call)
        })
        .await
        .context(builder::Runtime)??;

        Ok(result)
    }

}

#[svnexus_macro::enum_converter(repr_type=ffi::svn_wc_conflict_kind_t)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, uniffi::Enum)]
pub enum WorkingCopyConflictKind {
    Text = ffi::svn_wc_conflict_kind_t_svn_wc_conflict_kind_text,
    Property = ffi::svn_wc_conflict_kind_t_svn_wc_conflict_kind_property,
    Tree = ffi::svn_wc_conflict_kind_t_svn_wc_conflict_kind_tree,
}

#[svnexus_macro::enum_converter(repr_type=ffi::svn_wc_conflict_action_t)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, uniffi::Enum)]
pub enum WorkingCopyConflictAction {
    Add = ffi::svn_wc_conflict_action_t_svn_wc_conflict_action_add,
    Edit = ffi::svn_wc_conflict_action_t_svn_wc_conflict_action_edit,
    Delete = ffi::svn_wc_conflict_action_t_svn_wc_conflict_action_delete,
    Replace = ffi::svn_wc_conflict_action_t_svn_wc_conflict_action_replace,
}

#[svnexus_macro::enum_converter(repr_type=ffi::svn_wc_conflict_reason_t)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, uniffi::Enum)]
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
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, uniffi::Enum)]
pub enum WorkingCopyOperation {
    None = ffi::svn_wc_operation_t_svn_wc_operation_none,
    Update = ffi::svn_wc_operation_t_svn_wc_operation_update,
    Switch = ffi::svn_wc_operation_t_svn_wc_operation_switch,
    Merge = ffi::svn_wc_operation_t_svn_wc_operation_merge,
}

#[derive(uniffi::Record)]
pub struct WorkingCopyConflictVersion {
    repository_url: String,
    peg_revision: context::RevisionNumber,
    path_in_repository: String,
    node_kind: context::NodeKind,
    repository_uuid: Option<String>,
}

impl From<*const ffi::svn_wc_conflict_version_t> for WorkingCopyConflictVersion {
    fn from(value: *const ffi::svn_wc_conflict_version_t) -> WorkingCopyConflictVersion {
        unsafe {
            let this = value.as_ref().unwrap();
            let repository_url = this.repos_url.to_str().to_string();
            let peg_revision = this.peg_rev.try_into().expect("Unexpected revision");
            let path_in_repository = this.path_in_repos.to_str().to_string();
            let node_kind = this.node_kind.try_into().expect("Unexpected node kind");
            let repository_uuid = this.repos_uuid.to_nullable_string();
            WorkingCopyConflictVersion {
                repository_url,
                peg_revision,
                path_in_repository,
                node_kind,
                repository_uuid,
            }
        }
    }
}

#[derive(uniffi::Record)]
pub struct WorkingCopyConflictDescription {
    local_absolute_path: String,
    node_kind: context::NodeKind,
    kind: WorkingCopyConflictKind,
    property_name: Option<String>,
    is_binary: bool,
    mime_type: Option<String>,
    action: WorkingCopyConflictAction,
    reason: WorkingCopyConflictReason,
    base_absolute_path: Option<String>,
    their_absolute_path: Option<String>,
    my_absolute_path: Option<String>,
    merged_file: Option<String>,
    operation: WorkingCopyOperation,
    source_left_version: WorkingCopyConflictVersion,
    source_right_version: WorkingCopyConflictVersion,
    property_reject_absolute_path: Option<String>,
    property_value_base: Option<String>,
    property_value_working: Option<String>,
    property_value_incoming_old: Option<String>,
    property_value_incoming_new: Option<String>,
}

impl From<*const ffi::svn_wc_conflict_description2_t> for WorkingCopyConflictDescription {
    fn from(value: *const ffi::svn_wc_conflict_description2_t) -> Self {
        unsafe {
            let this = value.as_ref().unwrap();
            let local_absolute_path = this.local_abspath.to_str().to_string();
            let node_kind =
                context::NodeKind::try_from(this.node_kind).expect("Unexpected node kind");
            let kind =
                WorkingCopyConflictKind::try_from(this.kind).expect("Unexpected conflict kind");
            let property_name = this.property_name.to_nullable_string();
            let is_binary = this.is_binary != 0;
            let mime_type = this.mime_type.to_nullable_string();
            let action = WorkingCopyConflictAction::try_from(this.action)
                .expect("Unexpected conflict action");
            let reason = WorkingCopyConflictReason::try_from(this.reason)
                .expect("Unexpected conflict reason");
            let base_absolute_path = this.base_abspath.to_nullable_string();
            let their_absolute_path = this.their_abspath.to_nullable_string();
            let my_absolute_path = this.my_abspath.to_nullable_string();
            let merged_file = this.merged_file.to_nullable_string();
            let operation =
                WorkingCopyOperation::try_from(this.operation).expect("Unexpected operation");
            let source_left_version = WorkingCopyConflictVersion::from(this.src_left_version);
            let source_right_version = WorkingCopyConflictVersion::from(this.src_right_version);
            let property_reject_absolute_path = this.prop_reject_abspath.to_nullable_string();
            let property_value_base = this.prop_value_base.to_nullable_string();
            let property_value_working = this.prop_value_working.to_nullable_string();
            let property_value_incoming_old = this.prop_value_incoming_old.to_nullable_string();
            let property_value_incoming_new = this.prop_value_incoming_new.to_nullable_string();

            WorkingCopyConflictDescription {
                local_absolute_path,
                node_kind,
                kind,
                property_name,
                is_binary,
                mime_type,
                action,
                reason,
                base_absolute_path,
                their_absolute_path,
                my_absolute_path,
                merged_file,
                operation,
                source_left_version,
                source_right_version,
                property_reject_absolute_path,
                property_value_base,
                property_value_working,
                property_value_incoming_old,
                property_value_incoming_new,
            }
        }
    }
}


#[svnexus_macro::enum_converter(repr_type=ffi::svn_wc_conflict_choice_t)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, uniffi::Enum)]
pub enum WorkingCopyConflictChoice
{
    Undefined = ffi::svn_wc_conflict_choice_t_svn_wc_conflict_choose_undefined,
    Postpone = ffi::svn_wc_conflict_choice_t_svn_wc_conflict_choose_postpone,
    Base = ffi::svn_wc_conflict_choice_t_svn_wc_conflict_choose_base,
    TheirsFull = ffi::svn_wc_conflict_choice_t_svn_wc_conflict_choose_theirs_full,
    MineFull = ffi::svn_wc_conflict_choice_t_svn_wc_conflict_choose_mine_full,
    TheirsConflict = ffi::svn_wc_conflict_choice_t_svn_wc_conflict_choose_theirs_conflict,
    MineConflict = ffi::svn_wc_conflict_choice_t_svn_wc_conflict_choose_mine_conflict,
    Merged = ffi::svn_wc_conflict_choice_t_svn_wc_conflict_choose_merged,
    Unspecified = ffi::svn_wc_conflict_choice_t_svn_wc_conflict_choose_unspecified,
}


#[derive(uniffi::Record)]
pub struct WorkingCopyConflictResult {
    pub choise: WorkingCopyConflictChoice,
    pub merged_file: Option<String>,
    pub save_merged: bool,
    pub merged_value: Option<String>
}


#[derive(uniffi::Record, Debug, new)]
pub struct WorkingCopyInfo {
    schedule: WorkingCopySchedule,
    copy_from_revision: Option<context::RevisionNumber>,
    check_sum: CheckSum,
    changelist: String,
    depth: context::Depth,
    recorded_size: Option<u64>,
    recorded_time: i64,
    working_copy_absolute_path: String,
    moved_from_absolute_path: Option<String>,
    moved_to_absolute_path: Option<String>,
    working_copy_format: i32,
    store_pristine: bool,
}

impl From<*const ffi::svn_wc_info_t> for WorkingCopyInfo {
    fn from(value: *const ffi::svn_wc_info_t) -> WorkingCopyInfo {
        unsafe {
            let ptr = value.as_ref().unwrap();
            let schedule = ptr.schedule.try_into().unwrap();
            let copy_from_revision = ptr.copyfrom_rev.try_into().ok();
            let check_sum = CheckSum::from(ptr.checksum);
            let changelist = ptr.changelist.to_str().to_string();
            let depth = ptr.depth.try_into().unwrap();
            let recorded_size = ptr.recorded_size.try_into().ok();

            let recorded_time = ptr.recorded_time.try_into().unwrap();

            let working_copy_absolute_path = ptr.wcroot_abspath.to_str().to_string();

            let moved_from_absolute_path = ptr
                .moved_from_abspath
                .to_nullable_string();

            let moved_to_absolute_path = ptr
                .moved_to_abspath
                .to_nullable_string()
        ;

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


#[derive(uniffi::Enum, Debug)]
pub enum CheckSum {
    Md5(Vec<u8>),
    Sha1(Vec<u8>),
    Fnv1a32(Vec<u8>),
    Fnv1a32x4(Vec<u8>),
}

impl From<*const ffi::svn_checksum_t> for CheckSum {
    fn from(value: *const ffi::svn_checksum_t) -> CheckSum {
        unsafe {
            let ptr = value.as_ref().unwrap();

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

impl From<*const ffi::svn_merge_range_t> for MergeRange {
    fn from(ptr: *const ffi::svn_merge_range_t) -> Self {
        unsafe {
            let ptr = ptr.as_ref().unwrap();

            Self {
                start: ptr.start.try_into().unwrap(),
                end: ptr.end.try_into().unwrap(),
                inheritable: ptr.inheritable != 0,
            }
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct WorkingCopyNotify {
    pub path: String,
    pub action: WorkingCopyNotifyAction,
    pub kind: context::NodeKind,
    pub mime_type: Option<String>,
    pub lock: Option<context::Lock>,
    pub err: Option<SVNError>,
    pub content_state: WorkingCopyNotifyState,
    pub property_state: WorkingCopyNotifyState,
    pub lock_state: WorkingCopyNotifyLockState,
    pub revision: Option<context::RevisionNumber>,
    pub changelist_name: Option<String>,
    pub merge_range: Option<MergeRange>,
    pub url: Option<String>,
    pub path_prefix: Option<String>,
    pub property_name: Option<String>,
    pub revision_properties: HashMap<String, String>,
    pub old_revision: Option<context::RevisionNumber>,
    pub hunk_original_start: u32,
    pub hunk_original_length: u32,
    pub hunk_modified_start: u32,
    pub hunk_modified_length: u32,
    pub hunk_matched_line: u32,
    pub hunk_fuzz: u32,
}

impl From<*const ffi::svn_wc_notify_t> for WorkingCopyNotify {
    fn from(ptr: *const ffi::svn_wc_notify_t) -> Self {
        unsafe {
            let ptr = ptr.as_ref().unwrap();

            let path = ptr.path.to_str().to_string();

            let action = WorkingCopyNotifyAction::try_from(ptr.action)
                .expect(format!("Invalid notify action: {}", ptr.action).as_str());

            let kind = context::NodeKind::try_from(ptr.kind)
                .expect(format!("Invalid node kind: {}", ptr.kind).as_str());

            let mime_type = ptr.mime_type.to_nullable_string();

            // let lock = if ptr.lock.is_null() {
            //     None
            // } else {
            //     Some(Lock::from_ptr(ptr.lock))
            // };

            let lock = ptr.lock.map(context::Lock::from);

            let err = ptr.err.map(SVNError::from_not_null_ptr);

            // let err = if ptr.err.is_null() {
            //     None
            // } else {
            //     Some(Error::from_not_null_ptr(ptr.err))
            // };

            let content_state =
                WorkingCopyNotifyState::try_from(ptr.content_state).expect("Invalid notify state");

            let property_state =
                WorkingCopyNotifyState::try_from(ptr.prop_state).expect("Invalid notify state");

            let lock_state =
                WorkingCopyNotifyLockState::try_from(ptr.lock_state).expect("Invalid notify state");

            tracing::info!("WorkingCopyNotify revision: {}", ptr.revision);
            let revision = ptr.revision.try_into().ok();

            let changelist_name = ptr.changelist_name.to_nullable_string();

            let merge_range = ptr.merge_range.map(MergeRange::from);

            let url = ptr.url.to_nullable_string();

            let path_prefix = ptr.path_prefix.to_nullable_string();

            let property_name = ptr.prop_name.to_nullable_string();

            let revision_properties = HashMap::default();

            let old_revision = ptr.old_revision.try_into().ok();

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
    CommitAdded = ffi::svn_wc_notify_action_t_svn_wc_notify_commit_added,
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
