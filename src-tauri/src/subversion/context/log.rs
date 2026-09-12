use super::{
    ffi, svn_no_error, Context, NodeKind, Revision, RevisionNumber, RevisionRange, SubversionError,
};
use crate::apr;
use crate::error::{self, builder, FrontendErrorExtension};
use crate::extensions::Canonicalization;
use crate::utils::PointerMapper;
use crate::utils::{CStringer, Pointer};
use derive_new::new;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;
use std::collections::HashMap;
use std::ffi::{c_char, c_void};
use strum::EnumString;
use surrealdb::types::SurrealValue;

fn from_svn_stristate(value: ffi::svn_tristate_t) -> Option<bool> {
    let true_value = ffi::svn_tristate_t::try_from(ffi::svn_tristate_t_svn_tristate_true)
        .expect("value must be valid");
    let false_value = ffi::svn_tristate_t::try_from(ffi::svn_tristate_t_svn_tristate_false)
        .expect("Value must be valid");
    let unknown_value = ffi::svn_tristate_t::try_from(ffi::svn_tristate_t_svn_tristate_unknown)
        .expect("Value must be valid");

    if value == true_value {
        Some(true)
    } else if value == false_value {
        Some(false)
    } else if value == unknown_value {
        None
    } else {
        panic!("Invalid svn tristate value: {}", value)
    }

    // match value {
    //     true_value => Some(true),
    //     ffi::svn_tristate_t_svn_tristate_false => Some(false),
    //     ffi::svn_tristate_t_svn_tristate_unknown => None,
    //     _ => panic!("Invalid svn tristate value: {}", value),
    // }
}
#[svnexus_macro::enum_converter(repr_type=u8)]
#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, EnumString, strum::Display, ts_rs::TS, SurrealValue,
)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum LogChangedPathAction {
    Add = b'A',
    Delete = b'D',
    Replace = b'R',
    Modify = b'M',
}
impl LogChangedPathAction {
    fn from_char(value: std::ffi::c_char) -> Self {
        Self::try_from(u8::try_from(value).expect("Unexpected failure"))
            .expect("Unexpected failure")
    }
}

#[derive(new, Debug, Clone, Serialize, Deserialize, ts_rs::TS, SurrealValue)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LogChangedPathEntry {
    pub action: LogChangedPathAction,
    pub copy_from_path: Option<String>,
    pub copy_from_revision: Option<RevisionNumber>,
    pub node_kind: NodeKind,
    pub text_modified: Option<bool>,
    pub props_modified: Option<bool>,
}

#[derive(Default, new, Debug, Clone, Serialize, Deserialize, ts_rs::TS, SurrealValue)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LogEntry {
    pub revision: Option<RevisionNumber>,
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

            let action = LogChangedPathAction::from_char(ptr.action);
            let copy_from_path = ptr.copyfrom_path.to_nullable_string();
            let copy_from_revision = ptr.copyfrom_rev.try_into().ok();

            let node_kind = NodeKind::try_from(ptr.node_kind).expect("Unexpected failure");

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
            let log_entry = ptr.as_ref().expect("Failed to get reference to entry");

            let revision = log_entry.revision.try_into().ok();

            let mut pool = apr::Pool::create();

            let revision_properties = log_entry.revprops.map(|v| pool.convert_to_hash_map(v as _));

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

                    let path = (key as *const c_char).to_str().to_string();

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
                revision_properties,
                changed_path_entries,
                has_children,
                non_inheritable,
                subtractive_merge,
            }
        }
    }
}

#[derive(new, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LogResult {
    pub entries: Vec<LogEntry>,
}

#[derive(Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
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

// pub trait LogReceiver: Send + Sync + 'static {
//     fn on_log_entry(&self, log_entry: LogEntry) -> Result<(), error::FrontendError>;
// }

impl Context {
    pub fn log_next<F: (Fn(LogEntry) -> Result<(), error::FrontendError>) + 'static>(
        &mut self,
        opts: LogOptions,
        mut receiver: F,
    ) -> error::Result<()> {
        unsafe extern "C" fn log_receiver<
            F: (Fn(LogEntry) -> Result<(), error::FrontendError>) + 'static,
        >(
            baton: *mut c_void,
            log_entry: *mut ffi::svn_log_entry_t,
            _pool: *mut ffi::apr_pool_t,
        ) -> *mut ffi::svn_error_t {
            unsafe {
                let receiver = (baton as *mut F)
                    .as_ref()
                    .expect("Failed to cast baton to reference");
                let log_entry = LogEntry::from_raw(log_entry);
                let result = (receiver)(log_entry);
                result.native_error()

                // let this = (baton as *mut ContextInner).as_mut().unwrap();
                // let log_entry = LogEntry::from_raw(log_entry);
                // let result = this.log_receiver.as_ref().unwrap()(log_entry);
                // result.native_error()
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

            // self.inner.log_receiver = Some(Box::new(receiver));

            let error = ffi::svn_client_log5(
                targets,
                &peg_revision as *const _,
                pool.revision_range(&opts.revisions),
                opts.limit.try_into().unwrap_or(std::ffi::c_int::MAX),
                opts.discover_changed_paths.into(),
                opts.strict_node_history.into(),
                opts.include_merged_revisions.into(),
                revisions_properties,
                Some(log_receiver::<F>),
                receiver.pointer_mut() as _,
                self.ctx(),
                pool.as_mut_ptr(),
            );

            // self.inner.log_receiver = None;

            SubversionError::from_nullable_ptr(error).context(builder::Subversion)?;
        }
        Ok(())
    }

    pub fn log(&mut self, opts: LogOptions) -> error::Result<LogResult> {
        let mut entries: Vec<LogEntry> = Vec::with_capacity(128);

        unsafe extern "C" fn log_receiver(
            baton: *mut c_void,
            log_entry: *mut ffi::svn_log_entry_t,
            _pool: *mut ffi::apr_pool_t,
        ) -> *mut ffi::svn_error_t {
            unsafe {
                let entries = (baton as *mut Vec<LogEntry>)
                    .as_mut()
                    .expect("Failed to cast baton to mutable reference");
                entries.push(LogEntry::from_raw(log_entry));
            }
            svn_no_error()
        }

        unsafe {
            let mut pool = apr::Pool::create();
            let targets =
                pool.canonicalize_target_array(opts.targets.len(), opts.targets.iter())?;

            let peg_revision = opts.peg_revision.to_opt_revision();

            // self.inner.log_entries.clear();

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
                Some(log_receiver),
                entries.pointer_mut() as _,
                self.ctx(),
                pool.as_mut_ptr(),
            );

            SubversionError::from_nullable_ptr(error).context(builder::Subversion)?;
        }
        // tracing::info!("got entry size: {}", self.inner.log_entries.len());
        // let r = LogResult::new(std::mem::take(&mut self.inner.log_entries));
        // tracing::info!("result: {}", r.entries.len());
        Ok(LogResult { entries })
    }
}
