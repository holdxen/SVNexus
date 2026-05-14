use std::collections::HashMap;

use crate::extensions::Canonicalization;
use crate::utils::{Boxed, CStringer, Pointer};

use super::{
    Context, ContextInner, Depth, Lock, NodeKind, Revision, RevisionNumber, SVNError,
    WorkingCopyInfo, ffi, on_cancel, svn_no_error,
};
use crate::apr;
use crate::error;
use crate::error::builder;
use crate::utils::PointerMapper;
use derive_new::new;
use snafu::ResultExt;
use std::ffi::{c_char, c_void};

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

#[derive(uniffi::Record, Debug, new)]
pub struct InfoEntry {
    url: String,
    revision: Option<RevisionNumber>,
    repository_root_url: String,
    repository_uuid: String,
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
            let ptr = self.as_ref().unwrap();

            let url = ptr.URL.to_str().to_string();

            let revision = ptr.rev.try_into().ok();

            let repository_root_url = ptr.repos_root_URL.to_str().to_string();

            let repository_uuid = ptr.repos_UUID.to_str().to_string();

            let kind = ptr.kind.try_into().unwrap();

            let size = if ptr.size < 0 {
                None
            } else {
                Some(ptr.size.try_into().unwrap())
            };

            let last_changed_revision = ptr.last_changed_rev.try_into().ok();

            let last_changed_date = ptr.last_changed_date.try_into().unwrap();

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
                let context = (baton as *mut ContextInner).as_mut().unwrap();
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

            SVNError::from_nullable_ptr(error).context(builder::Svn)?;
        }

        let entries = std::mem::take(&mut self.inner.info_entries);

        Ok(InfoResult { entries })
    }
}
