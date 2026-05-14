use std::ffi::{c_char, c_void};

use derive_new::new;
use snafu::ResultExt;

use crate::{
    apr,
    error::{self, builder},
    extensions::Canonicalization,
    subversion::{
        SVNError,
        context::{ContextInner, Depth, Lock, NodeKind, Revision, RevisionNumber, on_cancel},
        ffi, svn_no_error,
    },
    utils::{Boxed, CStringer, Pointer, PointerMapper},
};

use super::Context;

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

impl Context {
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

            let path = pool.canonicalize_target(&opts.path)?;

            let path = ffi::svn_uri_canonicalize(path, pool.as_mut_ptr());

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
            SVNError::from_nullable_ptr(error).context(builder::Svn)?;
        }
        Ok(ListResult {
            entries: std::mem::take(&mut self.inner.list_entries),
        })
    }
}
