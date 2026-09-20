pub mod add;
pub mod blame;
pub mod cat;
pub mod checkout;
pub mod cleanup;
pub mod commit;
pub mod conflict;
pub mod copy;
pub mod delete;
pub mod difference;
pub mod export;
pub mod import;
pub mod info;
pub mod list;
pub mod lock;
pub mod log;
pub mod merge;
pub mod mkdir;
pub mod move_;
pub mod patch;
pub mod property;
pub mod relocate;
pub mod revert;
pub mod status;
pub mod switch;
pub mod tunnel;
pub mod update;
pub mod upgrade;
pub mod vacuum;

pub use add::AddOptions;
pub use checkout::CheckoutOptions;
pub use cleanup::CleanupOptions;
pub use commit::{CommitItem, CommitOptions, CommitResult};
pub use conflict::{Conflict, ConflictWalkOptions, ConflictWalkResult};
pub use delete::{DeleteOptions, DeleteResult};
pub use export::ExportOptions;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
pub use import::{ImportOptions, ImportResult};
pub use info::{InfoEntry, InfoOptions, InfoResult};
pub use list::{ListOptions, ListResult};
pub use lock::{LockOptions, UnlockOptions};
pub use mkdir::{MkdirOptions, MkdirResult};
pub use patch::PatchOptions;
pub use property::*;
pub use relocate::RelocateOptions;
pub use status::{StatusEntry, StatusOptions, StatusReceiver, StatusResult};
use surrealdb::types::SurrealValue;
use tempfile::NamedTempFile;
pub use update::UpdateOptions;

use super::ffi;
use super::stream::Stream;
use super::wc::*;
use super::{svn_no_error, SubversionError};
use crate::apr::{self, AprArray, AprPool};
use crate::error::{self, builder, FrontendError, FrontendErrorExtension};
use crate::extensions::{Canonicalization, CommonExtension, OptionExtension, ResultExtension};
use crate::platform;
use crate::subversion::utils;
use crate::subversion::version::Version;
use crate::utils::PointerMutMapper;
use crate::utils::{Boxed, CStringer};
use crate::utils::{Pointer, PointerMapper};
use derive_new::new;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;
use std::collections::HashMap;
use std::ffi::{c_char, c_void, OsString};
use std::io::Write;
use std::mem::ManuallyDrop;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, OnceLock};
use strum::EnumString;

#[derive(Debug)]
pub struct CancelToken {
    code: i32,
    msg: String,
}

pub static GLOBAL_CANCEL_TOKEN: OnceLock<CancelToken> = OnceLock::new();

pub fn set_global_cancel_token(token: CancelToken) -> error::Result<()> {
    GLOBAL_CANCEL_TOKEN
        .set(token)
        .ok()
        .any_context("Global cancel token is already set")?;

    Ok(())
}

pub type RevisionNumber = u32;

#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ClientCertificate {
    file: String,
    save: bool,
}

pub trait ContextNotifier: Send + Sync + 'static {
    fn may_save_password_as_plain_text(&self, realm_string: String) -> Result<bool, FrontendError>;
    fn working_copy_notify(&self, notify: WorkingCopyNotify) -> Result<(), FrontendError>;
    fn ssl_server_trust_prompt(
        &self,
        realm: String,
        failures: u32,
        info: SslServerCertInfo,
        may_save: bool,
    ) -> Result<Option<TrustServer>, FrontendError>;
    fn cancel(&self) -> Result<Option<String>, FrontendError>;
    fn progress_notify(&self, pos: i64, total: i64) -> Result<(), FrontendError>;
    fn authenticate(
        &self,
        realm: String,
        username: String,
        may_save: bool,
        need_password: bool,
    ) -> Result<Option<Authentication>, FrontendError>;
    fn conflict(
        &self,
        description: WorkingCopyConflictDescription,
    ) -> Result<WorkingCopyConflictResult, FrontendError>;
    fn ssl_client_certificate(
        &self,
        realm: String,
        may_save: bool,
    ) -> Result<Option<ClientCertificate>, FrontendError>;
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
    tunnel: HashMap<String, Box<dyn tunnel::Tuunel + Send>>,

    #[new(default)]
    info_entries: HashMap<String, InfoEntry>,

    #[new(default)]
    conflicts: Vec<Conflict>,
}

pub struct Context {
    ptr: *mut ffi::svn_client_ctx_t,
    pool: *mut ffi::apr_pool_t,
    inner: Box<ContextInner>,
    // ra_sessions: HashMap<i32, AutoPool<*mut ffi::svn_ra_session_t>>,
}

impl std::fmt::Debug for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Context").finish()
    }
}

unsafe impl Send for Context {}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            ffi::apr_pool_destroy(self.pool);
        }
    }
}

// impl Context {
//     pub fn ra_session(
//         &mut self,
//         key: i32,
//     ) -> error::Result<&mut AutoPool<*mut ffi::svn_ra_session_t>> {
//         self.ra_sessions
//             .get_mut(&key)
//             .any_context("No such ra session")
//     }

//     pub fn remove_ra_session(&mut self, key: i32) {
//         self.ra_sessions.remove(&key);
//     }
// }

pub struct CreateContextOptions {
    pub name: Option<String>,
    pub default_username: Option<String>,
    pub default_password: Option<String>,
    pub context_notifier: Arc<dyn ContextNotifier>,
    pub config: Config,
}

#[derive(Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
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
                ffi::APR_HASH_KEY_STRING
                    .try_into()
                    .expect("Failed to convert value"),
            )
        };
        if let Some(proxies) = self.proxies.as_ref() {
            unsafe { proxies.apply(servers_config as *mut _, pool)? };
        }
        Ok(())
    }
}

#[derive(new, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
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

#[derive(new, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct Proxy {
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
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
    Serialize,
    Deserialize,
    EnumString,
    strum::Display,
    ts_rs::TS,
    SurrealValue,
)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum NodeKind {
    None = ffi::svn_node_kind_t_svn_node_none,
    File = ffi::svn_node_kind_t_svn_node_file,
    Directory = ffi::svn_node_kind_t_svn_node_dir,
    Unknown = ffi::svn_node_kind_t_svn_node_unknown,
    Symlink = ffi::svn_node_kind_t_svn_node_symlink,
}

// #[repr(i32)]
#[svnexus_macro::enum_converter(repr_type=ffi::svn_depth_t)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ts_rs::TS,
)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum Depth {
    Unknown = ffi::svn_depth_t_svn_depth_unknown,
    Exclude = ffi::svn_depth_t_svn_depth_exclude,
    Empty = ffi::svn_depth_t_svn_depth_empty,
    Files = ffi::svn_depth_t_svn_depth_files,
    Immediates = ffi::svn_depth_t_svn_depth_immediates,
    Infinity = ffi::svn_depth_t_svn_depth_infinity,
}

#[derive(Default, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CommitInfo {
    revision: RevisionNumber,
    date: Option<String>,
    author: Option<String>,
    post_commit_err: Option<String>,
    repos_root: Option<String>,
}

// #[repr(u32)]
// #[derive(Debug, TryFromPrimitive)]
#[svnexus_macro::enum_converter(repr_type=ffi::svn_wc_status_kind)]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
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

#[derive(Clone, Copy, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
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

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Lock {
    path: Option<String>,
    token: String,
    owner: String,
    comment: Option<String>,
    is_dav_comment: bool,
    #[ts(type = "number")]
    creation_date: i64,
    #[ts(type = "number")]
    expiration_date: i64,
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
            let r = ffi::svn_string_ncreate(
                str.as_ptr() as _,
                len.try_into().expect("Failed to convert size"),
                self,
            );
            Ok(r)
        }
    }

    unsafe fn revision(self, value: Revision) -> *const ffi::svn_opt_revision_t {
        unsafe {
            let revision_ptr = self.malloc::<ffi::svn_opt_revision_t>();

            let revision = revision_ptr
                .as_mut()
                .expect("Failed to get mutable reference to revision");

            match value {
                Revision::Unspecified => {
                    revision.kind = ffi::svn_opt_revision_kind_svn_opt_revision_unspecified;
                }
                Revision::Number(number) => {
                    revision.kind = ffi::svn_opt_revision_kind_svn_opt_revision_number;
                    *revision.value.number.as_mut() = number
                        .try_into()
                        .expect("Failed to convert revision number");
                }
                Revision::Date(time) => {
                    revision.kind = ffi::svn_opt_revision_kind_svn_opt_revision_date;
                    *revision.value.date.as_mut() =
                        time.try_into().expect("Failed to convert time");
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
                    *revision.value.number.as_mut() = (*number)
                        .try_into()
                        .expect("Failed to convert revision number");
                }
            }
            Revision::Date(time) => {
                revision.kind = ffi::svn_opt_revision_kind_svn_opt_revision_date;
                unsafe {
                    *revision.value.date.as_mut() =
                        (*time).try_into().expect("Failed to convert time");
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
            let info = info.as_ref().expect("Failed to get reference to info");
            Self {
                revision: info
                    .revision
                    .try_into()
                    .expect("Failed to convert revision number"),
                date: info.date.to_nullable_string(),
                author: info.author.to_nullable_string(),
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

        let this = (baton as *mut ContextInner)
            .as_mut()
            .expect("Failed to cast baton to mutable reference");

        this.commit_info = Some(commit_info);

        svn_no_error()
    }
}

impl From<*const ffi::svn_lock_t> for Lock {
    fn from(ptr: *const ffi::svn_lock_t) -> Self {
        unsafe {
            let lock = ptr.as_ref().expect("Failed to get reference from pointer");
            let path = lock.path.to_nullable_string();

            let token = lock.token.to_str().to_string();

            let owner = lock.owner.to_str().to_string();

            let comment = lock.comment.to_nullable_string();

            let is_dav_comment = lock.is_dav_comment != 0;

            let creation_date: i64 = lock
                .creation_date
                .try_into()
                .expect("Failed to convert time");

            let expiration_date: i64 = lock
                .expiration_date
                .try_into()
                .expect("Failed to convert time");
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

// unsafe fn char_from_pool(string: &str, pool: *mut ffi::apr_pool_t) -> *const c_char {
//     let mut bytes = vec![0; string.len() + 1];
//     bytes[..string.len()].copy_from_slice(string.as_bytes());

//     unsafe { ffi::apr_pstrndup(pool, bytes.as_ptr() as _, bytes.len()) }
// }

unsafe extern "C" fn on_progress_notify(
    progress: ffi::apr_off_t,
    total: ffi::apr_off_t,
    baton: *mut c_void,
    _: *mut ffi::apr_pool_t,
) {
    unsafe {
        let this = &*(baton as *mut ContextInner);
        let result = this.context_notifier.progress_notify(
            progress.try_into().expect("Failed to convert value"),
            total.try_into().expect("Failed to convert value"),
        );
        if let Err(e) = result {
            tracing::error!("Unexpected Error from csharp: {}", e);
        }
    }
}

unsafe extern "C" fn on_cancel(baton: *mut c_void) -> *mut ffi::svn_error_t {
    unsafe {
        if let Some(token) = GLOBAL_CANCEL_TOKEN.get() {
            let msg = std::ffi::CString::new(token.msg.as_str()).unwrap_or_default();
            return ffi::svn_error_create(token.code, std::ptr::null_mut(), msg.as_ptr());
        }

        let ctx = (baton as *mut ContextInner)
            .as_mut()
            .expect("Failed to cast baton to mutable reference");
        let v = match ctx.context_notifier.cancel() {
            Ok(v) => v,
            Err(e) => return e.native_error(),
        };
        if let Some(msg) = v {
            let msg = std::ffi::CString::new(msg.as_str()).unwrap_or_default();
            return ffi::svn_error_create(
                ffi::svn_errno_t_SVN_ERR_CANCELLED as _,
                std::ptr::null_mut(),
                msg.as_ptr(),
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

        let this = (baton as *mut ContextInner)
            .as_mut()
            .expect("Failed to cast baton to mutable reference");

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
        let this = (baton as *mut ContextInner)
            .as_mut()
            .expect("Failed to cast baton to mutable reference");

        let notify = WorkingCopyNotify::from(notify);

        // tracing::info!("Notify: {:#?}", notify);

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
        let ctx = (baton as *mut ContextInner)
            .as_mut()
            .expect("Failed to cast baton to mutable reference");
        let realm_string = realm_string.to_nullable_str().unwrap_or_default();
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

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SslServerCertInfo {
    hostname: Option<String>,
    fingerprint: String,
    valid_from: String,
    valid_until: String,
    issuer: String,
    ascii_cert: String,
}

// #[derive(Debug, Clone, Copy)]
// struct CredSslServerTrust {
//     accepted_failures: SslFailures,
//     may_save: bool,
// }

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
            let info = info.as_ref().expect("Failed to get reference to info");
            let hostname = info.hostname.to_nullable_string();
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

#[derive(Debug, Clone, Copy, new, Default, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TrustServer {
    #[ts(type = "number")]
    accept_failures: u32,
    save: bool,
}

#[derive(derive_more::Debug, Clone, new, Default, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Authentication {
    pub username: String,
    #[debug(skip)]
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
        let realm = realm.to_nullable_str().unwrap_or_default();
        let info = SslServerCertInfo::from_raw(info);
        tracing::trace!(
            "Server info: realm={} failures={} info={:?} may_save={}",
            realm,
            failures,
            info,
            may_save
        );

        let this = (baton as *mut ContextInner)
            .as_mut()
            .expect("Failed to cast baton to mutable reference");

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

        let context = (baton as *mut ContextInner)
            .as_mut()
            .expect("Failed to cast baton to mutable reference");
        let v = match context.context_notifier.authenticate(
            realm.to_nullable_str().unwrap_or_default().to_string(),
            username.to_nullable_str().unwrap_or_default().to_string(),
            may_save != 0,
            true,
        ) {
            Ok(v) => v,
            Err(e) => return e.native_error(),
        };
        tracing::info!("Authenticate result: {:?}", v);
        if let Some(result) = v {
            *cred = pool.malloc();

            let cred = (*cred)
                .as_mut()
                .expect("Failed to get mutable reference to credential");

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

            SubversionError::from_nullable_ptr(error).context(builder::Subversion)?;

            let pool = ffi::svn_pool_create_ex(std::ptr::null_mut(), std::ptr::null_mut());

            ffi::svn_utf_initialize2(0, pool);

            let error = ffi::svn_nls_init();

            SubversionError::from_nullable_ptr(error).context(builder::Subversion)?;

            Ok(ContextFactory {})
        })
    }
}

#[derive(Debug, new, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
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
                revsions.len().try_into().expect("Failed to convert size"),
                size_of::<usize>()
                    .try_into()
                    .expect("Failed to convert size"),
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

#[svnexus_macro::enum_converter(repr_type=ffi::svn_diff_file_ignore_space_t)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum DiffFileIgnoreSpace {
    None = ffi::svn_diff_file_ignore_space_t_svn_diff_file_ignore_space_none,
    Change = ffi::svn_diff_file_ignore_space_t_svn_diff_file_ignore_space_change,
    All = ffi::svn_diff_file_ignore_space_t_svn_diff_file_ignore_space_all,
}

#[derive(Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
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

#[derive(Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DifferenceOptions {
    pub original: Vec<u8>,
    pub modified: Vec<u8>,
    pub options: Option<DifferenceFileOptions>,
}

impl DifferenceOptions {
    pub fn exec(self) -> error::Result<DifferenceResult> {
        unsafe {
            let mut pool = apr::Pool::create();

            let original = ffi::svn_string_ncreate(
                self.original.as_ptr() as _,
                self.original
                    .len()
                    .try_into()
                    .expect("Failed to convert size"),
                pool.as_mut_ptr(),
            );

            let modified = ffi::svn_string_ncreate(
                self.modified.as_ptr() as _,
                self.modified
                    .len()
                    .try_into()
                    .expect("Failed to convert size"),
                pool.as_mut_ptr(),
            );

            let mut diff = std::ptr::null_mut::<ffi::svn_diff_t>();

            let file_options = if let Some(options) = self.options {
                let file_options = ffi::svn_diff_file_options_create(pool.as_mut_ptr());

                options.setup(
                    file_options
                        .as_mut()
                        .expect("Failed to get mutable reference to file"),
                );
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

            SubversionError::from_nullable_ptr(error).context(builder::Subversion)?;

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
                    let changes = changes
                        .as_mut()
                        .expect("Failed to get mutable reference to changes");

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

#[derive(Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TextPosition {
    #[ts(type = "number")]
    pub pos: u64,
    #[ts(type = "number")]
    pub len: u64,
}

#[derive(Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TextChange {
    pub original: TextPosition,
    pub modified: TextPosition,
}

#[derive(Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DifferenceResult {
    pub modified: Vec<TextChange>,
}

#[derive(Debug, new, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct GetRepositoryRootResult {
    pub root_url: String,
    pub uuid: String,
}

#[derive(Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
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

pub trait InitializeRepositoryNotifier: Send + Sync + 'static {
    fn on_checkout_directly(&self) -> error::Result<()>;
    fn on_import(&self) -> error::Result<()>;
    fn on_backup(&self) -> error::Result<()>;
    fn on_backup_finished(&self, path: String) -> error::Result<()>;
    fn on_checkout(&self) -> error::Result<()>;
    fn on_finished(&self) -> error::Result<()>;
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

    pub fn initialize_repository(
        &mut self,
        opts: InitializeRepositoryOptions,
        notifier: Arc<dyn InitializeRepositoryNotifier>,
    ) -> error::Result<()> {
        struct Filter {
            ignore: Gitignore,
            file: std::fs::File,
            folders: Vec<String>,
            relate_to: String,
        }

        impl Filter {
            fn create(
                relate_to: String,
                file: std::fs::File,
                filters: Vec<String>,
            ) -> error::Result<Filter> {
                let mut builder = GitignoreBuilder::new(&relate_to);

                for line in filters {
                    builder.add_line(None, &line).context(builder::Glob)?;
                }

                let ignore = builder.build().context(builder::Glob)?;

                Ok(Self {
                    ignore,
                    file,
                    folders: Vec::new(),
                    relate_to,
                })
            }

            fn create_filter(
                relate_to: String,
                file: std::fs::File,
                filters: Vec<String>,
            ) -> error::Result<
                Box<
                    dyn FnMut(
                        String,
                        NodeKind,
                        bool,
                        Option<u64>,
                        i64,
                    ) -> error::Result<bool, error::FrontendError>,
                >,
            > {
                let mut this = Self::create(relate_to, file, filters)?;

                Ok(Box::new(move |path, kind, _, _, _| {
                    let relative_path = path
                        .trim_start_matches(&this.relate_to)
                        .trim_start_matches("/");

                    let matcher = this
                        .ignore
                        .matched(relative_path, matches!(kind, NodeKind::Directory));

                    let ignore = matcher.is_ignore();

                    if ignore {
                        // if matches!(kind, NodeKind::Directory) {

                        // }
                        //
                        let new = this.folders.iter().all(|i| !path.starts_with(i));

                        if new {
                            if let Err(e) = this
                                .file
                                .write_all(format!("{}\n", relative_path).as_bytes())
                            {
                                tracing::warn!("Failed to write ignore list to file: {}", e);
                            }
                        } else if matches!(kind, NodeKind::Directory) {
                            this.folders.push(path);
                        }
                    }

                    Ok(ignore)
                }))
            }
        }

        let (exclude_file, exclude_file_path) = NamedTempFile::new()?
            .keep()
            .with_any_context(|e| format!("Failed to create exclude file: {}", e))?;

        let mut filter = opts
            .filters
            .clone()
            .map(|f| Filter::create_filter(opts.local.clone(), exclude_file, f))
            .transpose()?;

        let filter = filter.as_mut();

        let filter = filter.map(|v| v.as_mut());

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
            };
            self.import_filter(import_optios, filter)?;
            self.cancelled()?;
            if let Some(directory) = opts.backup_directory {
                notifier.on_backup()?;
                // let file = utils::backup(
                //     &opts.local,
                //     if directory.is_empty() {
                //         None
                //     } else {
                //         Some(directory)
                //     },
                // )?;

                let path = PathBuf::from(&opts.local);
                if !path.exists() {
                    std::fs::create_dir_all(&path)?;
                } else if !path.is_dir() {
                    return builder::General {
                        detail: format!("{} must not be file", path.display()),
                    }
                    .fail();
                }

                // snafu::ensure!(
                //     path.exists(),
                //     builder::General {
                //         detail: "Path does not exist"
                //     }
                // );
                let file_name = path
                    .canonicalize()?
                    .file_name()
                    .map(|v| v.to_os_string())
                    .unwrap_or(OsString::from("backup"));

                let output = if directory.is_empty() {
                    tempfile::tempdir()?.keep()
                } else {
                    PathBuf::from(directory)
                };

                let mut index = 0;

                let file = loop {
                    let name = file_name.clone().also_apply(|f| {
                        if index == 0 {
                            f.push(".tar.gz")
                        } else {
                            f.push(format!(".{}.tar.gz", index));
                        }
                    });

                    let file = output.join(name);

                    if !file.exists() {
                        break file;
                    }
                    index += 1;
                };

                let tar = platform::tar()?;

                tracing::info!("execute tar in {}", opts.local);
                let status = Command::new(tar)
                    .current_dir(&opts.local)
                    .arg(OsString::new().also_apply(|s| {
                        s.push("--exclude-from=");
                        s.push(exclude_file_path);
                    }))
                    .arg("-zcvf")
                    .arg(&file)
                    .arg(".")
                    // .arg(format!("--exclude-from={}", ""))
                    .status()?;

                snafu::ensure!(
                    status.success(),
                    builder::General {
                        detail: "Failed to backup"
                    }
                );

                notifier
                    .on_backup_finished(file.to_str().expect("Invalid UTF-8 string").to_string())?;
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

    pub fn url_from_path(&mut self, path: String) -> error::Result<String> {
        unsafe {
            let mut pool = apr::Pool::create();
            let mut url: *const std::ffi::c_char = std::ptr::null_mut();
            let path = pool.string(path)?;
            let error = ffi::svn_client_url_from_path2(
                url.pointer_mut(),
                path,
                self.ctx(),
                pool.as_mut_ptr(),
                pool.as_mut_ptr(),
            );
            SubversionError::from_nullable_ptr(error).context(builder::Subversion)?;

            Ok(url.to_str().to_string())
        }
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
            SubversionError::from_nullable_ptr(error).context(builder::Subversion)?;

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
            SubversionError::from_nullable_ptr(error).context(builder::Subversion)?;

            let mut root: *const c_char = std::ptr::null_mut();
            let error = ffi::svn_client_get_wc_root(
                root.pointer_mut(),
                absolute_path,
                self.ctx(),
                pool.as_mut_ptr(),
                pool.as_mut_ptr(),
            );
            SubversionError::from_nullable_ptr(error).context(builder::Subversion)?;

            let root = root.to_str().to_string();
            Ok(root)
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
            SubversionError::from_nullable_ptr(error).context(builder::Subversion)?;

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

            SubversionError::from_nullable_ptr(error).context(builder::Subversion)?;

            let root_url = root_url.to_str().to_string();

            let uuid = uuid.to_str().to_string();

            Ok(GetRepositoryRootResult { root_url, uuid })
        }
    }

    pub fn open_repository_access_session(
        &mut self,
        url: String,
        path: Option<String>,
    ) -> error::Result<(*mut ffi::apr_pool_t, *mut ffi::svn_ra_session_t)> {
        unsafe {
            let mut pool = apr::Pool::create();
            let url = pool.canonicalize_uri(&url)?;
            let path = path
                .map(|v| pool.string(v))
                .transpose()?
                .unwrap_or_default();
            let mut session: *mut ffi::svn_ra_session_t = std::ptr::null_mut();
            let session_pool = apr::create();

            let guard = DropGuard::new(session_pool, |p| {
                ffi::apr_pool_destroy(p);
            });

            let error = ffi::svn_client_open_ra_session2(
                session.pointer_mut(),
                url,
                path,
                self.ctx(),
                session_pool,
                pool.as_mut_ptr(),
            );
            SubversionError::from_nullable_ptr(error).context(builder::Subversion)?;

            Ok((DropGuard::dismiss(guard), session))

            // let session = AutoPool {
            //     pool: session_pool,
            //     value: session,
            // };

            // let key = self
            //     .ra_sessions
            //     .keys()
            //     .map(|e| *e)
            //     .max()
            //     .unwrap_or_default()
            //     + 1;

            // self.ra_sessions.insert(key, session);

            // Ok(key)
        }
    }

    fn create(opts: CreateContextOptions) -> error::Result<Self> {
        unsafe extern "C" fn conflict(
            result: *mut *mut ffi::svn_wc_conflict_result_t,
            description: *const ffi::svn_wc_conflict_description2_t,
            baton: *mut c_void,
            result_pool: *mut ffi::apr_pool_t,
            _scratch_pool: *mut ffi::apr_pool_t,
        ) -> *mut ffi::svn_error_t {
            unsafe {
                let this = (baton as *mut ContextInner)
                    .as_mut()
                    .expect("Failed to cast baton to mutable reference");

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
                    user_result.choice.into(),
                    merged_file,
                    result_pool,
                );

                let result = (*result).as_mut().expect("Failed to get mutable reference");

                result.choice = user_result.choice.into();
                result.merged_value = user_result
                    .merged_value
                    .map(|e| {
                        ffi::svn_string_ncreate(
                            e.as_ptr() as _,
                            e.len().try_into().expect("Failed to convert size"),
                            result_pool,
                        )
                    })
                    .unwrap_or_default();

                result.save_merged = user_result.save_merged.into();
            }

            svn_no_error()
        }

        unsafe {
            let pool = apr::create();

            let guard = DropGuard::new(pool, |p| {
                ffi::apr_pool_destroy(p);
            });

            let mut context: *mut ffi::svn_client_ctx_t = std::ptr::null_mut();

            let mut store_password: ffi::svn_boolean_t =
                ffi::patches_svn_config_default_option_store_passwords();

            let mut store_auth_creds = ffi::patches_svn_config_default_option_store_auth_creds();

            let mut ssl_client_cert_file_prompt: ffi::svn_boolean_t = 0;

            let mut inner = Box::new(ContextInner::new(opts.context_notifier.clone()));

            let hash = read_subversion_config(pool, None)?;

            // opts.config.apply(hash, &mut pool)?;

            let config = apr::ffi::apr_hash_get(
                hash,
                ffi::SVN_CONFIG_CATEGORY_CONFIG.as_ptr() as *const i8 as *const c_void,
                ffi::APR_HASH_KEY_STRING
                    .try_into()
                    .expect("Failed to convert value"),
            ) as *mut ffi::svn_config_t;

            let error = ffi::svn_config_get_bool(
                config,
                store_password.pointer_mut(),
                ffi::SVN_CONFIG_SECTION_AUTH.as_ptr() as _,
                ffi::SVN_CONFIG_OPTION_STORE_PASSWORDS.as_ptr() as _,
                ffi::patches_svn_config_default_option_store_passwords(),
            );

            SubversionError::from_nullable_ptr(error).context(builder::Subversion)?;

            let error = ffi::svn_config_get_bool(
                config,
                store_auth_creds.pointer_mut(),
                ffi::SVN_CONFIG_SECTION_AUTH.as_ptr() as _,
                ffi::SVN_CONFIG_OPTION_STORE_AUTH_CREDS.as_ptr() as _,
                ffi::patches_svn_config_default_option_store_auth_creds(),
            );
            SubversionError::from_nullable_ptr(error).context(builder::Subversion)?;

            let error = ffi::svn_config_get_bool(
                config,
                ssl_client_cert_file_prompt.pointer_mut(),
                ffi::SVN_CONFIG_SECTION_AUTH.as_ptr() as _,
                ffi::SVN_CONFIG_OPTION_SSL_CLIENT_CERT_FILE_PROMPT.as_ptr() as _,
                0,
            );
            SubversionError::from_nullable_ptr(error).context(builder::Subversion)?;

            inner.tunnel.insert(
                "ssh".to_string(),
                tunnel::ssh_tunnel(opts.context_notifier.clone()),
            );

            let error = ffi::svn_client_create_context2(&mut context as *mut _, hash, pool);
            SubversionError::from_nullable_ptr(error).context(builder::Subversion)?;

            assert!(!context.is_null());

            let mut array: *mut ffi::apr_array_header_t = std::ptr::null_mut();

            let mut provider = std::ptr::null_mut();

            let error = ffi::svn_auth_get_platform_specific_client_providers(
                array.pointer_mut(),
                config,
                pool,
            );

            SubversionError::from_nullable_ptr(error).context(builder::Subversion)?;

            ffi::svn_auth_get_simple_provider2(
                &mut provider as *mut _,
                Some(may_save_password_as_plain_text),
                inner.inner_void_pointer_mut(),
                pool,
            );
            let ptr = ffi::apr_array_push(array) as *mut *mut ffi::svn_auth_provider_object_t;
            *ptr = provider;

            ffi::svn_auth_get_username_provider(&mut provider as *mut _, pool);
            let ptr = ffi::apr_array_push(array) as *mut *mut ffi::svn_auth_provider_object_t;
            *ptr = provider;

            // ssl
            ffi::svn_auth_get_ssl_server_trust_file_provider(&mut provider as *mut _, pool);
            let ptr = ffi::apr_array_push(array) as *mut *mut ffi::svn_auth_provider_object_t;
            *ptr = provider;

            ffi::svn_auth_get_ssl_client_cert_file_provider(&mut provider as *mut _, pool);
            let ptr = ffi::apr_array_push(array) as *mut *mut ffi::svn_auth_provider_object_t;
            *ptr = provider;

            ffi::svn_auth_get_ssl_client_cert_pw_file_provider2(
                &mut provider as *mut _,
                Some(may_save_password_as_plain_text),
                inner.inner_void_pointer_mut(),
                pool,
            );
            let ptr = ffi::apr_array_push(array) as *mut *mut ffi::svn_auth_provider_object_t;
            *ptr = provider;

            ffi::svn_auth_get_simple_prompt_provider(
                &mut provider as *mut _,
                Some(on_authenticate),
                inner.inner_void_pointer_mut(),
                1000000,
                pool,
            );
            let ptr = ffi::apr_array_push(array) as *mut *mut ffi::svn_auth_provider_object_t;
            *ptr = provider;

            unsafe extern "C" fn username_provider(
                cred: *mut *mut ffi::svn_auth_cred_username_t,
                baton: *mut c_void,
                realm: *const c_char,
                may_save: ffi::svn_boolean_t,
                pool: *mut ffi::apr_pool_t,
            ) -> *mut ffi::svn_error_t {
                unsafe {
                    let cancel = on_cancel(baton);
                    if !cancel.is_null() {
                        return cancel;
                    }

                    let context = (baton as *mut ContextInner)
                        .as_mut()
                        .expect("Failed to cast baton to mutable reference");
                    let v = match context.context_notifier.authenticate(
                        realm.to_nullable_str().unwrap_or_default().to_string(),
                        Default::default(),
                        may_save != 0,
                        false,
                    ) {
                        Ok(v) => v,
                        Err(e) => return e.native_error(),
                    };
                    tracing::info!("Authenticate result: {:?}", v);
                    if let Some(result) = v {
                        *cred = pool.malloc();

                        let cred = (*cred)
                            .as_mut()
                            .expect("Failed to get mutable reference to credential");

                        cred.may_save = result.save.into();
                        cred.username = pool.string(result.username).expect("Invalid username");
                    }
                    svn_no_error()
                }
            }

            ffi::svn_auth_get_username_prompt_provider(
                provider.pointer_mut(),
                Some(username_provider),
                inner.inner_void_pointer_mut(),
                200,
                pool,
            );

            ffi::svn_auth_get_ssl_server_trust_prompt_provider(
                &mut provider as *mut _,
                Some(ssl_server_trust_prompt),
                inner.inner_void_pointer_mut(),
                pool,
            );
            let ptr = ffi::apr_array_push(array) as *mut *mut ffi::svn_auth_provider_object_t;
            *ptr = provider;

            unsafe extern "C" fn ssl_client_certificate_provider(
                cred: *mut *mut ffi::svn_auth_cred_ssl_client_cert_t,
                baton: *mut c_void,
                realm: *const c_char,
                may_save: ffi::svn_boolean_t,
                pool: *mut ffi::apr_pool_t,
            ) -> *mut ffi::svn_error_t {
                unsafe {
                    let context = (baton as *mut ContextInner)
                        .as_ref()
                        .expect("Failed to cast baton to reference");

                    let realm = realm.to_nullable_str().unwrap_or_default().to_string();
                    let may_save = may_save != 0;

                    let result = context
                        .context_notifier
                        .ssl_client_certificate(realm, may_save);

                    match result {
                        Ok(v) => {
                            if let Some(v) = v {
                                *cred = pool.malloc();

                                let result = pool.string(v.file.as_str()).map_err(|_| {
                                    error::UnexpectedSnafu {
                                        detail: "Invalid file path",
                                    }
                                    .build()
                                    .native_error()
                                });

                                let file = match result {
                                    Ok(file) => file,
                                    Err(e) => return e,
                                };

                                (*cred)
                                    .as_mut()
                                    .expect("Failed to get mutable reference to credential")
                                    .also_build(|e| {
                                        e.cert_file = file;

                                        e.may_save = v.save.into();

                                        e
                                    });
                            }
                        }
                        Err(e) => return e.native_error(),
                    }
                }

                svn_no_error()
            }

            if ssl_client_cert_file_prompt != 0 {
                ffi::svn_auth_get_ssl_client_cert_prompt_provider(
                    provider.pointer_mut(),
                    Some(ssl_client_certificate_provider),
                    inner.inner_void_pointer_mut(),
                    2,
                    pool,
                );
            }

            let ctx = context
                .as_mut()
                .expect("Failed to get mutable reference to context");

            ffi::svn_auth_open(ctx.auth_baton.pointer_mut(), array, pool);

            if let Some(username) = opts.default_username {
                ffi::svn_auth_set_parameter(
                    ctx.auth_baton,
                    ffi::SVN_AUTH_PARAM_DEFAULT_USERNAME.as_ptr() as _,
                    pool.string(username)? as *const c_void,
                );
            }
            if let Some(password) = opts.default_password {
                ffi::svn_auth_set_parameter(
                    ctx.auth_baton,
                    ffi::SVN_AUTH_PARAM_DEFAULT_PASSWORD.as_ptr() as _,
                    pool.string(password)? as *const c_void,
                );
            }
            if store_password == 0 {
                ffi::svn_auth_set_parameter(
                    ctx.auth_baton,
                    ffi::SVN_AUTH_PARAM_DONT_STORE_PASSWORDS.as_ptr() as _,
                    b"\0".as_ptr() as _,
                );
            }
            if store_auth_creds == 0 {
                ffi::svn_auth_set_parameter(
                    ctx.auth_baton,
                    ffi::SVN_AUTH_PARAM_NO_AUTH_CACHE.as_ptr() as _,
                    b"\0".as_ptr() as _,
                );
            }

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
            ctx.check_tunnel_func = Some(tunnel::check_tunnel);
            ctx.open_tunnel_func = Some(tunnel::open_tunnel);
            ctx.tunnel_baton = inner.inner_void_pointer_mut();

            if let Some(ref name) = opts.name {
                ctx.client_name = pool.string(name)?;
            }
            Ok(Self {
                ptr: context,
                pool: DropGuard::dismiss(guard),
                inner,
                // ra_sessions: Default::default(),
            })
        }
    }
}

pub struct DropGuard<T, F>
where
    F: FnOnce(T),
{
    inner: ManuallyDrop<T>,
    f: ManuallyDrop<F>,
}

impl<T, F> DropGuard<T, F>
where
    F: FnOnce(T),
{
    #[must_use]
    pub const fn new(inner: T, f: F) -> Self {
        Self {
            inner: ManuallyDrop::new(inner),
            f: ManuallyDrop::new(f),
        }
    }

    #[inline]
    pub fn dismiss(guard: Self) -> T {
        // First we ensure that dropping the guard will not trigger
        // its destructor
        let mut guard = ManuallyDrop::new(guard);

        // Next we manually read the stored value from the guard.
        //
        // SAFETY: this is safe because we've taken ownership of the guard.
        let value = unsafe { ManuallyDrop::take(&mut guard.inner) };

        // Finally we drop the stored closure. We do this *after* having read
        // the value, so that even if the closure's `drop` function panics,
        // unwinding still tries to drop the value.
        //
        // SAFETY: this is safe because we've taken ownership of the guard.
        unsafe { ManuallyDrop::drop(&mut guard.f) };
        value
    }
}

impl<T, F> Drop for DropGuard<T, F>
where
    F: FnOnce(T),
{
    fn drop(&mut self) {
        // SAFETY: `DropGuard` is in the process of being dropped.
        let inner = unsafe { ManuallyDrop::take(&mut self.inner) };

        // SAFETY: `DropGuard` is in the process of being dropped.
        let f = unsafe { ManuallyDrop::take(&mut self.f) };

        f(inner);
    }
}

pub unsafe fn read_subversion_config(
    pool: *mut ffi::apr_pool_t,
    path: Option<&str>,
) -> error::Result<*mut ffi::apr_hash_t> {
    let mut hash: *mut ffi::apr_hash_t = std::ptr::null_mut();

    let p = path
        .map(|p| unsafe { pool.string(p) })
        .transpose()?
        .unwrap_or_default();

    unsafe {
        let mut error = ffi::svn_config_get_config(hash.pointer_mut(), p, pool);
        // 逻辑根据svn的源码
        if let Some(e) = error.as_ref() {
            if ffi::patches_apr_status_is_eacces(e.apr_err) != 0
                || ffi::patches_svn_apr_status_is_enotdir(e.apr_err) != 0
            {
                tracing::warn!(
                    "Failed to read config in {:?}: {}, fallback to default config",
                    path,
                    e.message.to_nullable_str().unwrap_or_default()
                );
                ffi::svn_error_clear(error);
                error = ffi::svn_config__get_default_config(hash.pointer_mut(), pool);
            }
        }
        super::SubversionError::from_nullable_ptr(error).context(builder::Subversion)?;
        assert!(!hash.is_null(), "Failed to read Subversion configuration");
    }

    Ok(hash)
}
