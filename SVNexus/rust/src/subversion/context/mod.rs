pub mod conflict;
pub mod import;
pub mod info;
pub mod list;
pub mod log;
pub mod patch;
pub mod status;

pub use conflict::{Conflict, ConflictWalkOptions, ConflictWalkResult};
pub use import::{ImportOptions, ImportResult};
pub use info::{InfoEntry, InfoOptions, InfoResult};
pub use list::{ListEntry, ListOptions, ListResult};
pub use log::{LogChangedPathEntry, LogEntry, LogOptions, LogReceiver, LogResult};
pub use patch::PatchOptions;
pub use status::{StatusEntry, StatusOptions, StatusReceiver, StatusResult};

use super::ffi;
use super::stream::Stream;
use super::wc::*;
use super::{SVNError, svn_no_error};
use crate::apr::{self, AprArray, AprPool, AutoPool};
use crate::error::{self, CSharpError, CSharpErrorExtension, builder};
use crate::extensions::{Canonicalization, OptionExtension};
use crate::subversion::utils;
use crate::subversion::version::Version;
use crate::utils::PointerMutMapper;
use crate::utils::SubversionStringer;
use crate::utils::{Boxed, CStringer};
use crate::utils::{Pointer, PointerMapper};
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
        f.debug_struct("Context").finish()
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
    info: Option<CommitInfo>,
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
    #[uniffi(default)]
    commit_message: String,
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
    path: Option<String>,
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
            let path = lock.path.to_nullable_string();

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

impl Default for WorkingCopyStatus {
    fn default() -> Self {
        Self::None
    }
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

// unsafe extern "C" fn first_ssl_client_cert_pw() -> *mut ffi::svn_error_t {
//     svn_no_error()
// }

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
    inherited: bool,
    actual_revision: bool,
    #[uniffi(default)]
    changelists: Option<Vec<String>>,
}

#[derive(Debug, uniffi::Record)]
pub struct PropertyGetResult {
    properties: HashMap<String, String>,
    inherited_properties: Option<Vec<InheritedProperty>>,
    actual_revision: Option<RevisionNumber>,
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
    revision_table: Option<HashMap<String, String>>,
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
                // tracing::info!("output_common");
                // tracing::info!("+==============");
                // tracing::info!("original_start: {}", original_start);
                // tracing::info!("original_length: {}", original_length);
                // tracing::info!("modified_start: {}", modified_start);
                // tracing::info!("modified_length: {}", modified_length);
                // tracing::info!("latest_start: {}", latest_start);
                // tracing::info!("latest_length: {}", latest_length);
                // tracing::info!("-==============");

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
                // tracing::info!("output_diff_modified");
                // tracing::info!("*==============");
                // tracing::info!("original_start: {}", original_start);
                // tracing::info!("original_length: {}", original_length);
                // tracing::info!("modified_start: {}", modified_start);
                // tracing::info!("modified_length: {}", modified_length);
                // tracing::info!("latest_start: {}", latest_start);
                // tracing::info!("latest_length: {}", latest_length);
                // tracing::info!("/==============");

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
                // tracing::info!("output_diff_common");
                // tracing::info!("(==============");
                // tracing::info!("original_start: {}", original_start);
                // tracing::info!("original_length: {}", original_length);
                // tracing::info!("modified_start: {}", modified_start);
                // tracing::info!("modified_length: {}", modified_length);
                // tracing::info!("latest_start: {}", latest_start);
                // tracing::info!("latest_length: {}", latest_length);
                // tracing::info!(")==============");

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
                // tracing::info!("output_diff_latest");
                // tracing::info!("[==============");
                // tracing::info!("original_start: {}", original_start);
                // tracing::info!("original_length: {}", original_length);
                // tracing::info!("modified_start: {}", modified_start);
                // tracing::info!("modified_length: {}", modified_length);
                // tracing::info!("latest_start: {}", latest_start);
                // tracing::info!("latest_length: {}", latest_length);
                // tracing::info!("]==============");

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
pub struct GetRepositoryRootResult {
    pub root_url: String,
    pub uuid: String,
}

#[derive(uniffi::Record)]
pub struct InitializeRepositoryOptions {
    pub local: String,
    pub remote: String,
    pub backup_directory: Option<String>,
    pub commit_message: String,
    pub ignore_unknown_node_types: bool,
    pub no_ignore: bool,
    pub no_autoprops: bool,
    pub filters: Option<Vec<String>>,
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

#[derive(Debug, uniffi::Enum)]
pub enum PropertySetOptions {
    Local {
        name: String,
        value: Option<String>,
        targets: Vec<String>,
        depth: Depth,
        skip_checks: bool,
        #[uniffi(default)]
        changelists: Option<Vec<String>>,
    },
    Remote {
        name: String,
        value: Option<String>,
        url: String,
        skip_checks: bool,
        base_revision_for_url: RevisionNumber,
        #[uniffi(default)]
        revision_properties: Option<HashMap<String, String>>,
        commit_message: String,
    },
}

impl Context {
    pub fn ctx(&mut self) -> *mut ffi::svn_client_ctx_t {
        self.ptr
    }

    fn cancelled(&mut self) -> error::Result<()> {
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

            let paths = pool.canonicalize_target_array(opts.paths.len(), opts.paths.iter())?;

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
            let target = pool.canonicalize_target(&opts.target)?;

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
                no_ignore: opts.no_ignore,
                no_autoprops: opts.no_autoprops,
                ignore_unknown_node_types: opts.ignore_unknown_node_types,
                revision_property_table: Default::default(),
                commit_message: opts.commit_message,
                filters: opts.filters,
            };
            self.import(import_optios)?;
            self.cancelled()?;
            if let Some(directory) = opts.backup_directory {
                notifier.on_backup()?;
                let file = utils::backup(
                    &opts.local,
                    if directory.is_empty() {
                        None
                    } else {
                        Some(directory)
                    },
                )?;
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

    fn take_commit_result(&mut self) -> CommitResult {
        let items = std::mem::take(&mut self.inner.commit_items);

        let info = std::mem::take(&mut self.inner.commit_info);

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

            let from_path_or_url = opts.from_path_or_url.replace('\\', "/");

            let mut from_path_or_url = pool.string(&from_path_or_url)? as *const c_char;
            let to_path = pool.string(opts.to_path)?;
            let to_path = ffi::svn_dirent_canonicalize(to_path, pool.as_mut_ptr());
            let peg_revision = opts.peg_revision.to_opt_revision();
            let revision = opts.revision.to_opt_revision();

            if ffi::svn_path_is_url(from_path_or_url) != 0 {
                from_path_or_url =
                    Self::check_url(from_path_or_url, &opts.from_path_or_url, &mut pool)?;
            }

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

    unsafe fn check_url(
        url: *const c_char,
        orignal_url: &str,
        pool: &mut apr::Pool,
    ) -> error::Result<*const c_char> {
        unsafe {
            if ffi::svn_path_is_url(url) == 0 {
                return builder::InvalidArgument {
                    detail: format!("{} is not a url", orignal_url),
                }
                .fail();
            }

            let target = ffi::svn_path_uri_from_iri(url, pool.as_mut_ptr());

            #[cfg(target_os = "windows")]
            let mut target = ffi::svn_path_uri_autoescape(target, pool.as_mut_ptr());

            #[cfg(not(target_os = "windows"))]
            let target = ffi::svn_path_uri_autoescape(target, pool.as_mut_ptr());

            #[cfg(target_os = "windows")]
            {
                let mut p = ffi::apr_pstrdup(pool.as_mut_ptr(), target);
                target = p;

                while *p as u8 != b'\0' {
                    if *p as u8 == b'\\' {
                        *p = b'/' as c_char;
                    }
                    p = p.add(1);
                }
            }

            if ffi::svn_path_is_backpath_present(target) != 0 {
                return builder::InvalidArgument {
                    detail: format!(".. is not allowed in url: {}", orignal_url),
                }
                .fail();
            }

            let url = ffi::svn_uri_canonicalize(target, pool.as_mut_ptr());

            if ffi::svn_uri_is_canonical(url, pool.as_mut_ptr()) == 0 {
                return builder::InvalidArgument {
                    detail: format!("{} is invalid", orignal_url),
                }
                .fail();
            }

            Ok(url)
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

        let mut url = unsafe { pool.string(opts.url.as_str())? } as *const c_char;

        let path = unsafe { pool.string(opts.path.as_str()) }?;

        // let url = CString::new(opts.url)
        //     .ok()
        //     .context(builder::InvalidArgument {
        //         detail: "Invalid url",
        //     })?;

        // let path = CString::new(opts.path).whatever_context::<_, error::Error>("Invalid path")?;

        if opts.url.contains('\\') {}

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
            url = Self::check_url(url, &opts.url, &mut pool)?;
            // if ffi::svn_path_is_url(url) == 0 {
            //     return builder::InvalidArgument {
            //         detail: format!("{} is not a url", opts.url),
            //     }
            //     .fail();
            // }

            // {
            //     let target = ffi::svn_path_uri_from_iri(url, pool.as_mut_ptr());

            //     #[cfg(target_os = "windows")]
            //     let mut target = ffi::svn_path_uri_autoescape(target, pool.as_mut_ptr());

            //     #[cfg(not(target_os = "windows"))]
            //     let target = ffi::svn_path_uri_autoescape(target, pool.as_mut_ptr());

            //     #[cfg(target_os = "windows")]
            //     {
            //         let mut p = ffi::apr_pstrdup(pool.as_mut_ptr(), target);
            //         target = p;

            //         while *p as u8 != b'\0' {
            //             if *p as u8 == b'\\' {
            //                 *p = b'/' as c_char;
            //             }
            //             p = p.add(1);
            //         }
            //     }

            //     if svn_path_is_backpath_present(target) != 0 {
            //         return builder::InvalidArgument {
            //             detail: ".. is not allowed in url",
            //         }.fail();
            //     }

            //     url = svn_uri_canonicalize(target, pool.as_mut_ptr());
            // }

            // if ffi::svn_uri_is_canonical(url, pool.as_mut_ptr()) == 0 {
            //     return builder::InvalidArgument {
            //         detail: format!("{} is invalid", opts.url),
            //     }
            //     .fail();
            // }

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

            let path = pool.canonicalize_dirent(opts.path.as_str())?;

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
            let targets =
                pool.canonicalize_dirent_array(opts.targets.len(), opts.targets.iter())?;

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

            let path = pool.canonicalize_target_array(opts.path.len(), opts.path.iter())?;

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

            self.inner.commit_message = opts.commit_message.clone();

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
                pool.canonicalize_dirent_array(opts.paths.len(), opts.paths.iter())?,
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

    pub fn cat(&mut self, opts: CatOptions) -> error::Result<CatResult> {
        unsafe {
            let mut properties: *mut ffi::apr_hash_t = std::ptr::null_mut();

            let mut pool = apr::Pool::create();

            let mut stream = Stream::create(Default::default());

            let path = pool.canonicalize_target(&opts.path)?;

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
            let paths = pool.canonicalize_dirent_array(opts.paths.len(), opts.paths.iter())?;
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

    pub fn property_get(&mut self, opts: PropertyGetOptions) -> error::Result<PropertyGetResult> {
        unsafe {
            let mut pool = apr::Pool::create();

            let mut inherited_properties: *mut ffi::apr_array_header_t = std::ptr::null_mut();

            let mut properties: *mut ffi::apr_hash_t = std::ptr::null_mut();

            let property_name = pool.string(opts.property_name.as_str())?;

            let target = pool.string(opts.target.as_str())?;

            let peg_revision = opts.peg_revision.to_opt_revision();

            let revision = opts.revision.to_opt_revision();

            let mut actual_revision: ffi::svn_revnum_t = 0;

            let changelist = opts
                .changelists
                .map(|v| pool.string_array(v.len(), v.iter()))
                .transpose()?
                .unwrap_or_default();

            let error = ffi::svn_client_propget5(
                properties.pointer_mut(),
                if opts.inherited {
                    inherited_properties.pointer_mut()
                } else {
                    std::ptr::null_mut()
                },
                property_name,
                target,
                peg_revision.pointer(),
                revision.pointer(),
                if opts.actual_revision {
                    actual_revision.pointer_mut()
                } else {
                    std::ptr::null_mut()
                },
                opts.depth.into(),
                changelist,
                self.ctx(),
                pool.as_mut_ptr(),
                pool.as_mut_ptr(),
            );

            SVNError::from_nullable_ptr(error).context(builder::Svn)?;

            let inherited_properties = inherited_properties.map(|v| {
                v.to_vec(|p| InheritedProperty::from(p as *const ffi::svn_prop_inherited_item_t))
            });

            let actual_revision = if opts.actual_revision {
                Some(actual_revision.try_into().unwrap())
            } else {
                None
            };

            let properties = if properties.is_null() {
                tracing::info!("Unexpected null properties, treat as empty");
                Default::default()
            } else {
                pool.convert_to_hash_map(properties)
            };

            let result = PropertyGetResult {
                properties,
                inherited_properties,
                actual_revision,
            };

            Ok(result)
        }
    }

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
            let path = pool.canonicalize_dirent(&path)?;

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
            let path1 = pool.canonicalize_target(&opts.path1)?;
            let revision1 = pool.revision(opts.revision1);
            let path2 = pool.canonicalize_target(&opts.path2)?;
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

    pub fn get_repository_root(
        &mut self,
        target: String,
    ) -> error::Result<GetRepositoryRootResult> {
        unsafe {
            let mut pool = apr::Pool::create();
            let target = pool.canonicalize_target(&target)?;

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
            let targets =
                pool.canonicalize_target_array(opts.targets.len(), opts.targets.iter())?;

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
            let targets =
                pool.canonicalize_target_array(opts.targets.len(), opts.targets.iter())?;

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
            let url = pool.canonicalize_uri(&url)?;
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

    pub fn property_set(&mut self, opts: PropertySetOptions) -> error::Result<()> {
        unsafe {
            let mut pool = apr::Pool::create();

            match opts {
                PropertySetOptions::Local {
                    name,
                    value,
                    targets,
                    depth,
                    skip_checks,
                    changelists,
                } => {
                    let name = pool.string(name)?;
                    // let value = value.map(|e| pool.string(e)).transpose()?.unwrap_or_default();
                    let value = value
                        .map(|e| {
                            ffi::svn_string_ncreate(
                                e.as_ptr() as _,
                                e.len().try_into().unwrap(),
                                pool.as_mut_ptr(),
                            )
                        })
                        .unwrap_or_default();
                    let targets = pool.canonicalize_dirent_array(targets.len(), targets.iter())?;
                    let changelists = changelists
                        .map(|v| pool.string_array(v.len(), v.iter()))
                        .transpose()?
                        .unwrap_or_default();

                    let error = ffi::svn_client_propset_local(
                        name,
                        value,
                        targets,
                        depth.into(),
                        skip_checks.into(),
                        changelists,
                        self.ctx(),
                        pool.as_mut_ptr(),
                    );
                    SVNError::from_nullable_ptr(error).context(builder::Svn)?;
                }
                PropertySetOptions::Remote {
                    name,
                    value,
                    url,
                    skip_checks,
                    base_revision_for_url,
                    revision_properties,
                    commit_message,
                } => {
                    let name = pool.string(name)?;
                    // let value = value.map(|e| pool.string(e)).transpose()?.unwrap_or_default();
                    let value = value
                        .map(|e| pool.svn_string(e))
                        .transpose()?
                        .unwrap_or_default();
                    let url = pool.canonicalize_uri(&url)?;

                    let revision_properties = revision_properties
                        .map(|e| {
                            pool.string_hash_map(
                                e.iter(),
                                |p: &mut apr::Pool, k: &str| p.string(k).map(|o| o as _),
                                |p: &mut apr::Pool, v: &str| p.svn_string(v).map(|o| o as _),
                            )
                        })
                        .transpose()?
                        .unwrap_or_default();

                    self.inner.commit_message = commit_message;

                    let error = ffi::svn_client_propset_remote(
                        name,
                        value,
                        url,
                        skip_checks.into(),
                        base_revision_for_url.try_into().unwrap(),
                        revision_properties,
                        Some(commit_callback),
                        self.inner.inner_void_pointer_mut(),
                        self.ctx(),
                        pool.as_mut_ptr(),
                    );
                    SVNError::from_nullable_ptr(error).context(builder::Svn)?;
                }
            }
        }
        Ok(())
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
