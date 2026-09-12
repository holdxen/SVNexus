use std::collections::HashMap;

use derive_new::new;
use serde::{Deserialize, Serialize};

use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CopySourceItem {
    path: String,
    peg_revision: Revision,
    revision: Revision,
}

#[derive(Debug, Clone, new, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CopyOptions {
    sources: Vec<CopySourceItem>,
    destination: String,
    copy_as_child: bool,
    make_parents: bool,
    ignore_externals: bool,
    metadata_only: bool,
    pin_externals: bool,
    revision_property_table: Option<HashMap<String, String>>,
    commit_message: String,
}

#[derive(Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CopyResult {
    commit_info: Option<CommitInfo>,
}

impl Context {
    pub fn copy(&mut self, opts: CopyOptions) -> error::Result<CopyResult> {
        unsafe {
            let mut pool = apr::Pool::create();

            let sources = ffi::apr_array_make(
                pool.as_mut_ptr(),
                opts.sources.len().try_into().expect("Failed to convert size"),
                size_of::<usize>().try_into().expect("Failed to convert size"),
            );

            for source in &opts.sources {
                let copy_source: *mut ffi::svn_client_copy_source_t = pool.malloc();
                let copy_source = &mut *copy_source;

                let path = pool.canonicalize_target(&source.path)?;
                copy_source.path = path;

                let revision: *mut ffi::svn_opt_revision_t = pool.malloc();
                *revision = source.revision.to_opt_revision();
                copy_source.revision = revision;

                let peg_revision: *mut ffi::svn_opt_revision_t = pool.malloc();
                *peg_revision = source.peg_revision.to_opt_revision();
                copy_source.peg_revision = peg_revision;

                let ptr = ffi::apr_array_push(sources) as *mut *mut ffi::svn_client_copy_source_t;
                *ptr = copy_source;
            }

            let destination = pool.canonicalize_target(&opts.destination)?;

            let revprop_table = opts
                .revision_property_table
                .as_ref()
                .map(|e| {
                    pool.string_hash_map(
                        e.iter(),
                        |p: &mut apr::Pool, k: &str| p.string(k).map(|o| o as _),
                        |p: &mut apr::Pool, v: &str| p.svn_string(v).map(|o| o as _),
                    )
                })
                .transpose()?
                .unwrap_or_default();

            self.inner.commit_message = opts.commit_message;

            let error = ffi::svn_client_copy7(
                sources,
                destination,
                opts.copy_as_child.into(),
                opts.make_parents.into(),
                opts.ignore_externals.into(),
                opts.metadata_only.into(),
                opts.pin_externals.into(),
                std::ptr::null(),
                revprop_table,
                Some(commit_callback),
                self.inner.inner_void_pointer_mut(),
                self.ctx(),
                pool.as_mut_ptr(),
            );

            SubversionError::from_nullable_ptr(error).context(builder::Subversion)?;

            Ok(CopyResult {
                commit_info: self.inner.commit_info.take(),
            })
        }
    }
}
