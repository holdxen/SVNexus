use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::*;

#[derive(Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MoveOptions {
    src_paths: Vec<String>,
    destination: String,
    move_as_child: bool,
    make_parents: bool,
    allow_mixed_revisions: bool,
    metadata_only: bool,
    revision_property_table: Option<HashMap<String, String>>,
    commit_message: String,
}

#[derive(Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MoveResult {
    commit_info: Option<CommitInfo>,
}

impl Context {
    pub fn move_(&mut self, opts: MoveOptions) -> error::Result<MoveResult> {
        unsafe {
            let mut pool = apr::Pool::create();

            let src_paths =
                pool.canonicalize_target_array(opts.src_paths.len(), opts.src_paths.iter())?;

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

            let error = ffi::svn_client_move7(
                src_paths,
                destination,
                opts.move_as_child.into(),
                opts.make_parents.into(),
                opts.allow_mixed_revisions.into(),
                opts.metadata_only.into(),
                revprop_table,
                Some(commit_callback),
                self.inner.inner_void_pointer_mut(),
                self.ctx(),
                pool.as_mut_ptr(),
            );

            SubversionError::from_nullable_ptr(error).context(builder::Subversion)?;

            Ok(MoveResult {
                commit_info: self.inner.commit_info.take(),
            })
        }
    }
}
