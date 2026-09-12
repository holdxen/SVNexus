use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::*;

#[derive(Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MkdirOptions {
    paths: Vec<String>,
    make_parents: bool,
    revision_property_table: Option<HashMap<String, String>>,
    commit_message: String,
}

#[derive(Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MkdirResult {
    commit_info: Option<CommitInfo>,
}

impl Context {
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

            SubversionError::from_nullable_ptr(error).context(builder::Subversion)?;

            Ok(MkdirResult {
                commit_info: self.inner.commit_info.take(),
            })
        }
    }
}
