use super::ffi;
use super::stream::Stream;
use super::wc::*;
use super::{SVNError, svn_no_error};
use crate::apr::{self, AprArray, AprPool, AutoPool, Pool};
use crate::error::{self, CSharpError, CSharpErrorExtension, builder};
use crate::extensions::{CommonExtension, OptionExtension};
use crate::subversion::version::Version;
use crate::subversion::{ra, utils};
use crate::utils::PointerMutMapper;
use crate::utils::SubversionStringer;
use crate::utils::{Boxed, CStringer};
use crate::utils::{Pointer, PointerMapper};
use core::panic;
use derive_new::new;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;
use std::collections::HashMap;
use std::ffi::{CStr, c_char, c_void};
use std::sync::Arc;
use strum::EnumString;

pub type RevisionNumber = u32;

#[uniffi::export(with_foreign)]
pub trait ContextNotifier: Send + Sync + 'static {
    fn may_save_password_as_plain_text(&self, realm_string: String) -> Result<bool, CSharpError>;
    fn working_copy_notify(&self, notify: WorkingCopyNotify) -> Result<(), CSharpError>;
    fn ssl_server_trust_prompt(
        &self,
        realm: String,
        failures: u32,
        info: SslServerCertInfo,
        may_save: bool,
    ) -> Result<Option<TrustServer>, CSharpError>;
    fn cancel(&self) -> Result<Option<String>, CSharpError>;
    fn progress_notify(&self, pos: i64, total: i64) -> Result<(), CSharpError>;
    fn authenticate(
        &self,
        realm: String,
        username: String,
        may_save: bool,
    ) -> Result<Option<Authentication>, CSharpError>;
    fn conflict(
        &self,
        description: WorkingCopyConflictDescription,
    ) -> Result<WorkingCopyConflictResult, CSharpError>;
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
    status_receiver: Option<Arc<dyn StatusReceiver>>,

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
    //
    #[new(default)]
    current_list_options: Option<ListOptions>,

    #[new(default)]
    list_entries: Vec<ListEntry>,

    // revision_statck: Vec<i32>,
    #[new(default)]
    log_receiver: Option<Arc<dyn LogReceiver>>,

    #[new(default)]
    log_entries: Vec<LogEntry>,

    #[new(default)]
    info_entries: HashMap<String, InfoEntry>,

    #[new(default)]
    conflicts: Vec<Conflict>,
}

pub struct Context {
    ptr: *mut ffi::svn_client_ctx_t,
    config: *mut apr::ffi::apr_hash_t,
    pool: apr::Pool,
    inner: Box<ContextInner>,
    ra_sessions: HashMap<i32, AutoPool<*mut ffi::svn_ra_session_t>>,
}

impl std::fmt::Debug for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Context")
            .finish()
    }
}

unsafe impl Send for AutoPool<*mut ffi::svn_ra_session_t> {}
unsafe impl Send for Context {}

impl Drop for Context {
    fn drop(&mut self) {
        std::mem::take(&mut self.ra_sessions);
    }
}

impl Context {
    pub fn ra_session(
        &mut self,
        key: i32,
    ) -> error::Result<&mut AutoPool<*mut ffi::svn_ra_session_t>> {
        self.ra_sessions
            .get_mut(&key)
            .any_context("No such ra session")
    }

    pub fn remove_ra_session(&mut self, key: i32) {
        self.ra_sessions.remove(&key);
    }
}

#[derive(uniffi::Record)]
pub struct CreateContextOptions {
    pub name: Option<String>,
    pub default_username: Option<String>,
    pub default_password: Option<String>,
    pub context_notifier: Arc<dyn ContextNotifier>,
    pub config: Config,
}

#[derive(new, uniffi::Record)]
pub struct Config {
    pub proxies: Option<Proxies>,
}

impl Config {
    pub unsafe fn apply(
        &self,
        config: *mut apr::ffi::apr_hash_t,
        pool: &mut apr::Pool,
    ) -> error::Result<()> {
        let servers_config = unsafe {
            apr::ffi::apr_hash_get(
                config,
                ffi::SVN_CONFIG_CATEGORY_SERVERS.as_ptr() as *const i8 as *const c_void,
                ffi::APR_HASH_KEY_STRING.try_into().unwrap(),
            )
        };
        if let Some(proxies) = self.proxies.as_ref() {
            unsafe { proxies.apply(servers_config as *mut _, pool)? };
        }
        Ok(())
    }
}

#[derive(new, uniffi::Record)]
pub struct Proxies {
    pub http: Option<Proxy>,
    pub https: Option<Proxy>,
    pub socks: Option<Proxy>,
}

impl Proxies {
    pub unsafe fn apply(
        &self,
        config: *mut ffi::svn_config_t,
        pool: &mut apr::Pool,
    ) -> error::Result<()> {
        // Implementation details
        //

        if let Some(http) = self.http.as_ref() {
            // Apply HTTP proxy settings
            tracing::info!("set http proxy: {:#?}", http);
            unsafe {
                ffi::svn_config_set(
                    config,
                    ffi::SVN_CONFIG_SECTION_GLOBAL.as_ptr() as _,
                    ffi::SVN_CONFIG_OPTION_HTTP_PROXY_HOST.as_ptr() as _,
                    pool.string(http.host.as_str())?,
                );

                ffi::svn_config_set(
                    config,
                    ffi::SVN_CONFIG_SECTION_GLOBAL.as_ptr() as _,
                    ffi::SVN_CONFIG_OPTION_HTTP_PROXY_PORT.as_ptr() as _,
                    pool.string(http.port.to_string())?,
                );
                if let Some(username) = http.username.as_ref() {
                    ffi::svn_config_set(
                        config,
                        ffi::SVN_CONFIG_SECTION_GLOBAL.as_ptr() as _,
                        ffi::SVN_CONFIG_OPTION_HTTP_PROXY_USERNAME.as_ptr() as _,
                        pool.string(username.as_str())?,
                    );
                }
                if let Some(password) = http.password.as_ref() {
                    ffi::svn_config_set(
                        config,
                        ffi::SVN_CONFIG_SECTION_GLOBAL.as_ptr() as _,
                        ffi::SVN_CONFIG_OPTION_HTTP_PROXY_PASSWORD.as_ptr() as _,
                        pool.string(password.as_str())?,
                    );
                }
            };
        } else if let Some(https) = self.https.as_ref() {
            // Apply HTTPS proxy settings
            tracing::info!("unsupported https proxy now");
        } else if let Some(socks) = self.socks.as_ref() {
            tracing::info!("unsuppored socks proxy now");
        } else {
            tracing::info!("No proxy settings applied")
        }
        Ok(())
    }
}

#[derive(new, Debug, uniffi::Record)]
pub struct LockOptions {
    targets: Vec<String>,
    comment: Option<String>,
    steal_lock: bool,
}

#[derive(new, Debug, uniffi::Record)]
pub struct UnlockOptions {
    targets: Vec<String>,
    break_lock: bool,
}

#[derive(uniffi::Record, Debug)]
pub struct PatchOptions {
    patch_absolute_path: String,
    wc_absolute_path: String,
    dry_run: bool,
    strip_count: u32,
    reverse: bool,
    ignore_whitespace: bool,
    remove_tempfiles: bool,
}

#[derive(new, Debug, uniffi::Record)]
pub struct Proxy {
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(new, Debug, uniffi::Record)]
pub struct SwitchOptions {
    path: String,
    url: String,
    peg_revision: Revision,
    revision: Revision,
    depth: Depth,
    depth_is_sticky: bool,
    ignore_externals: bool,
    allow_unversioned_obstructions: bool,
    ignore_ancestry: bool,
}

#[derive(new, uniffi::Record)]
pub struct CommitResult {
    items: Vec<CommitItem>,
    info: CommitInfo,
}

#[svnexus_macro::enum_converter(repr_type=ffi::svn_node_kind_t)]
#[derive(
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Copy,
    Debug,
    uniffi::Enum,
    Serialize,
    Deserialize,
    EnumString,
    strum::Display,
)]
pub enum NodeKind {
    None = ffi::svn_node_kind_t_svn_node_none,
    File = ffi::svn_node_kind_t_svn_node_file,
    Directory = ffi::svn_node_kind_t_svn_node_dir,
    Unknown = ffi::svn_node_kind_t_svn_node_unknown,
    Symlink = ffi::svn_node_kind_t_svn_node_symlink,
}

#[derive(uniffi::Record, Debug)]
pub struct CommitItem {
    path: Option<String>,
    kind: NodeKind,
    url: String,
    revision: Option<RevisionNumber>,
    copy_from_url: Option<String>,
    copy_from_revision: Option<RevisionNumber>,
    state: u8,
    incoming_property_changes: HashMap<String, String>, // properties: HashMap<String, String>,
    outgoing_property_changes: HashMap<String, String>,
    session_real_path: Option<String>,
    move_from_absolute_path: Option<String>,
}

// #[repr(i32)]
#[svnexus_macro::enum_converter(repr_type=ffi::svn_depth_t)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, uniffi::Enum)]
pub enum Depth {
    Unknown = ffi::svn_depth_t_svn_depth_unknown,
    Exclude = ffi::svn_depth_t_svn_depth_exclude,
    Empty = ffi::svn_depth_t_svn_depth_empty,
    Files = ffi::svn_depth_t_svn_depth_files,
    Immediates = ffi::svn_depth_t_svn_depth_immediates,
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

#[derive(Debug, uniffi::Record, svnexus_macro::ToDebugString)]
pub struct StatusEntry {
    path: String,
    node_kind: NodeKind,
    local_absolute_path: String,
    file_size: Option<u64>,
    versioned: bool,
    conflicted: bool,
    node_status: WorkingCopyStatus,
    text_status: WorkingCopyStatus,
    property_status: WorkingCopyStatus,
    wc_is_locked: bool,
    copied: bool,
    repository_root_url: Option<String>,
    repository_uuid: Option<String>,
    repository_relpath: Option<String>,
    revision: Option<RevisionNumber>,
    last_changed_revision: Option<RevisionNumber>,
    last_changed_date: i64,
    last_changed_author: Option<String>,
    switched: bool,
    file_external: bool,
    lock: Option<Lock>,
    changelist: Option<String>,
    depth: Depth,
    out_of_date_kind: NodeKind,
    repository_node_status: WorkingCopyStatus,
    repository_text_status: WorkingCopyStatus,
    repository_property_status: WorkingCopyStatus,
    repository_lock: Option<Lock>,
    out_of_date_changed_revision: Option<RevisionNumber>,
    out_of_date_changed_date: Option<i64>,
    out_of_date_changed_author: Option<String>,
    moved_from_absolute_path: Option<String>,
    moved_to_absolute_path: Option<String>,
}

impl StatusEntry {
    fn fix_node_kind(&mut self) -> error::Result<()> {
        if self.node_status != WorkingCopyStatus::Unversioned {
            return Ok(());
        }

        let metadata = std::fs::metadata(self.path.as_str())?;

        if metadata.is_symlink() {
            self.node_kind = NodeKind::Symlink;
        } else if metadata.is_dir() {
            self.node_kind = NodeKind::Directory
        } else if metadata.is_file() {
            self.node_kind = NodeKind::File;
        }

        Ok(())
    }
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
    #[uniffi(default)]
    changelists: Option<Vec<String>>,
    revision_property_table: Option<HashMap<String, String>>,
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
#[derive(Debug, uniffi::Enum, PartialEq, Eq, Hash)]
pub enum WorkingCopyStatus {
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
    revision_property_table: Option<HashMap<String, String>>,
}

#[derive(Debug, uniffi::Record)]
pub struct StatusOptions {
    path: String,
    revision: Revision,
    depth: Depth,
    get_all: bool,
    check_out_of_date: bool,
    check_working_copy: bool,
    no_ignore: bool,
    ignore_externals: bool,
    depth_as_sticky: bool,
    changelist: Option<Vec<String>>,
}

#[uniffi::export(with_foreign)]
pub trait StatusReceiver: Send + Sync + 'static {
    fn on_status_entry(&self, entry: StatusEntry) -> Result<(), error::CSharpError>;
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

#[derive(Debug, Clone, Copy, uniffi::Enum, strum::Display)]
pub enum NativeEOL {
    LF,
    CRLF,
    CR,
    None,
}

impl NativeEOL {
    fn is_none(self) -> bool {
        matches!(self, NativeEOL::None)
    }
}

#[derive(Debug, uniffi::Record)]
pub struct ExportOptions {
    from_path_or_url: String,
    to_path: String,
    peg_revision: Revision,
    revision: Revision,
    r#override: bool,
    ignore_externals: bool,
    ignore_keywords: bool,
    depth: Depth,
    native_eol: NativeEOL,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct Lock {
    path: String,
    token: String,
    owner: String,
    comment: Option<String>,
    is_dav_comment: bool,
    creation_date: i64,
    expiration_date: i64,
}

pub struct BlameOptions {
    path: String,
    peg_revision: Revision,
    start_revision: Revision,
    end_revision: Revision,
    difference_options: DifferenceOptions,
    ignore_mime_type: bool,
    include_merged_revisions: bool,
}

impl Default for Revision {
    fn default() -> Self {
        Self::Unspecified
    }
}

#[easy_ext::ext]
pub impl apr::Pool {
    unsafe fn revision(&mut self, value: Revision) -> *const ffi::svn_opt_revision_t {
        unsafe { self.as_mut_ptr().revision(value) }
    }

    unsafe fn svn_string(&mut self, str: impl AsRef<str>) -> error::Result<*mut ffi::svn_string_t> {
        unsafe { self.as_mut_ptr().svn_string(str) }
    }
}

#[easy_ext::ext]
pub impl *mut ffi::apr_pool_t {
    unsafe fn svn_string(self, str: impl AsRef<str>) -> error::Result<*mut ffi::svn_string_t> {
        unsafe {
            let str = str.as_ref();
            let len = str.len();
            let r = ffi::svn_string_ncreate(str.as_ptr() as _, len.try_into().unwrap(), self);
            Ok(r)
        }
    }

    unsafe fn revision(self, value: Revision) -> *const ffi::svn_opt_revision_t {
        unsafe {
            let revision_ptr = self.malloc::<ffi::svn_opt_revision_t>();

            let revision = revision_ptr.as_mut().unwrap();

            match value {
                Revision::Unspecified => {
                    revision.kind = ffi::svn_opt_revision_kind_svn_opt_revision_unspecified;
                }
                Revision::Number(number) => {
                    revision.kind = ffi::svn_opt_revision_kind_svn_opt_revision_number;
                    *revision.value.number.as_mut() = number.try_into().unwrap();
                }
                Revision::Date(time) => {
                    revision.kind = ffi::svn_opt_revision_kind_svn_opt_revision_date;
                    *revision.value.date.as_mut() = time.try_into().unwrap();
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
            revision_ptr
        }
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

impl From<*const ffi::svn_commit_info_t> for CommitInfo {
    fn from(info: *const ffi::svn_commit_info_t) -> Self {
        unsafe {
            let info = info.as_ref().unwrap();
            Self {
                revision: info.revision.try_into().unwrap(),
                date: info.date.to_str().to_string(),
                author: info.author.to_str().to_string(),
                post_commit_err: info.post_commit_err.to_nullable_string(),
                repos_root: info.repos_root.to_nullable_string(),
            }
        }
    }
}

unsafe extern "C" fn commit_callback(
    commit_info: *const ffi::svn_commit_info_t,
    baton: *mut c_void,
    _pool: *mut ffi::apr_pool_t,
) -> *mut ffi::svn_error_t {
    unsafe {
        let error = on_cancel(baton);
        if !error.is_null() {
            return error;
        }
        let commit_info = CommitInfo::from(commit_info);

        let this = (baton as *mut ContextInner).as_mut().unwrap();

        this.commit_info = Some(commit_info);

        svn_no_error()
    }
}

impl From<*const ffi::svn_lock_t> for Lock {
    fn from(ptr: *const ffi::svn_lock_t) -> Self {
        unsafe {
            let lock = ptr.as_ref().unwrap();
            let path = lock.path.to_str().to_string();

            let token = lock.token.to_str().to_string();

            let owner = lock.owner.to_str().to_string();

            let comment = lock.comment.to_nullable_string();

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
            let path = path.to_str().to_string();

            let entry = entry.as_ref().unwrap();

            let node_kind = NodeKind::try_from(entry.kind).unwrap();

            let local_absolute_path = entry.local_abspath.to_str().to_string();

            // let file_size: u64 = entry.filesize.try_into().unwrap();
            let file_size = if entry.filesize == -1 {
                None
            } else {
                Some(entry.filesize.try_into().unwrap())
            };

            let versioned = entry.versioned != 0;

            let conflicted = entry.conflicted != 0;

            let node_status = WorkingCopyStatus::try_from(entry.node_status).unwrap();

            let text_status = WorkingCopyStatus::try_from(entry.text_status).unwrap();

            let property_status = WorkingCopyStatus::try_from(entry.prop_status).unwrap();

            let wc_is_locked = entry.wc_is_locked != 0;

            let copied = entry.copied != 0;

            let repository_root_url = entry.repos_root_url.to_nullable_string();

            let repository_uuid = entry.repos_uuid.to_nullable_string();

            let repository_relpath = entry.repos_relpath.to_nullable_string();

            let revision = RevisionNumber::try_from(entry.revision).ok();

            let last_changed_revision = entry.changed_rev.try_into().ok();

            let last_changed_date: i64 = entry.changed_date.try_into().unwrap();

            let last_changed_author = entry.changed_author.to_nullable_string();

            let switched = entry.switched != 0;

            let file_external = entry.file_external != 0;

            let lock = entry.lock.map(Lock::from);

            let changelist = entry.changelist.to_nullable_string();

            let depth = Depth::try_from(entry.depth).unwrap();

            let out_of_date_kind = NodeKind::try_from(entry.ood_kind).unwrap();

            let repository_node_status = WorkingCopyStatus::try_from(entry.repos_node_status).unwrap();

            let repository_text_status = WorkingCopyStatus::try_from(entry.repos_text_status).unwrap();

            let repository_property_status = WorkingCopyStatus::try_from(entry.repos_prop_status).unwrap();

            let repository_lock = entry.repos_lock.map(Lock::from);

            let out_of_date_changed_revision = entry.ood_changed_rev.try_into().ok();

            let out_of_date_changed_date = if entry.ood_changed_date == 0 {
                None
            } else {
                Some(entry.ood_changed_date.try_into().unwrap())
            };

            let out_of_date_changed_author = entry.ood_changed_author.to_nullable_string();

            let moved_from_absolute_path = entry.moved_from_abspath.to_nullable_string();

            let moved_to_absolute_path = entry.moved_to_abspath.to_nullable_string();

            Self {
                path,
                node_kind,
                local_absolute_path,
                file_size,
                versioned,
                conflicted,
                node_status,
                text_status,
                property_status,
                wc_is_locked,
                copied,
                repository_root_url,
                repository_uuid,
                repository_relpath,
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
                repository_node_status,
                repository_text_status,
                repository_property_status,
                repository_lock,
                out_of_date_changed_revision,
                out_of_date_changed_date,
                out_of_date_changed_author,
                moved_from_absolute_path,
                moved_to_absolute_path,
            }
        }
    }
}

impl Default for WorkingCopyStatus {
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
    pub log_entries: Vec<LogEntry>,
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
            let item = item.as_ref().unwrap();

            tracing::info!("item path={:?}", CStr::from_ptr(item.path));
            let path = item.path.to_nullable_string();

            let kind = NodeKind::try_from(item.kind).unwrap();

            let url = item.url.to_str().to_string();

            let copy_from_url = item.copyfrom_url.to_nullable_string();

            let mut incoming_property_changes = HashMap::new();
            if !item.incoming_prop_changes.is_null() {
                let prop = item.incoming_prop_changes.as_ref().unwrap();

                incoming_property_changes.reserve(prop.nelts as _);

                for i in 0..prop.nelts {
                    let elements = prop.elts as *const *const ffi::svn_prop_t;

                    let offset = elements.offset(i as _);
                    if (*offset).is_null() {
                        continue;
                    }
                    let element = &**offset;

                    let key = element.name.to_str().to_string();

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

                    let key = element.name.to_str().to_string();

                    let slice = std::slice::from_raw_parts(
                        (*element.value).data as *const u8,
                        (*element.value).len,
                    );
                    let value = String::from_utf8(slice.to_vec()).unwrap();

                    outgoing_property_changes.insert(key, value);
                }
            }

            let revision = item.revision.try_into().ok();
            let state = item.state_flags;

            let session_real_path = item.session_relpath.to_nullable_string();

            let copy_from_revision = if item.copyfrom_url.is_null() || item.copyfrom_rev < 0 {
                None
            } else {
                Some(item.copyfrom_rev.try_into().unwrap())
            };

            let move_from_absolute_path = item.moved_from_abspath.to_nullable_string();

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

// unsafe fn char_from_pool(string: &str, pool: *mut ffi::apr_pool_t) -> *const c_char {
//     let mut bytes = vec![0; string.len() + 1];
//     bytes[..string.len()].copy_from_slice(string.as_bytes());

//     unsafe { ffi::apr_pstrndup(pool, bytes.as_ptr() as _, bytes.len()) }
// }

unsafe extern "C" fn on_progress_notify(
    progress: ffi::apr_off_t,
    total: ffi::apr_off_t,
    baton: *mut c_void,
    pool: *mut ffi::apr_pool_t,
) {
    unsafe {
        let this = &*(baton as *mut ContextInner);
        let result = this
            .context_notifier
            .progress_notify(progress.try_into().unwrap(), total.try_into().unwrap());
        if let Err(e) = result {
            tracing::error!("Unexpected Error from csharp: {}", e);
        }
    }
}

unsafe extern "C" fn on_cancel(baton: *mut c_void) -> *mut ffi::svn_error_t {
    unsafe {
        let ctx = (baton as *mut ContextInner).as_mut().unwrap();
        let v = match ctx.context_notifier.cancel() {
            Ok(v) => v,
            Err(e) => return e.native_error(),
        };
        if let Some(msg) = v {
            let mut pool = apr::Pool::create();
            return ffi::svn_error_create(
                ffi::svn_errno_t_SVN_ERR_CANCELLED as _,
                std::ptr::null_mut(),
                pool.string(msg.as_str()).unwrap_or_default() as _,
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

    tracing::info!("Enter on get commit message");

    unsafe {
        let error = on_cancel(baton);
        if !error.is_null() {
            return error;
        }

        let this = (baton as *mut ContextInner).as_mut().unwrap();

        tracing::info!("Set commit message: {}", this.commit_message);

        *log_msg = pool
            .string(this.commit_message.as_str())
            .expect("Invalid message");

        this.commit_message.clear();

        *tmp_file = std::ptr::null();

        let items = commit_items.to_vec(|ptr| CommitItem::from_item(ptr as _));

        // let items = CommitItem::from_items(commit_items);

        this.commit_items.extend(items);
    }

    svn_no_error()
}

unsafe extern "C" fn on_notify(
    baton: *mut c_void,
    notify: *const ffi::svn_wc_notify_t,
    _pool: *mut ffi::apr_pool_t,
) {
    unsafe {
        let this = (baton as *mut ContextInner).as_mut().unwrap();

        let notify = WorkingCopyNotify::from(notify);

        tracing::info!("Notify: {:#?}", notify);

        let result = this.context_notifier.working_copy_notify(notify);
        if let Err(e) = result {
            tracing::error!("Unexpected error from csharp: {}", e);
        }

        // (this.on_notify)(Notify::from_raw(notify));
    }
}

// struct Pointer<T>(pub T);

impl Default for Depth {
    fn default() -> Self {
        Depth::Unknown
    }
}

unsafe extern "C" fn may_save_password_as_plain_text(
    save: *mut ffi::svn_boolean_t,
    realm_string: *const c_char,
    baton: *mut c_void,
    _pool: *mut ffi::apr_pool_t,
) -> *mut ffi::svn_error_t {
    tracing::info!("may_save_password_as_plain_text");

    unsafe {
        let ctx = (baton as *mut ContextInner).as_mut().unwrap();
        let realm_string = realm_string.to_str();
        // *save = (ctx.on_may_save_password_as_plain_text)(realm_string).into();

        let v = match ctx
            .context_notifier
            .may_save_password_as_plain_text(realm_string.to_string())
        {
            Ok(v) => v,
            Err(e) => return e.native_error(),
        };
        *save = v.into();
    }

    std::ptr::null_mut()
}

unsafe extern "C" fn first_ssl_client_cert_pw() -> *mut ffi::svn_error_t {
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
        unsafe {
            let info = info.as_ref().unwrap();
            let hostname = info.hostname.to_str().to_string();
            let fingerprint = info.fingerprint.to_str().to_string();
            let valid_from = info.valid_from.to_str().to_string();
            let valid_until = info.valid_until.to_str().to_string();
            let issuer = info.issuer_dname.to_str().to_string();
            let ascii_cert = info.ascii_cert.to_str().to_string();

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
        tracing::trace!(
            "Server info: realm={} failures={} info={:?} may_save={}",
            realm,
            failures,
            info,
            may_save
        );

        let this = (baton as *mut ContextInner).as_mut().unwrap();

        let trust = this.context_notifier.ssl_server_trust_prompt(
            realm.to_string(),
            failures,
            // SslFailures::from_bits_retain(failures),
            info,
            may_save != 0,
        );

        match trust {
            Err(e) => {
                return e.native_error();
            }
            Ok(Some(trust)) => {
                *cred = pool.malloc::<ffi::svn_auth_cred_ssl_server_trust_t>();
                let cred = &mut **cred;

                cred.accepted_failures = trust.accept_failures;
                cred.may_save = trust.save as _;
            }
            Ok(None) => {
                *cred = std::ptr::null_mut();
            }
        }

        svn_no_error()
    }
}

// #[derive(TryFromPrimitive)]
// #[repr(u8)]
#[svnexus_macro::enum_converter(repr_type=u8)]
#[derive(uniffi::Enum, Debug, Clone, Copy, Serialize, Deserialize, EnumString, strum::Display)]
pub enum LogChangedPathAction {
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

#[derive(new, Debug, uniffi::Record, Clone, Serialize, Deserialize)]
pub struct LogChangedPathEntry {
    pub action: LogChangedPathAction,
    pub copy_from_path: Option<String>,
    pub copy_from_revision: Option<RevisionNumber>,
    pub node_kind: NodeKind,
    pub text_modified: Option<bool>,
    pub props_modified: Option<bool>,
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
            let copy_from_path = ptr.copyfrom_path.to_nullable_string();
            let copy_from_revision = ptr.copyfrom_rev.try_into().ok();

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

#[derive(Default, new, Debug, uniffi::Record, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub revision: Option<RevisionNumber>,
    pub date: Option<i64>,
    pub author: Option<String>,
    pub message: Option<String>,
    pub revision_properties: Option<HashMap<String, String>>,
    pub changed_path_entries: HashMap<String, LogChangedPathEntry>,
    pub has_children: bool,
    pub non_inheritable: bool,
    pub subtractive_merge: bool, // merged_in_revsions: Vec<RevisionNumber>,
}

impl LogEntry {
    fn from_raw(ptr: *mut ffi::svn_log_entry_t) -> Self {
        unsafe {
            let log_entry = ptr.as_ref().unwrap();

            let mut author: *const c_char = std::ptr::null();
            let mut date: *const c_char = std::ptr::null();
            let mut message: *const c_char = std::ptr::null();

            if !log_entry.revprops.is_null() {
                ffi::svn_compat_log_revprops_out(
                    &mut author as _,
                    &mut date as _,
                    &mut message as _,
                    log_entry.revprops,
                );
            }

            let revision = log_entry.revision.try_into().ok();

            let mut pool = apr::Pool::create();

            let revision_properties = log_entry.revprops.map(|v| pool.convert_to_hash_map(v as _));

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

            let author = author.to_nullable_string();
            let message = message.to_nullable_string();
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
                revision_properties,
                changed_path_entries,
                has_children,
                non_inheritable,
                subtractive_merge,
            }
        }
    }
}

unsafe extern "C" fn on_authenticate(
    cred: *mut *mut ffi::svn_auth_cred_simple_t,
    baton: *mut c_void,
    realm: *const c_char,
    username: *const c_char,
    may_save: ffi::svn_boolean_t,
    pool: *mut ffi::apr_pool_t,
) -> *mut ffi::svn_error_t {
    tracing::info!("on_authenticate");
    unsafe {
        let cancel = on_cancel(baton);
        if !cancel.is_null() {
            return cancel;
        }

        let context = (baton as *mut ContextInner).as_mut().unwrap();
        let v = match context.context_notifier.authenticate(
            realm.to_nullable_str().unwrap_or_default().to_string(),
            username.to_nullable_str().unwrap_or_default().to_string(),
            may_save != 0,
        ) {
            Ok(v) => v,
            Err(e) => return e.native_error(),
        };
        tracing::info!("Authenticate result: {:?}", v);
        if let Some(result) = v {
            *cred = pool.malloc();

            let cred = (*cred).as_mut().unwrap();

            cred.may_save = result.save.into();
            cred.password = pool.string(result.password).expect("Invalid password");
            cred.username = pool.string(result.username).expect("Invalid username");
        }
        svn_no_error()
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
pub struct RelocateOptions {
    pub wc_root_dir: String,
    pub from_prefix: String,
    pub to_prefix: String,
    pub ignore_externals: bool,
}

#[derive(Debug, uniffi::Record)]
pub struct RevertOptions {
    paths: Vec<String>,
    depth: Depth,
    #[uniffi(default)]
    changelists: Option<Vec<String>>,
    #[uniffi(default)]
    clear_changelists: bool,
    metadata_only: bool,
    added_keep_local: bool,
}

#[derive(uniffi::Record, Debug, new)]
pub struct RevisionRange {
    pub start: Revision,
    pub end: Revision,
}

#[derive(Debug, uniffi::Record)]
pub struct LogOptions {
    pub targets: Vec<String>,
    pub peg_revision: Revision,
    pub limit: u32,
    pub revisions: Vec<RevisionRange>,
    pub discover_changed_paths: bool,
    pub strict_node_history: bool,
    pub include_merged_revisions: bool,
    pub revisions_properties: Option<Vec<String>>,
}

#[uniffi::export(with_foreign)]
pub trait LogReceiver: Send + Sync + 'static {
    fn on_log_entry(&self, log_entry: LogEntry) -> Result<(), error::CSharpError>;
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

#[derive(uniffi::Record, Debug)]
pub struct MkdirOptions {
    paths: Vec<String>,
    make_parents: bool,
    revision_property_table: Option<HashMap<String, String>>,
    commit_message: String,
}

#[derive(uniffi::Record, Debug)]
pub struct MkdirResult {
    commit_info: Option<CommitInfo>,
}

#[derive(Debug, new, uniffi::Record)]
pub struct MergeOptions {
    source1: String,
    revision1: Revision,
    source2: String,
    revision2: Revision,
    target: String,
    depth: Depth,
    ignore_merge_info: bool,
    ignore_ancestry: bool,
    force_delete: bool,
    record_only: bool,
    dry_run: bool,
    allow_mixed_revision: bool,
    extra_merge_options: Option<Vec<String>>,
}

#[derive(Debug, new, uniffi::Record)]
pub struct CatOptions {
    path: String,
    peg_revision: Revision,
    revision: Revision,
    expand_keywords: bool,
    get_properties: bool,
}

#[derive(Debug, new, uniffi::Record)]
pub struct CatResult {
    content: Vec<u8>,
    properties: Option<HashMap<String, String>>,
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
    #[uniffi(default)]
    changelists: Option<Vec<String>>,
}

#[derive(Debug, uniffi::Record)]
pub struct InfoResult {
    pub entries: HashMap<String, InfoEntry>,
}

#[derive(Debug, uniffi::Record)]
pub struct UpdateOptions {
    paths: Vec<String>,
    revision: Revision,
    depth: Depth,
    depth_is_sticky: bool,
    ignore_externals: bool,
    allow_unver_obstructions: bool,
    adds_as_modification: bool,
    make_parents: bool,
}

#[derive(Debug, uniffi::Record)]
pub struct PropertyGetOptions {
    property_name: String,
    target: String,
    peg_revision: Revision,
    revision: Revision,
    depth: Depth,
    #[uniffi(default)]
    changelists: Vec<String>,
}

#[derive(Debug, uniffi::Record)]
pub struct PropertyGetResult {
    properties: HashMap<String, String>,
    inherited_properties: Vec<String>,
    actual_revision: RevisionNumber,
}

#[derive(Debug, uniffi::Record)]
pub struct RevisionPropertyListOptions {
    url: String,
    revision: Revision,
}

#[derive(Debug, uniffi::Record)]
pub struct RevisionPropertyListResult {
    properties: HashMap<String, String>,
    revison: RevisionNumber,
}

#[derive(Debug, uniffi::Record)]
pub struct PropertyListOptions {
    target: String,
    peg_revision: Revision,
    revision: Revision,
    depth: Depth,
    #[uniffi(default)]
    changelists: Option<Vec<String>>,
    inherited: bool,
}

#[derive(Debug, Default, uniffi::Record)]
pub struct PropertyListResult {
    entries: Vec<PropertyListEntry>,
}

#[derive(Debug, Default, uniffi::Record)]
pub struct PropertyListEntry {
    path: String,
    properties: Option<HashMap<String, String>>,
    inherited_properties: Option<Vec<InheritedProperty>>,
}

#[derive(Debug, Default, uniffi::Record)]
pub struct InheritedProperty {
    path: String,
    properties: HashMap<String, String>,
}

impl From<*const ffi::svn_prop_inherited_item_t> for InheritedProperty {
    fn from(value: *const ffi::svn_prop_inherited_item_t) -> Self {
        unsafe {
            let value = value.as_ref().unwrap();
            let mut pool = apr::Pool::create();

            let path = value.path_or_url.to_str().to_string();
            let properties = pool.convert_to_hash_map(value.prop_hash);

            Self { path, properties }
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct ListOptions {
    path: String,
    peg_revision: Revision,
    revision: Revision,
    patterns: Option<Vec<String>>,
    depth: Depth,
    dirent_created_revision: bool,
    dirent_has_properties: bool,
    dirent_kind: bool,
    dirent_last_author: bool,
    dirent_size: bool,
    dirent_time: bool,
    fetch_locks: bool,
    include_externals: bool,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct ConflictWalkOptions {
    local_absolute_path: String,
    depth: Depth,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct ConflictWalkResult {
    conflicts: Vec<Conflict>,
}

#[svnexus_macro::enum_converter(repr_type = ffi::svn_client_conflict_option_id_t)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, uniffi::Enum)]
pub enum ConflictOptionId {
    Undefined = ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_undefined,
    Postpone = ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_postpone,
    BaseText = ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_base_text,
    IncomingText = ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_incoming_text,
    WorkingText = ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_working_text,
    IncomingTextWhereConflicted = ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_incoming_text_where_conflicted,
    WorkingTextWhereConflicted = ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_working_text_where_conflicted,
    MergedText = ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_merged_text,
    Unspecified = ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_unspecified,
    AcceptCurrentWcState =
        ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_accept_current_wc_state,
    UpdateMoveDestination =
        ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_update_move_destination,
    UpdateAnyMovedAwayChildren = ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_update_any_moved_away_children,
    IncomingAddIgnore =
        ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_incoming_add_ignore,
    IncomingAddedFileTextMerge = ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_incoming_added_file_text_merge,
    IncomingAddedFileReplaceAndMerge = ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_incoming_added_file_replace_and_merge,
    IncomingAddedDirMerge =
        ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_incoming_added_dir_merge,
    IncomingAddedDirReplace =
        ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_incoming_added_dir_replace,
    IncomingAddedDirReplaceAndMerge = ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_incoming_added_dir_replace_and_merge,
    IncomingDeleteIgnore =
        ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_incoming_delete_ignore,
    IncomingDeleteAccept =
        ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_incoming_delete_accept,
    IncomingMoveFileTextMerge = ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_incoming_move_file_text_merge,
    IncomingMoveDirMerge =
        ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_incoming_move_dir_merge,
    LocalMoveFileTextMerge =
        ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_local_move_file_text_merge,
    LocalMoveDirMerge =
        ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_local_move_dir_merge,
    SiblingMoveFileTextMerge = ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_sibling_move_file_text_merge,
    SiblingMoveDirMerge =
        ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_sibling_move_dir_merge,
    BothMovedFileMerge =
        ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_both_moved_file_merge,
    BothMovedFileMoveMerge =
        ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_both_moved_file_move_merge,
    BothMovedDirMerge =
        ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_both_moved_dir_merge,
    BothMovedDirMoveMerge =
        ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_both_moved_dir_move_merge,
}

pub struct ConflictOption {
    id: ConflictOptionId,
    label: String,
    description: String,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct ConflictPropertyValue {
    base_value: Option<String>,
    working_value: Option<String>,
    incoming_old_value: Option<String>,
    incoming_new_value: Option<String>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct ConflictInfo {
    local_absolute_path: String,
    operation: WorkingCopyOperation,
    incoming_change: WorkingCopyConflictAction,
    local_change: WorkingCopyConflictReason,
    recommended_option_id: ConflictOptionId,
}

impl From<*mut ffi::svn_client_conflict_t> for ConflictInfo {
    fn from(value: *mut ffi::svn_client_conflict_t) -> Self {
        unsafe {
            let local_absolute_path = ffi::svn_client_conflict_get_local_abspath(value)
                .to_str()
                .to_string();
            let operation = ffi::svn_client_conflict_get_operation(value)
                .try_into()
                .unwrap();
            let incoming_change = ffi::svn_client_conflict_get_incoming_change(value)
                .try_into()
                .unwrap();
            let local_change = ffi::svn_client_conflict_get_local_change(value)
                .try_into()
                .unwrap();
            let recommended_option_id = ffi::svn_client_conflict_get_recommended_option_id(value)
                .try_into()
                .unwrap();
            ConflictInfo {
                local_absolute_path,
                operation,
                incoming_change,
                local_change,
                recommended_option_id,
            }
        }
    }
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum Conflict {
    Text {
        info: ConflictInfo,
        base_absolute_path: Option<String>,
        working_absolute_path: Option<String>,
        incoming_old_absolute_path: Option<String>,
        incoming_new_absolute_path: Option<String>,
    },
    Property {
        info: ConflictInfo,
        properties: HashMap<String, ConflictPropertyValue>,
        description: Option<String>,
    },
    Tree {
        info: ConflictInfo,
        victim_node_kind: NodeKind,
    },
}

#[derive(Debug, Clone, new, uniffi::Record)]
pub struct ListResult {
    entries: Vec<ListEntry>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct ListEntry {
    path: String,
    kind: Option<NodeKind>,
    size: Option<u64>,
    has_properties: Option<bool>,
    created_revision: Option<RevisionNumber>,
    time: Option<i64>,
    last_author: Option<String>,
    lock: Option<Lock>,
    absolute_path: String,
    external_parent_url: Option<String>,
    external_target: Option<String>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct CopySourceItem {
    path: String,
    peg_revision: Revision,
    revision: Revision,
}

#[derive(Debug, Clone, new, uniffi::Record)]
pub struct CopyOptions {
    sources: Vec<CopySourceItem>,
    destination: String,
    copy_as_child: bool,
    make_parents: bool,
    ignore_externals: bool,
    metadata_only: bool,
    pin_externals: bool,
    #[uniffi(default)]
    commit_message: String,
}

#[svnexus_macro::enum_converter(repr_type=ffi::svn_diff_file_ignore_space_t)]
#[derive(uniffi::Enum, Debug, Clone, Copy)]
pub enum DiffFileIgnoreSpace {
    None = ffi::svn_diff_file_ignore_space_t_svn_diff_file_ignore_space_none,
    Change = ffi::svn_diff_file_ignore_space_t_svn_diff_file_ignore_space_change,
    All = ffi::svn_diff_file_ignore_space_t_svn_diff_file_ignore_space_all,
}

#[derive(uniffi::Record, Debug)]
pub struct DifferenceFileOptions {
    ignore_space: DiffFileIgnoreSpace,
    ignore_eol_style: bool,
    show_c_function: bool,
    context_size: i32,
}

impl DifferenceFileOptions {
    fn setup(&self, options: &mut ffi::svn_diff_file_options_t) {
        options.ignore_eol_style = self.ignore_eol_style.into();
        options.ignore_space = self.ignore_space.into();
        options.show_c_function = self.show_c_function.into();
        options.context_size = self.context_size.into();
    }
}

// impl From<DifferenceFileOptions> for ffi::svn_diff_file_options_t {
//     fn from(value: DifferenceFileOptions) -> Self {
//         Self {
//             ignore_eol_style: value.ignore_eol_style.into(),
//             ignore_space: value.ignore_space.into(),
//             show_c_function: value.show_c_function.into(),
//             context_size: value.context_size.try_into().unwrap(),
//         }
//     }
// }

#[derive(uniffi::Record, Debug)]
pub struct DifferenceOptions {
    pub original: Vec<u8>,
    pub modified: Vec<u8>,
    pub options: Option<DifferenceFileOptions>,
}

#[uniffi::export]
impl DifferenceOptions {
    pub fn exec(self) -> error::Result<DifferenceResult> {
        unsafe {
            let mut pool = apr::Pool::create();

            let original = ffi::svn_string_ncreate(
                self.original.as_ptr() as _,
                self.original.len().try_into().unwrap(),
                pool.as_mut_ptr(),
            );

            let modified = ffi::svn_string_ncreate(
                self.modified.as_ptr() as _,
                self.modified.len().try_into().unwrap(),
                pool.as_mut_ptr(),
            );

            let mut diff = std::ptr::null_mut::<ffi::svn_diff_t>();

            let file_options = if let Some(options) = self.options {
                let file_options = ffi::svn_diff_file_options_create(pool.as_mut_ptr());

                options.setup(file_options.as_mut().unwrap());
                file_options
            } else {
                std::ptr::null_mut()
            };


            // let options: ffi::svn_diff_file_options_t = options.options.into();

            tracing::info!("{}:{}", file!(), line!());
            let error = ffi::svn_diff_mem_string_diff(
                diff.pointer_mut(),
                original,
                modified,
                file_options,
                pool.as_mut_ptr(),
            );
            tracing::info!("{}:{}", file!(), line!());

            SVNError::from_nullable_ptr(error).context(builder::Svn)?;

            let mut functions = ffi::svn_diff_output_fns_t::default();

            let mut modified: Vec<TextChange> = Vec::with_capacity(128);

            unsafe extern "C" fn output_common(
                output_baton: *mut ::std::os::raw::c_void,
                original_start: ffi::apr_off_t,
                original_length: ffi::apr_off_t,
                modified_start: ffi::apr_off_t,
                modified_length: ffi::apr_off_t,
                latest_start: ffi::apr_off_t,
                latest_length: ffi::apr_off_t,
            ) -> *mut ffi::svn_error_t {
                tracing::info!("output_common");
                tracing::info!("+==============");
                tracing::info!("original_start: {}", original_start);
                tracing::info!("original_length: {}", original_length);
                tracing::info!("modified_start: {}", modified_start);
                tracing::info!("modified_length: {}", modified_length);
                tracing::info!("latest_start: {}", latest_start);
                tracing::info!("latest_length: {}", latest_length);
                tracing::info!("-==============");

                svn_no_error()
            }

            unsafe extern "C" fn output_diff_modified(
                output_baton: *mut c_void,
                original_start: ffi::apr_off_t,
                original_length: ffi::apr_off_t,
                modified_start: ffi::apr_off_t,
                modified_length: ffi::apr_off_t,
                latest_start: ffi::apr_off_t,
                latest_length: ffi::apr_off_t,
            ) -> *mut ffi::svn_error_t {
                tracing::info!("output_diff_modified");
                tracing::info!("*==============");
                tracing::info!("original_start: {}", original_start);
                tracing::info!("original_length: {}", original_length);
                tracing::info!("modified_start: {}", modified_start);
                tracing::info!("modified_length: {}", modified_length);
                tracing::info!("latest_start: {}", latest_start);
                tracing::info!("latest_length: {}", latest_length);
                tracing::info!("/==============");

                unsafe {
                    let changes = output_baton as *mut Vec<TextChange>;
                    let changes = changes.as_mut().unwrap();

                    let original = TextPosition {
                        pos: original_start.try_into().unwrap_or_default(),
                        len: original_length.try_into().unwrap_or_default(),
                    };

                    let modified = TextPosition {
                        pos: modified_start.try_into().unwrap_or_default(),
                        len: modified_length.try_into().unwrap_or_default(),
                    };

                    let change = TextChange { original, modified };

                    // let mut change = TextChange::default();

                    // if original_length > 0 {
                    //     change.original = Some(TextPosition {
                    //         pos: original_start.try_into().unwrap(),
                    //         len: original_length.try_into().unwrap(),
                    //     });
                    // }

                    // if modified_length > 0 {
                    //     change.modified = Some(TextPosition {
                    //         pos: modified_start.try_into().unwrap(),
                    //         len: modified_length.try_into().unwrap(),
                    //     });
                    // }

                    changes.push(change);
                }

                std::ptr::null_mut()
            }

            unsafe extern "C" fn output_diff_common(
                output_baton: *mut ::std::os::raw::c_void,
                original_start: ffi::apr_off_t,
                original_length: ffi::apr_off_t,
                modified_start: ffi::apr_off_t,
                modified_length: ffi::apr_off_t,
                latest_start: ffi::apr_off_t,
                latest_length: ffi::apr_off_t,
            ) -> *mut ffi::svn_error_t {
                tracing::info!("output_diff_common");
                tracing::info!("(==============");
                tracing::info!("original_start: {}", original_start);
                tracing::info!("original_length: {}", original_length);
                tracing::info!("modified_start: {}", modified_start);
                tracing::info!("modified_length: {}", modified_length);
                tracing::info!("latest_start: {}", latest_start);
                tracing::info!("latest_length: {}", latest_length);
                tracing::info!(")==============");

                svn_no_error()
            }

            unsafe extern "C" fn output_diff_latest(
                output_baton: *mut ::std::os::raw::c_void,
                original_start: ffi::apr_off_t,
                original_length: ffi::apr_off_t,
                modified_start: ffi::apr_off_t,
                modified_length: ffi::apr_off_t,
                latest_start: ffi::apr_off_t,
                latest_length: ffi::apr_off_t,
            ) -> *mut ffi::svn_error_t {
                tracing::info!("output_diff_latest");
                tracing::info!("[==============");
                tracing::info!("original_start: {}", original_start);
                tracing::info!("original_length: {}", original_length);
                tracing::info!("modified_start: {}", modified_start);
                tracing::info!("modified_length: {}", modified_length);
                tracing::info!("latest_start: {}", latest_start);
                tracing::info!("latest_length: {}", latest_length);
                tracing::info!("]==============");

                svn_no_error()
            }

            functions.output_diff_common = Some(output_diff_common);
            functions.output_diff_modified = Some(output_diff_modified);
            functions.output_common = Some(output_common);
            functions.output_diff_latest = Some(output_diff_latest);

            ffi::svn_diff_output2(
                diff,
                modified.pointer_mut() as *mut c_void,
                functions.pointer(),
                None,
                std::ptr::null_mut::<std::ffi::c_void>(),
            );

            Ok(DifferenceResult { modified })
        }
    }
}

#[derive(uniffi::Record, Debug)]
pub struct ClientDifferenceOptions {
    options: Option<Vec<String>>,
    path1: String,
    revision1: Revision,
    path2: String,
    revision2: Revision,
    relate_to: Option<String>,
    depth: Depth,
    ignore_ancestry: bool,
    no_added: bool,
    no_deleted: bool,
    show_copies_as_adds: bool,
    ignore_content_type: bool,
    ignore_properties: bool,
    properties_only: bool,
    use_git_format: bool,
    pretty_print_merge_info: bool,
    header_encoding: String,
    #[uniffi(default)]
    changelists: Option<Vec<String>>,
}

#[derive(uniffi::Record, Debug)]
pub struct ClientDifferenceResult {
    out: Vec<u8>,
    err: Vec<u8>,
}

#[derive(uniffi::Record, Debug)]
pub struct TextPosition {
    pub pos: u64,
    pub len: u64,
}

#[derive(uniffi::Record, Debug)]
pub struct TextChange {
    pub original: TextPosition,
    pub modified: TextPosition,
}

#[derive(uniffi::Record, Debug)]
pub struct DifferenceResult {
    pub modified: Vec<TextChange>,
}

#[derive(uniffi::Record, Debug, new)]
pub struct InfoEntry {
    url: String,
    revision: RevisionNumber,
    repository_root_url: String,
    repository_uuid: String,
    kind: NodeKind,
    size: Option<u64>,
    last_changed_revision: RevisionNumber,
    last_changed_date: i64,
    last_changed_author: String,
    lock: Option<Lock>,
    working_copy_info: Option<WorkingCopyInfo>,
}

#[derive(uniffi::Record, Debug, new)]
pub struct GetRepositoryRootResult {
    pub root_url: String,
    pub uuid: String,
}

#[easy_ext::ext]
impl *const ffi::svn_client_info2_t {
    fn to_info_entry(self) -> InfoEntry {
        unsafe {
            let ptr = self.as_ref().unwrap();

            let url = ptr.URL.to_str().to_string();

            let revision = ptr.rev.try_into().unwrap();

            let repository_root_url = ptr.repos_root_URL.to_str().to_string();

            let repository_uuid = ptr.repos_UUID.to_str().to_string();

            let kind = ptr.kind.try_into().unwrap();

            let size = if ptr.size < 0 {
                None
            } else {
                Some(ptr.size.try_into().unwrap())
            };

            let last_changed_revision = ptr.last_changed_rev.try_into().unwrap();

            let last_changed_date = ptr.last_changed_date.try_into().unwrap();

            let last_changed_author = ptr.last_changed_author.to_str().to_string();

            let lock = if ptr.lock.is_null() {
                None
            } else {
                Some(Lock::from(ptr.lock))
            };

            let working_copy_info = if ptr.wc_info.is_null() {
                None
            } else {
                Some(ptr.wc_info.into())
            };

            InfoEntry {
                url,
                revision,
                repository_root_url,
                repository_uuid,
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

#[derive(uniffi::Record)]
pub struct InitializeRepositoryOptions {
    pub local: String,
    pub remote: String,
    pub backup: bool,
}

#[uniffi::export(with_foreign)]
pub trait InitializeRepositoryNotifier: Send + Sync + 'static {
    fn on_checkout_directly(&self) -> error::Result<()>;
    fn on_import(&self) -> error::Result<()>;
    fn on_backup(&self) -> error::Result<()>;
    fn on_backup_finished(&self, path: String) -> error::Result<()>;
    fn on_checkout(&self) -> error::Result<()>;
    fn on_finished(&self) -> error::Result<()>;
}

#[derive(Debug, uniffi::Record)]
pub struct CleanupOptions {
    pub path: String,
    break_locks: bool,
    fix_recorded_timestamps: bool,
    clear_dav_cache: bool,
    vacuum_pristines: bool,
    include_externals: bool,
}

impl Context {
    pub fn ctx(&mut self) -> *mut ffi::svn_client_ctx_t {
        self.ptr
    }

    

    fn cancelled(&self) -> error::Result<()> {
        let msg = self.inner.context_notifier.cancel().map_err(|e| {
            builder::General {
                detail: format!("Unexpected error: {}", e),
            }
            .build()
        })?;
        if let Some(msg) = msg {
            builder::General {
                detail: format!("Cancelled: {}", msg),
            }
            .fail()
        } else {
            Ok(())
        }
    }

    pub fn mkdir(&mut self, opts: MkdirOptions) -> error::Result<MkdirResult> {
        unsafe {
            let mut pool = apr::Pool::create();

            let paths = pool.string_array(opts.paths.len(), opts.paths.iter())?;

            let table = opts
                .revision_property_table
                .map(|e| {
                    pool.string_hash_map(
                        e.iter(),
                        |p: &mut apr::Pool, k: &str| p.string(k).map(|o| o as _),
                        |p: &mut apr::Pool, v: &str| p.svn_string(v).map(|o| o as _),
                    )
                })
                .transpose()?
                .unwrap_or_default();

            let error = ffi::svn_client_mkdir4(
                paths,
                opts.make_parents.into(),
                table,
                Some(commit_callback),
                self.inner.inner_void_pointer_mut(),
                self.ctx(),
                pool.as_mut_ptr(),
            );

            SVNError::from_nullable_ptr(error).context(builder::Svn)?;

            Ok(MkdirResult {
                commit_info: self.inner.commit_info.take(),
            })
        }
    }

    pub fn revision_property_list(
        &mut self,
        opts: RevisionPropertyListOptions,
    ) -> error::Result<RevisionPropertyListResult> {
        unsafe {
            let mut pool = apr::Pool::create();
            let mut hash: *mut ffi::apr_hash_t = std::ptr::null_mut();

            let url = pool.string(opts.url)?;

            let revision = opts.revision.to_opt_revision();

            let mut revision_number: ffi::svn_revnum_t = 0;

            let error = ffi::svn_client_revprop_list(
                hash.pointer_mut(),
                url,
                revision.pointer(),
                revision_number.pointer_mut(),
                self.ctx(),
                pool.as_mut_ptr(),
            );

            SVNError::from_nullable_ptr(error).context(builder::Svn)?;

            Ok(RevisionPropertyListResult {
                revison: revision_number.try_into().unwrap(),
                properties: pool.convert_to_hash_map(hash),
            })
        }
    }

    pub fn property_list(
        &mut self,
        opts: PropertyListOptions,
    ) -> error::Result<PropertyListResult> {
        unsafe extern "C" fn receiver(
            baton: *mut c_void,
            path: *const c_char,
            prop_hash: *mut ffi::apr_hash_t,
            inherited_props: *mut ffi::apr_array_header_t,
            scratch_pool: *mut ffi::apr_pool_t,
        ) -> *mut ffi::svn_error_t {
            unsafe {
                let result = (baton as *mut PropertyListResult).as_mut().unwrap();
                let path = path.to_str().to_string();
                let properties = prop_hash.map_mut(|hash| scratch_pool.convert_to_hash_map(hash));
                let inherited_properties =
                    inherited_props.map(|e| e.to_vec(|e| InheritedProperty::from(e as *const _)));

                result.entries.push(PropertyListEntry {
                    path,
                    properties,
                    inherited_properties,
                });
            }

            svn_no_error()
        }
        let mut result = PropertyListResult::default();
        unsafe {
            let mut pool = apr::Pool::create();
            let target = pool.string(opts.target)?;

            let peg_revision = opts.peg_revision.to_opt_revision();
            let revision = opts.revision.to_opt_revision();

            let changelists = opts
                .changelists
                .as_ref()
                .map(|e| pool.string_array(e.len(), e.iter()))
                .transpose()?
                .unwrap_or_default();

            let error = ffi::svn_client_proplist4(
                target,
                peg_revision.pointer(),
                revision.pointer(),
                opts.depth.into(),
                changelists,
                opts.inherited.into(),
                Some(receiver),
                result.pointer_mut() as _,
                self.ctx(),
                pool.as_mut_ptr(),
            );
            SVNError::from_nullable_ptr(error).context(builder::Svn)?;
            Ok(result)
        }
    }

    pub fn initialize_repository(
        &mut self,
        opts: InitializeRepositoryOptions,
        notifier: Arc<dyn InitializeRepositoryNotifier>,
    ) -> error::Result<()> {
        if std::fs::read_dir(&opts.local)?.next().is_none() {
            notifier.on_checkout_directly()?;
            let options = CheckoutOptions {
                url: opts.remote,
                path: opts.local,
                peg_revision: Revision::Head,
                revision: Revision::Head,
                depth: Depth::Infinity,
                ignore_externals: false,
                allow_unversioned_obstructions: true,
                store_pristine: None,
            };
            self.checkout(options)?;
            notifier.on_finished()?;
        } else {
            notifier.on_import()?;
            let import_optios = ImportOptions {
                path: opts.local.clone(),
                url: opts.remote.clone(),
                depth: Depth::Infinity,
                no_ignore: false,
                no_autoprops: false,
                ignore_unknown_node_types: true,
                revision_property_table: Default::default(),
                commit_message: "init".to_string(),
            };
            self.import(import_optios)?;
            self.cancelled()?;
            if opts.backup {
                notifier.on_backup()?;
                let file = utils::backup(&opts.local)?;
                notifier.on_backup_finished(file.to_str().unwrap().to_string())?;
            }
            self.cancelled()?;
            utils::clear_dir(&opts.local)?;

            notifier.on_checkout()?;

            self.cancelled()?;

            let checkout_options = CheckoutOptions {
                path: opts.local.clone(),
                url: opts.remote,
                revision: Revision::Head,
                depth: Depth::Infinity,
                ignore_externals: false,
                allow_unversioned_obstructions: true,
                store_pristine: None,
                peg_revision: Revision::Head,
            };
            self.checkout(checkout_options)?;

            notifier.on_finished()?;
        }
        Ok(())
    }

    pub fn status_next(
        &mut self,
        opts: StatusOptions,
        receiver: Arc<dyn StatusReceiver>,
    ) -> error::Result<()> {
        unsafe extern "C" fn status_receiver(
            baton: *mut c_void,
            path: *const c_char,
            status: *const ffi::svn_client_status_t,
            pool: *mut ffi::apr_pool_t,
        ) -> *mut ffi::svn_error_t {
            unsafe {
                let error = on_cancel(baton);
                if !error.is_null() {
                    return error;
                }

                let this = (baton as *mut ContextInner).as_mut().unwrap();

                let mut entry = StatusEntry::from_path_and_ptr(path, status);

                let result = entry.fix_node_kind();
                if let Err(err) = result {
                    tracing::warn!("Failed to detect node kind of {}:{}", entry.path, err);
                }

                this.status_receiver
                    .as_ref()
                    .expect("status receiver must be set")
                    .on_status_entry(entry)
                    .native_error()
            }
        }

        let mut result_revision: ffi::svn_revnum_t = 0;

        let revision = opts.revision.to_opt_revision();

        // tracing::info!("status options={:#?}", opts);

        unsafe {
            let mut pool = apr::Pool::create();
            let path = pool.string(opts.path.as_str())?;
            let changelist = opts
                .changelist
                .as_ref()
                .map(|e| pool.string_array(e.len(), e.iter()))
                .transpose()?
                .unwrap_or_default();

            self.inner.status_receiver = Some(receiver);

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
                Some(status_receiver),
                &mut *self.inner as *mut ContextInner as *mut c_void,
                pool.as_mut_ptr(),
            );

            self.inner.status_receiver.take();

            SVNError::from_nullable_ptr(error).context(builder::Svn)?;
        };

        Ok(())
    }

    pub fn status(&mut self, opts: StatusOptions) -> error::Result<StatusResult> {
        unsafe extern "C" fn status_receiver(
            baton: *mut c_void,
            path: *const c_char,
            status: *const ffi::svn_client_status_t,
            _pool: *mut ffi::apr_pool_t,
        ) -> *mut ffi::svn_error_t {
            unsafe {
                let error = on_cancel(baton);
                if !error.is_null() {
                    return error;
                }

                let this = (baton as *mut ContextInner).as_mut().unwrap();

                let mut entry = StatusEntry::from_path_and_ptr(path, status);

                let result = entry.fix_node_kind();
                if let Err(err) = result {
                    tracing::warn!("Failed to detect node kind of {}:{}", entry.path, err);
                }

                this.status_entries.push(entry);
            }
            // tracing::info!("translated entry: {:#?}", entry);

            svn_no_error()
        }

        let mut result_revision: ffi::svn_revnum_t = 0;

        let revision = opts.revision.to_opt_revision();

        // let mut changelist =
        // unsafe { apr::Array::from_string_list(opts.changelist.len(), opts.changelist)? };

        self.inner.status_entries.clear();

        tracing::trace!("status options={:#?}", opts);

        unsafe {
            let mut pool = apr::Pool::create();
            let path = pool.string(opts.path.as_str())?;
            let changelist = opts
                .changelist
                .as_ref()
                .map(|e| pool.string_array(e.len(), e.iter()))
                .transpose()?
                .unwrap_or_default();

            let is_absolute = ffi::svn_dirent_is_absolute(path) != 0;
            if !is_absolute {
                return builder::InvalidArgument {
                    detail: format!("path is not absolute: {}", opts.path),
                }
                .fail();
            }

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
                Some(status_receiver),
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

    pub fn export(&mut self, opts: ExportOptions) -> error::Result<RevisionNumber> {
        unsafe {
            let mut pool = apr::Pool::create();
            let native_eol = if opts.native_eol.is_none() {
                std::ptr::null_mut()
            } else {
                pool.string(opts.native_eol.to_string())?
            };

            let mut revision_number: ffi::svn_revnum_t = 0;

            let from_path_or_url = pool.string(opts.from_path_or_url)?;
            let to_path = pool.string(opts.to_path)?;
            let to_path = ffi::svn_dirent_canonicalize(to_path, pool.as_mut_ptr());
            let peg_revision = opts.peg_revision.to_opt_revision();
            let revision = opts.revision.to_opt_revision();

            let error = ffi::svn_client_export5(
                revision_number.pointer_mut(),
                from_path_or_url,
                to_path,
                peg_revision.pointer(),
                revision.pointer(),
                opts.r#override.into(),
                opts.ignore_externals.into(),
                opts.ignore_keywords.into(),
                opts.depth.into(),
                native_eol,
                self.ctx(),
                pool.as_mut_ptr(),
            );
            SVNError::from_nullable_ptr(error).context(builder::Svn)?;
            Ok(RevisionNumber::try_from(revision_number).expect("Unexpected revision number"))
        }
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

    pub fn add(&mut self, opts: AddOptions) -> error::Result<()> {
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
                self.ctx(),
                pool.as_mut_ptr(),
            );
            SVNError::from_nullable_ptr(error).context(builder::Svn)
        }
    }

    pub fn commit(&mut self, opts: CommitOptions) -> error::Result<CommitResult> {
        tracing::info!("Commit optinos: {:#?}", opts);
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

            let changelists = opts
                .changelists
                .map(|e| pool.string_array(e.len(), e.iter()))
                .transpose()?
                .unwrap_or_default();

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
            //

            let revision_properties_table = opts
                .revision_property_table
                .as_ref()
                .map(|e| {
                    pool.string_hash_map(
                        e.iter(),
                        |p, k| p.string(k).map(|o| o as _),
                        |p, v| p.svn_string(v).map(|o| o as _),
                    )
                })
                .transpose()?
                .unwrap_or_default();

            tracing::info!("Table is {:?}", revision_properties_table);

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
                self.inner.inner_void_pointer_mut(),
                self.ctx(),
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
        tracing::info!("Delete options: {:#?}", opts);
        unsafe {
            let mut pool = apr::Pool::create();

            let path = pool.string_array(opts.path.len(), opts.path.iter())?;

            let table = opts
                .revision_property_table
                .map(|e| {
                    pool.string_hash_map(
                        e.iter(),
                        |p: &mut apr::Pool, k: &str| p.string(k).map(|o| o as _),
                        |p: &mut apr::Pool, v: &str| p.svn_string(v).map(|o| o as _),
                    )
                })
                .transpose()?
                .unwrap_or_default();

            let error = ffi::svn_client_delete4(
                path,
                opts.force.into(),
                opts.keep_local.into(),
                table,
                Some(commit_callback),
                self.inner.inner_void_pointer_mut(),
                self.ctx(),
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
                opts.changelists
                    .map(|c| pool.string_array(c.len(), c.iter()))
                    .transpose()?
                    .unwrap_or_default(),
                opts.clear_changelists.into(),
                opts.metadata_only.into(),
                opts.added_keep_local.into(),
                self.ctx(),
                pool.as_mut_ptr(),
            );
            SVNError::from_nullable_ptr(error).context(builder::Svn)?;
        }

        Ok(())
    }

    pub fn log_next(
        &mut self,
        opts: LogOptions,
        receiver: Arc<dyn LogReceiver>,
    ) -> error::Result<()> {
        unsafe extern "C" fn log_receiver(
            baton: *mut c_void,
            log_entry: *mut ffi::svn_log_entry_t,
            pool: *mut ffi::apr_pool_t,
        ) -> *mut ffi::svn_error_t {
            unsafe {
                let error = on_cancel(baton);
                if !error.is_null() {
                    return error;
                }
                let this = (baton as *mut ContextInner).as_mut().unwrap();
                let log_entry = LogEntry::from_raw(log_entry);
                // tracing::info!("On get log entry: {:#?}", log_entry);
                let result = this.log_receiver.as_ref().unwrap().on_log_entry(log_entry);
                result.native_error()
                // if let Err(err) = result {
                //     return ffi::svn_error_create(
                //         ffi::svn_errno_t_SVN_ERR_CANCELLED as _,
                //         std::ptr::null_mut(),
                //         pool.string(err.to_string().as_str()).unwrap_or_default() as _,
                //     );
                // }
            }
        }
        unsafe {
            let mut pool = apr::Pool::create();
            let targets = pool.string_array(opts.targets.len(), opts.targets.iter())?;

            let peg_revision = opts.peg_revision.to_opt_revision();

            let revisions_properties = opts
                .revisions_properties
                .as_ref()
                .map(|props| pool.string_array(props.len(), props.iter()))
                .transpose()?
                .unwrap_or_default();

            self.inner.log_receiver = Some(receiver);

            let error = ffi::svn_client_log5(
                targets,
                &peg_revision as *const _,
                pool.revision_range(&opts.revisions),
                opts.limit.try_into().unwrap_or(std::ffi::c_int::MAX),
                opts.discover_changed_paths.into(),
                opts.strict_node_history.into(),
                opts.include_merged_revisions.into(),
                revisions_properties,
                Some(log_receiver),
                self.inner.inner_void_pointer_mut(),
                self.ctx(),
                pool.as_mut_ptr(),
            );

            self.inner.log_receiver = None;

            SVNError::from_nullable_ptr(error).context(builder::Svn)?;
        }
        Ok(())
    }

    pub fn log(&mut self, opts: LogOptions) -> error::Result<LogResult> {
        unsafe extern "C" fn svn_client_log_entries_receiver(
            baton: *mut c_void,
            log_entry: *mut ffi::svn_log_entry_t,
            pool: *mut ffi::apr_pool_t,
        ) -> *mut ffi::svn_error_t {
            unsafe {
                let error = on_cancel(baton);
                if !error.is_null() {
                    return error;
                }
                let this = (baton as *mut ContextInner).as_mut().unwrap();
                this.log_entries.push(LogEntry::from_raw(log_entry));
            }
            svn_no_error()
        }

        unsafe {
            let mut pool = apr::Pool::create();
            let targets = pool.string_array(opts.targets.len(), opts.targets.iter())?;

            let peg_revision = opts.peg_revision.to_opt_revision();

            self.inner.log_entries.clear();

            let revisions_properties = opts
                .revisions_properties
                .as_ref()
                .map(|props| pool.string_array(props.len(), props.iter()))
                .transpose()?
                .unwrap_or_default();

            let error = ffi::svn_client_log5(
                targets,
                &peg_revision as *const _,
                pool.revision_range(&opts.revisions),
                opts.limit.try_into().unwrap_or(std::ffi::c_int::MAX),
                opts.discover_changed_paths.into(),
                opts.strict_node_history.into(),
                opts.include_merged_revisions.into(),
                revisions_properties,
                Some(svn_client_log_entries_receiver),
                self.inner.inner_void_pointer_mut(),
                self.ctx(),
                pool.as_mut_ptr(),
            );

            SVNError::from_nullable_ptr(error).context(builder::Svn)?;
        }
        tracing::info!("got entry size: {}", self.inner.log_entries.len());
        let r = LogResult::new(std::mem::take(&mut self.inner.log_entries));
        tracing::info!("result: {}", r.log_entries.len());
        Ok(r)
    }

    pub fn import(&mut self, opts: ImportOptions) -> error::Result<ImportResult> {
        unsafe extern "C" fn svn_client_import_filter(
            baton: *mut c_void,
            filtered: *mut ffi::svn_boolean_t,
            local_abspath: *const c_char,
            direct: *const ffi::svn_io_dirent2_t,
            pool: *mut ffi::apr_pool_t,
        ) -> *mut ffi::svn_error_t {
            svn_no_error()
        }
        unsafe {
            let mut pool = apr::Pool::create();
            if opts.commit_message.as_bytes().contains(&b'\0') {
                return builder::General {
                    detail: "Invalid commit message",
                }
                .fail();
            }
            self.inner.commit_message = opts.commit_message;
            self.inner.commit_items.clear();

            let error = ffi::svn_client_import5(
                pool.string(opts.path)?,
                pool.string(opts.url)?,
                opts.depth.into(),
                opts.no_ignore.into(),
                opts.no_autoprops.into(),
                opts.ignore_unknown_node_types.into(),
                pool.string_hash_map(
                    opts.revision_property_table.iter(),
                    |p, k| p.string(k).map(|o| o as _),
                    |p, v| p.svn_string(v).map(|o| o as _),
                )?,
                Some(svn_client_import_filter),
                self.inner.inner_void_pointer_mut(),
                Some(commit_callback),
                self.inner.inner_void_pointer_mut(),
                self.ctx(),
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
            let mut properties: *mut ffi::apr_hash_t = std::ptr::null_mut();

            let mut pool = apr::Pool::create();

            let mut stream = Stream::create(Default::default());

            let path = pool.string(opts.path.as_str())?;

            let peg_revision = opts.peg_revision.to_opt_revision();

            let revision = opts.revision.to_opt_revision();

            let error = ffi::svn_client_cat3(
                if opts.get_properties {
                    properties.pointer_mut()
                } else {
                    std::ptr::null_mut()
                },
                stream.ptr(),
                path,
                &peg_revision as *const _,
                &revision as *const _,
                opts.expand_keywords.into(),
                self.ptr,
                pool.as_mut_ptr(),
                pool.as_mut_ptr(),
            );

            SVNError::from_nullable_ptr(error).context(builder::Svn)?;
            let content = stream.take_write_buffer();

            let properties = if properties.is_null() {
                None
            } else {
                Some(pool.convert_to_hash_map(properties))
            };

            Ok(CatResult {
                content,
                properties,
            })
        }
    }

    pub fn info(&mut self, opts: InfoOptions) -> error::Result<InfoResult> {
        unsafe extern "C" fn info_receiver(
            baton: *mut c_void,
            path: *const c_char,
            info: *const ffi::svn_client_info2_t,
            scratch_pool: *mut apr::ffi::apr_pool_t,
        ) -> *mut ffi::svn_error_t {
            unsafe {
                let error = on_cancel(baton);
                if !error.is_null() {
                    return error;
                }
                let context = (baton as *mut ContextInner).as_mut().unwrap();
                let path = path.to_str().to_string();
                let info = info.to_info_entry();
                context.info_entries.insert(path, info);
            }

            svn_no_error()
        }
        unsafe {
            let mut pool = apr::Pool::create();

            let path = pool.string(opts.path.as_str())?;

            let peg_revision = opts.peg_revision.to_opt_revision();

            let revision = opts.revision.to_opt_revision();

            let depth = opts.depth.into();

            let fetch_excluded = opts.fetch_excluded.into();

            let fetch_actual_only = opts.fetch_actual_only.into();

            let include_externals = opts.include_externals.into();

            let changelists = opts
                .changelists
                .map(|p| pool.string_array(p.len(), p.iter()))
                .transpose()?
                .unwrap_or_default();

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
                self.ctx(),
                pool.as_mut_ptr(),
            );

            SVNError::from_nullable_ptr(error).context(builder::Svn)?;
        }

        let entries = std::mem::take(&mut self.inner.info_entries);

        Ok(InfoResult { entries })
    }

    pub fn url_from_path(&mut self, path: String) -> error::Result<String> {
        unsafe {
            let mut pool = apr::Pool::create();
            let mut url: *const i8 = std::ptr::null_mut();
            let path = pool.string(path)?;
            let error = ffi::svn_client_url_from_path2(
                url.pointer_mut(),
                path,
                self.ctx(),
                pool.as_mut_ptr(),
                pool.as_mut_ptr(),
            );
            SVNError::from_nullable_ptr(error).context(builder::Svn)?;

            Ok(url.to_str().to_string())
        }
    }

    pub fn update(&mut self, opts: UpdateOptions) -> error::Result<Vec<Option<RevisionNumber>>> {
        unsafe {
            let mut pool = apr::Pool::create();
            let paths = pool.string_array(opts.paths.len(), opts.paths.iter())?;
            let revision = opts.revision.to_opt_revision();
            let depth = opts.depth.into();
            let mut revisions: *mut ffi::apr_array_header_t = std::ptr::null_mut();
            let error = ffi::svn_client_update4(
                revisions.pointer_mut(),
                paths,
                revision.pointer(),
                depth,
                opts.depth_is_sticky.into(),
                opts.ignore_externals.into(),
                opts.allow_unver_obstructions.into(),
                opts.adds_as_modification.into(),
                opts.make_parents.into(),
                self.ctx(),
                pool.as_mut_ptr(),
            );
            SVNError::from_nullable_ptr(error).context(builder::Svn)?;
            assert!(!revisions.is_null(), "Expected non-null revisions");

            use apr::AprArray;

            Ok(revisions.to_value_vec(|ptr| (*(ptr as *const ffi::svn_revnum_t)).try_into().ok()))
        }
    }

    fn property_get(&mut self, opts: PropertyGetOptions) {}

    pub fn cleanup(&mut self, opts: CleanupOptions) -> error::Result<()> {
        unsafe {
            let mut pool = apr::Pool::create();

            let path = pool.string(opts.path)?;

            let path = ffi::svn_dirent_canonicalize(path, pool.as_mut_ptr());

            let error = ffi::svn_client_cleanup2(
                path,
                opts.break_locks.into(),
                opts.fix_recorded_timestamps.into(),
                opts.clear_dav_cache.into(),
                opts.vacuum_pristines.into(),
                opts.include_externals.into(),
                self.ctx(),
                pool.as_mut_ptr(),
            );
            SVNError::from_nullable_ptr(error).context(builder::Svn)?;
        }

        Ok(())
    }

    pub fn list(&mut self, opts: ListOptions) -> error::Result<ListResult> {
        unsafe extern "C" fn list_receiver(
            baton: *mut c_void,
            path: *const c_char,
            dirent: *const ffi::svn_dirent_t,
            lock: *const ffi::svn_lock_t,
            abs_path: *const c_char,
            external_parent_url: *const c_char,
            external_target: *const c_char,
            _scratch_pool: *mut apr::ffi::apr_pool_t,
        ) -> *mut ffi::svn_error_t {
            unsafe {
                let error = on_cancel(baton);
                if !error.is_null() {
                    return error;
                }

                let this = (baton as *mut ContextInner).as_mut().unwrap();
                let opts = this.current_list_options.as_ref().unwrap();
                let path = path.to_str().to_string();
                let lock = lock.map(Lock::from);
                let absolute_path = abs_path.to_str().to_string();
                let external_parent_url = external_parent_url.to_nullable_string();
                let external_target = external_target.to_nullable_string();
                let mut kind = None;
                let mut size = None;
                let mut has_properties = None;
                let mut created_revision = None;
                let mut time = None;
                let mut last_author = None;

                if !dirent.is_null() {
                    let dirent = dirent.as_ref().unwrap();
                    if opts.dirent_kind {
                        kind = Some(NodeKind::try_from(dirent.kind).unwrap());
                    }
                    if opts.dirent_size {
                        size = dirent.size.try_into().ok();
                    }
                    if opts.dirent_has_properties {
                        has_properties = Some(dirent.has_props != 0);
                    }
                    if opts.dirent_created_revision {
                        created_revision = Some(dirent.created_rev.try_into().unwrap());
                    }
                    if opts.dirent_time {
                        time = Some(dirent.time);
                    }
                    if opts.dirent_last_author {
                        last_author = Some(dirent.last_author.to_str().to_string())
                    }
                }

                let entry = ListEntry {
                    path,
                    kind,
                    size,
                    has_properties,
                    created_revision,
                    time,
                    last_author,
                    lock,
                    absolute_path,
                    external_parent_url,
                    external_target,
                };

                // tracing::info!("List entry: {:#?}", entry);

                this.list_entries.push(entry);
            }
            svn_no_error()
        }
        unsafe {
            let mut pool = apr::Pool::create();

            let path = pool.string(opts.path.as_str())?;

            let peg_revision = opts.revision.to_opt_revision();

            let revision = opts.revision.to_opt_revision();

            // let patterns = pool.string_array(opts.patterns.len(), opts.patterns.iter())?;
            //
            let patterns = opts
                .patterns
                .as_ref()
                .map(|e| pool.string_array(e.len(), e.iter()))
                .transpose()?
                .unwrap_or_default();

            let mut dirent: u32 = 0;
            if opts.dirent_created_revision {
                dirent |= ffi::SVN_DIRENT_CREATED_REV;
            }
            if opts.dirent_has_properties {
                dirent |= ffi::SVN_DIRENT_HAS_PROPS;
            }
            if opts.dirent_kind {
                dirent |= ffi::SVN_DIRENT_KIND;
            }
            if opts.dirent_last_author {
                dirent |= ffi::SVN_DIRENT_LAST_AUTHOR;
            }
            if opts.dirent_size {
                dirent |= ffi::SVN_DIRENT_SIZE;
            }

            if opts.dirent_size {
                dirent |= ffi::SVN_DIRENT_TIME;
            }

            self.inner.current_list_options = Some(opts.clone());
            tracing::info!("Call list4: {:#?}", opts);
            let error = ffi::svn_client_list4(
                path,
                peg_revision.pointer(),
                revision.pointer(),
                patterns,
                opts.depth.into(),
                dirent,
                opts.fetch_locks.into(),
                opts.include_externals.into(),
                Some(list_receiver),
                self.inner.inner_void_pointer_mut(),
                self.ctx(),
                pool.as_mut_ptr(),
            );
            tracing::info!("Finish list4:");
            SVNError::from_nullable_ptr(error).context(builder::Svn)?;
        }
        Ok(ListResult {
            entries: std::mem::take(&mut self.inner.list_entries),
        })
    }

    pub fn conflict_walk(
        &mut self,
        opts: ConflictWalkOptions,
    ) -> error::Result<ConflictWalkResult> {
        // struct Baton {
        //     ctx: *mut ffi::svn_client_ctx_t,
        //     inner: *mut ContextInner,
        // }

        unsafe extern "C" fn conflict_walker(
            baton: *mut c_void,
            conflict: *mut ffi::svn_client_conflict_t,
            scratch_pool: *mut ffi::apr_pool_t,
        ) -> *mut ffi::svn_error_t {
            let mut is_text: ffi::svn_boolean_t = 0;
            let mut is_tree: ffi::svn_boolean_t = 0;
            let mut property_conflict: *mut ffi::apr_array_header_t = std::ptr::null_mut();

            unsafe {
                let inner = (baton as *mut ContextInner).as_mut().unwrap();

                let error = ffi::svn_client_conflict_get_conflicted(
                    is_text.pointer_mut(),
                    property_conflict.pointer_mut(),
                    is_tree.pointer_mut(),
                    conflict,
                    scratch_pool,
                    scratch_pool,
                );
                if !error.is_null() {
                    return error;
                }
                if is_text == 0 && is_tree == 0 && property_conflict.is_null() {
                    tracing::info!("Unspecified conflict");
                    return svn_no_error();
                }

                let info = ConflictInfo::from(conflict);

                if is_text != 0 {
                    // let mut options: *mut ffi::apr_array_header_t = std::ptr::null_mut();
                    // let error = ffi::svn_client_conflict_text_get_resolution_options(options.pointer_mut(), conflict, ctx, scratch_pool, scratch_pool);
                    // if !error.is_null() {
                    //     return error;
                    // }

                    let mut base_absolute_path: *const c_char = std::ptr::null_mut();
                    let mut working_absolute_path: *const c_char = std::ptr::null_mut();
                    let mut incoming_old_absolute_path: *const c_char = std::ptr::null_mut();
                    let mut incoming_new_absolute_path: *const c_char = std::ptr::null_mut();

                    let error = ffi::svn_client_conflict_text_get_contents(
                        base_absolute_path.pointer_mut(),
                        working_absolute_path.pointer_mut(),
                        incoming_old_absolute_path.pointer_mut(),
                        incoming_new_absolute_path.pointer_mut(),
                        conflict,
                        scratch_pool,
                        scratch_pool,
                    );
                    if !error.is_null() {
                        return error;
                    }

                    let base_absolute_path = base_absolute_path.to_nullable_string();
                    let working_absolute_path = working_absolute_path.to_nullable_string();
                    let incoming_old_absolute_path =
                        incoming_old_absolute_path.to_nullable_string();
                    let incoming_new_absolute_path =
                        incoming_new_absolute_path.to_nullable_string();

                    inner.conflicts.push(Conflict::Text {
                        base_absolute_path,
                        working_absolute_path,
                        incoming_old_absolute_path,
                        incoming_new_absolute_path,
                        info,
                    });
                } else if !property_conflict.is_null() {
                    let property_names = property_conflict.to_vec(|ptr| ptr);

                    let mut properties = HashMap::new();

                    for i in property_names {
                        let mut base_value: *const ffi::svn_string_t = std::ptr::null_mut();
                        let mut working_value: *const ffi::svn_string_t = std::ptr::null_mut();
                        let mut incoming_old_value: *const ffi::svn_string_t = std::ptr::null_mut();
                        let mut incoming_new_value: *const ffi::svn_string_t = std::ptr::null_mut();

                        let error = ffi::svn_client_conflict_prop_get_propvals(
                            base_value.pointer_mut(),
                            working_value.pointer_mut(),
                            incoming_old_value.pointer_mut(),
                            incoming_new_value.pointer_mut(),
                            conflict,
                            i,
                            scratch_pool,
                        );
                        if !error.is_null() {
                            return error;
                        }
                        let base_value = base_value.to_nullable_string();
                        let working_value = working_value.to_nullable_string();
                        let incoming_old_value = incoming_old_value.to_nullable_string();
                        let incoming_new_value = incoming_new_value.to_nullable_string();

                        properties.insert(
                            i.to_str().to_string(),
                            ConflictPropertyValue {
                                base_value,
                                working_value,
                                incoming_old_value,
                                incoming_new_value,
                            },
                        );
                    }
                    let mut description: *const c_char = std::ptr::null_mut();
                    let error = ffi::svn_client_conflict_prop_get_description(
                        description.pointer_mut(),
                        conflict,
                        scratch_pool,
                        scratch_pool,
                    );
                    if !error.is_null() {
                        return error;
                    }

                    let description = description.to_nullable_string();

                    inner.conflicts.push(Conflict::Property {
                        properties,
                        description,
                        info,
                    });
                } else if is_tree != 0 {
                    let victim_node_kind =
                        ffi::svn_client_conflict_tree_get_victim_node_kind(conflict)
                            .try_into()
                            .unwrap();

                    inner.conflicts.push(Conflict::Tree {
                        info,
                        victim_node_kind,
                    });
                }
            }
            svn_no_error()
        }
        unsafe {
            let mut pool = apr::Pool::create();
            let local_absolute_path = pool.string(opts.local_absolute_path)?;
            let error = ffi::svn_client_conflict_walk(
                local_absolute_path,
                opts.depth.into(),
                Some(conflict_walker),
                self.inner.inner_void_pointer_mut(),
                self.ctx(),
                pool.as_mut_ptr(),
            );
            SVNError::from_nullable_ptr(error).context(builder::Svn)?;
        };

        Ok(ConflictWalkResult {
            conflicts: std::mem::take(&mut self.inner.conflicts),
        })
    }

    pub fn default_wc_version(&mut self) -> error::Result<Version> {
        unsafe {
            let mut pool = apr::Pool::create();
            let mut version: *const ffi::svn_version_t = std::ptr::null();
            let error = ffi::svn_client_default_wc_version(
                version.pointer_mut(),
                self.ctx(),
                pool.as_mut_ptr(),
                pool.as_mut_ptr(),
            );
            SVNError::from_nullable_ptr(error).context(builder::Svn)?;

            Ok(Version::from(version))
        }
    }

    pub fn get_wc_root(&mut self, path: String) -> error::Result<String> {
        unsafe {
            let mut pool = apr::Pool::create();
            let path = pool.string(path)?;

            let mut absolute_path: *const c_char = std::ptr::null();

            let error =
                ffi::svn_dirent_get_absolute(absolute_path.pointer_mut(), path, pool.as_mut_ptr());
            SVNError::from_nullable_ptr(error).context(builder::Svn)?;

            let mut root: *const c_char = std::ptr::null_mut();
            let error = ffi::svn_client_get_wc_root(
                root.pointer_mut(),
                absolute_path,
                self.ctx(),
                pool.as_mut_ptr(),
                pool.as_mut_ptr(),
            );
            SVNError::from_nullable_ptr(error).context(builder::Svn)?;

            let root = root.to_str().to_string();
            Ok(root)
        }
    }

    pub fn switch(&mut self, opts: SwitchOptions) -> error::Result<RevisionNumber> {
        unsafe {
            let mut pool = apr::Pool::create();
            let path = pool.string(opts.path)?;
            let url = pool.string(opts.url)?;
            let peg_revision = opts.peg_revision.to_opt_revision();
            let revision = opts.revision.to_opt_revision();
            let depth = opts.depth.into();
            let depth_is_sticky = opts.depth_is_sticky.into();
            let ignore_externals = opts.ignore_externals.into();
            let allow_unversioned_obstructions = opts.allow_unversioned_obstructions.into();
            let ignore_ancestry = opts.ignore_ancestry.into();
            let mut result: ffi::svn_revnum_t = 0;
            let error = ffi::svn_client_switch3(
                result.pointer_mut(),
                path,
                url,
                peg_revision.pointer(),
                revision.pointer(),
                depth,
                depth_is_sticky,
                ignore_externals,
                allow_unversioned_obstructions,
                ignore_ancestry,
                self.ctx(),
                pool.as_mut_ptr(),
            );
            SVNError::from_nullable_ptr(error).context(builder::Svn)?;

            Ok(result.try_into().expect("Unexpected revision"))
        }
    }

    pub fn relocate(&mut self, opts: RelocateOptions) -> error::Result<()> {
        unsafe {
            let mut pool = apr::Pool::create();
            let root = pool.string(opts.wc_root_dir)?;
            let from = pool.string(opts.from_prefix)?;
            let to = pool.string(opts.to_prefix)?;
            let error = ffi::svn_client_relocate2(
                root,
                from,
                to,
                opts.ignore_externals.into(),
                self.ctx(),
                pool.as_mut_ptr(),
            );
            SVNError::from_nullable_ptr(error).context(builder::Svn)?;
            Ok(())
        }
    }

    pub fn merge(&mut self, opts: MergeOptions) -> error::Result<()> {
        unsafe {
            let mut pool = apr::Pool::create();

            let source1 = pool.string(opts.source1)?;
            let revision1 = opts.revision1.to_opt_revision();

            let source2 = pool.string(opts.source2)?;
            let revision2 = opts.revision2.to_opt_revision();

            let target = pool.string(opts.target)?;

            let merge_options = opts
                .extra_merge_options
                .map(|p| pool.string_array(p.len(), p.iter()))
                .transpose()?
                .unwrap_or_default();

            let error = ffi::svn_client_merge5(
                source1,
                revision1.pointer(),
                source2,
                revision2.pointer(),
                target,
                opts.depth.into(),
                opts.ignore_merge_info.into(),
                opts.ignore_ancestry.into(),
                opts.force_delete.into(),
                opts.record_only.into(),
                opts.dry_run.into(),
                opts.allow_mixed_revision.into(),
                merge_options,
                self.ctx(),
                pool.as_mut_ptr(),
            );

            SVNError::from_nullable_ptr(error).context(builder::Svn)?;

            Ok(())
        }
    }

    pub fn difference(
        &mut self,
        opts: ClientDifferenceOptions,
    ) -> error::Result<ClientDifferenceResult> {
        unsafe {
            let mut pool = apr::Pool::create();

            let options = opts
                .options
                .map(|p| pool.string_array(p.len(), p.iter()))
                .transpose()?
                .unwrap_or_default();
            let path1 = pool.string(opts.path1)?;
            let revision1 = pool.revision(opts.revision1);
            let path2 = pool.string(opts.path2)?;
            let revision2 = pool.revision(opts.revision2);

            let relate_to = opts
                .relate_to
                .as_ref()
                .map(|v| pool.string(v))
                .transpose()?
                .unwrap_or_default();

            let header_encoding = pool.string(opts.header_encoding)?;

            let changelists = opts
                .changelists
                .map(|p| pool.string_array(p.len(), p.iter()))
                .transpose()?
                .unwrap_or_default();

            let mut out = Stream::create(Default::default());
            let mut err = Stream::create(Default::default());

            let error = ffi::svn_client_diff7(
                options,
                path1,
                revision1,
                path2,
                revision2,
                relate_to,
                opts.depth.into(),
                opts.ignore_ancestry.into(),
                opts.no_added.into(),
                opts.no_deleted.into(),
                opts.show_copies_as_adds.into(),
                opts.ignore_content_type.into(),
                opts.ignore_properties.into(),
                opts.properties_only.into(),
                opts.use_git_format.into(),
                opts.pretty_print_merge_info.into(),
                header_encoding,
                out.ptr(),
                err.ptr(),
                changelists,
                self.ctx(),
                pool.as_mut_ptr(),
            );

            SVNError::from_nullable_ptr(error).context(builder::Svn)?;

            let out = out.take_write_buffer();
            let err = err.take_write_buffer();

            Ok(ClientDifferenceResult { out, err })
        }
    }

    pub fn patch(&mut self, opts: PatchOptions) -> error::Result<()> {
        unsafe extern "C" fn patch_filter(
            baton: *mut c_void,
            filtered: *mut ffi::svn_boolean_t,
            canon_path_from_patchfile: *const c_char,
            patch_abspath: *const c_char,
            reject_abspath: *const c_char,
            scratch_pool: *mut ffi::apr_pool_t,
        ) -> *mut ffi::svn_error_t {
            svn_no_error()
        }
        unsafe {
            let mut pool = apr::Pool::create();

            let patch = pool.string(opts.patch_absolute_path)?;

            let wc = pool.string(opts.wc_absolute_path)?;

            let error = ffi::svn_client_patch(
                wc,
                patch,
                opts.dry_run.into(),
                opts.strip_count.try_into().unwrap(),
                opts.reverse.into(),
                opts.ignore_whitespace.into(),
                opts.remove_tempfiles.into(),
                Some(patch_filter),
                self.inner.inner_void_pointer_mut(),
                self.ctx(),
                pool.as_mut_ptr(),
            );

            SVNError::from_nullable_ptr(error).context(builder::Svn)?;
            Ok(())
        }
    }

    pub fn get_repository_root(
        &mut self,
        target: String,
    ) -> error::Result<GetRepositoryRootResult> {
        unsafe {
            let mut pool = apr::Pool::create();
            let target = pool.string(target)?;

            let mut absolute_path: *const c_char = std::ptr::null();

            let error = ffi::svn_dirent_get_absolute(
                absolute_path.pointer_mut(),
                target,
                pool.as_mut_ptr(),
            );
            SVNError::from_nullable_ptr(error).context(builder::Svn)?;

            let mut root_url: *const c_char = std::ptr::null_mut();
            let mut uuid: *const c_char = std::ptr::null_mut();

            let error = ffi::svn_client_get_repos_root(
                root_url.pointer_mut(),
                uuid.pointer_mut(),
                absolute_path,
                self.ctx(),
                pool.as_mut_ptr(),
                pool.as_mut_ptr(),
            );

            SVNError::from_nullable_ptr(error).context(builder::Svn)?;

            let root_url = root_url.to_str().to_string();

            let uuid = uuid.to_str().to_string();

            Ok(GetRepositoryRootResult { root_url, uuid })
        }
    }

    pub fn lock(&mut self, opts: LockOptions) -> error::Result<()> {
        unsafe {
            let mut pool = apr::Pool::create();
            let targets = pool.string_array(opts.targets.len(), opts.targets.iter())?;

            let comment = opts
                .comment
                .map(|v| pool.string(v))
                .transpose()?
                .unwrap_or_default();

            let error = ffi::svn_client_lock(
                targets,
                comment,
                opts.steal_lock.into(),
                self.ctx(),
                pool.as_mut_ptr(),
            );
            SVNError::from_nullable_ptr(error).context(builder::Svn)?;
        }
        Ok(())
    }
    pub fn unlock(&mut self, opts: UnlockOptions) -> error::Result<()> {
        unsafe {
            let mut pool = apr::Pool::create();
            let targets = pool.string_array(opts.targets.len(), opts.targets.iter())?;

            let error = ffi::svn_client_unlock(
                targets,
                opts.break_lock.into(),
                self.ctx(),
                pool.as_mut_ptr(),
            );
            SVNError::from_nullable_ptr(error).context(builder::Svn)?;
        }
        Ok(())
    }

    pub fn open_repository_access_session(
        &mut self,
        url: String,
        path: Option<String>,
    ) -> error::Result<i32> {
        unsafe {
            let mut pool = apr::Pool::create();
            let url = pool.string(url)?;
            let path = path
                .map(|v| pool.string(v))
                .transpose()?
                .unwrap_or_default();
            let mut session: *mut ffi::svn_ra_session_t = std::ptr::null_mut();
            let mut session_pool = apr::Pool::create();
            let error = ffi::svn_client_open_ra_session2(
                session.pointer_mut(),
                url,
                path,
                self.ctx(),
                session_pool.as_mut_ptr(),
                pool.as_mut_ptr(),
            );
            SVNError::from_nullable_ptr(error).context(builder::Svn)?;

            let session = AutoPool {
                pool: session_pool,
                value: session,
            };

            let key = self
                .ra_sessions
                .keys()
                .map(|e| *e)
                .max()
                .unwrap_or_default()
                + 1;

            self.ra_sessions.insert(key, session);

            Ok(key)
        }
    }

    fn create(opts: CreateContextOptions) -> error::Result<Self> {
        let mut pool = unsafe { apr::Pool::create() };

        let mut ptr: *mut ffi::svn_client_ctx_t = std::ptr::null_mut();

        let config = unsafe { pool.read_subversion_config(None)? };

        unsafe { opts.config.apply(config, &mut pool)? };

        unsafe {
            let error =
                ffi::svn_client_create_context2(&mut ptr as *mut _, config, pool.as_mut_ptr());
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
            ffi::svn_auth_get_simple_provider2(
                &mut provider as *mut _,
                Some(may_save_password_as_plain_text),
                &mut *inner as *mut _ as *mut c_void,
                pool.as_mut_ptr(),
            );
            let ptr = ffi::apr_array_push(array) as *mut *mut ffi::svn_auth_provider_object_t;
            *ptr = provider;

            ffi::svn_auth_get_username_provider(&mut provider as *mut _, pool.as_mut_ptr());
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
                1000000,
                pool.as_mut_ptr(),
            );
            let ptr = ffi::apr_array_push(array) as *mut *mut ffi::svn_auth_provider_object_t;
            *ptr = provider;
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

        unsafe extern "C" fn conflict(
            result: *mut *mut ffi::svn_wc_conflict_result_t,
            description: *const ffi::svn_wc_conflict_description2_t,
            baton: *mut c_void,
            result_pool: *mut ffi::apr_pool_t,
            scratch_pool: *mut ffi::apr_pool_t,
        ) -> *mut ffi::svn_error_t {
            unsafe {
                let this = (baton as *mut ContextInner).as_mut().unwrap();

                let description = WorkingCopyConflictDescription::from(description);

                let user_result = this.context_notifier.conflict(description);
                let user_result = match user_result {
                    Ok(v) => v,
                    Err(e) => return e.native_error(),
                };

                let merged_file = user_result
                    .merged_file
                    .map(|e| result_pool.string(e).expect("Invalid file"))
                    .unwrap_or_default();

                *result = ffi::svn_wc_create_conflict_result(
                    user_result.choise.into(),
                    merged_file,
                    result_pool,
                );

                let result = (*result).as_mut().unwrap();

                result.choice = user_result.choise.into();
                result.merged_value = user_result
                    .merged_value
                    .map(|e| {
                        ffi::svn_string_ncreate(
                            e.as_ptr() as _,
                            e.len().try_into().unwrap(),
                            result_pool,
                        )
                    })
                    .unwrap_or_default();

                result.save_merged = user_result.save_merged.into();
            }

            svn_no_error()
        }

        unsafe {
            let ctx = ptr.as_mut().unwrap();
            ctx.auth_baton = auth_baton;
            ctx.log_msg_func3 = Some(on_get_commit_message);
            ctx.log_msg_baton3 = inner.inner_void_pointer_mut();
            ctx.notify_func2 = Some(on_notify);
            ctx.notify_baton2 = inner.inner_void_pointer_mut();
            ctx.cancel_func = Some(on_cancel);
            ctx.cancel_baton = inner.inner_void_pointer_mut();

            ctx.progress_func = Some(on_progress_notify);
            ctx.progress_baton = inner.inner_void_pointer_mut();

            ctx.conflict_func2 = Some(conflict);
            ctx.conflict_baton2 = inner.inner_void_pointer_mut();

            if let Some(ref name) = opts.name {
                ctx.client_name = pool.string(name)?;
            }
        }

        Ok(Self {
            ptr,
            pool,
            inner,
            config,
            ra_sessions: Default::default(),
        })
    }
}
