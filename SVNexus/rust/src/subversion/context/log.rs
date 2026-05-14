use super::{
    Context, ContextInner, NodeKind, Revision, RevisionNumber, RevisionRange, SVNError, ffi,
    on_cancel, svn_no_error,
};
use crate::apr;
use crate::error::{self, CSharpErrorExtension, builder};
use crate::extensions::Canonicalization;
use crate::utils::CStringer;
use crate::utils::{Boxed, PointerMapper};
use derive_new::new;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;
use std::collections::HashMap;
use std::ffi::{c_char, c_void};
use std::sync::Arc;
use strum::EnumString;

fn from_svn_stristate(value: u32) -> Option<bool> {
    match value {
        ffi::svn_tristate_t_svn_tristate_true => Some(true),
        ffi::svn_tristate_t_svn_tristate_false => Some(false),
        ffi::svn_tristate_t_svn_tristate_unknown => None,
        _ => panic!("Invalid svn tristate value: {}", value),
    }
}
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

#[derive(
    Default, new, Debug, uniffi::Record, Clone, Serialize, Deserialize, svnexus_macro::ToDebugString,
)]
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

#[derive(new, Debug, uniffi::Record, svnexus_macro::ToDebugString)]
pub struct LogResult {
    pub log_entries: Vec<LogEntry>,
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

impl Context {
    pub fn log_next(
        &mut self,
        opts: LogOptions,
        receiver: Arc<dyn LogReceiver>,
    ) -> error::Result<()> {
        unsafe extern "C" fn log_receiver(
            baton: *mut c_void,
            log_entry: *mut ffi::svn_log_entry_t,
            _pool: *mut ffi::apr_pool_t,
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
            let targets =
                pool.canonicalize_target_array(opts.targets.len(), opts.targets.iter())?;

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
            _pool: *mut ffi::apr_pool_t,
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
            let targets =
                pool.canonicalize_target_array(opts.targets.len(), opts.targets.iter())?;

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
}
