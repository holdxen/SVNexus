use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::extensions::Canonicalization;
use crate::utils::{Boxed, CStringer, Pointer};

use super::{
    ffi, on_cancel, svn_no_error, Context, ContextInner, Depth, Lock, NodeKind, Revision,
    RevisionNumber, SubversionError, WorkingCopyInfo,
};
use crate::apr;
use crate::error;
use crate::error::builder;
use crate::utils::PointerMapper;
use derive_new::new;
use snafu::ResultExt;
use std::ffi::{c_char, c_void};

#[derive(Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct InfoOptions {
    path: String,
    peg_revision: Revision,
    revision: Revision,
    depth: Depth,
    fetch_excluded: bool,
    fetch_actual_only: bool,
    include_externals: bool,
    changelists: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct InfoResult {
    pub entries: HashMap<String, InfoEntry>,
}

#[derive(Debug, new, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct InfoEntry {
    url: Option<String>,
    revision: Option<RevisionNumber>,
    repository_root_url: Option<String>,
    repository_uuid: Option<String>,
    kind: NodeKind,
    size: Option<u64>,
    last_changed_revision: Option<RevisionNumber>,
    last_changed_date: i64,
    last_changed_author: Option<String>,
    lock: Option<Lock>,
    working_copy_info: Option<WorkingCopyInfo>,
}

#[easy_ext::ext]
impl *const ffi::svn_client_info2_t {
    fn to_info_entry(self) -> InfoEntry {
        unsafe {
            let ptr = self.as_ref().expect("Failed to get reference from pointer");

            let url = ptr.URL.to_nullable_string();

            let revision = ptr.rev.try_into().ok();

            let repository_root_url = ptr.repos_root_URL.to_nullable_string();

            let repository_uuid = ptr.repos_UUID.to_nullable_string();

            let kind = ptr.kind.try_into().expect("Failed to convert node kind");

            let size = if ptr.size < 0 {
                None
            } else {
                Some(ptr.size.try_into().expect("Failed to convert size"))
            };

            let last_changed_revision = ptr.last_changed_rev.try_into().ok();

            let last_changed_date = ptr
                .last_changed_date
                .try_into()
                .expect("Failed to convert time");

            let last_changed_author = ptr.last_changed_author.to_nullable_string();

            let lock = if ptr.lock.is_null() {
                None
            } else {
                Some(Lock::from(ptr.lock))
            };

            let working_copy_info = ptr.wc_info.map(|v| v.into());

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

impl Context {
    pub fn info(&mut self, opts: InfoOptions) -> error::Result<InfoResult> {
        unsafe extern "C" fn info_receiver(
            baton: *mut c_void,
            path: *const c_char,
            info: *const ffi::svn_client_info2_t,
            _scratch_pool: *mut apr::ffi::apr_pool_t,
        ) -> *mut ffi::svn_error_t {
            unsafe {
                let error = on_cancel(baton);
                if !error.is_null() {
                    return error;
                }
                let context = (baton as *mut ContextInner)
                    .as_mut()
                    .expect("Failed to cast baton to mutable reference");
                let path = path.to_str().to_string();
                let info = info.to_info_entry();
                context.info_entries.insert(path, info);
            }

            svn_no_error()
        }
        unsafe {
            let mut pool = apr::Pool::create();

            let path = pool.canonicalize_target(&opts.path)?;

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

            SubversionError::from_nullable_ptr(error).context(builder::Subversion)?;
        }

        let entries = std::mem::take(&mut self.inner.info_entries);

        Ok(InfoResult { entries })
    }
}
