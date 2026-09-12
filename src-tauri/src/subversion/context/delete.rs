use std::collections::HashMap;

use derive_new::new;
use serde::{Deserialize, Serialize};

use super::*;

#[derive(new, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DeleteResult {
    info: Option<CommitInfo>,
}

#[derive(Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DeleteOptions {
    path: Vec<String>,
    force: bool,
    keep_local: bool,
    revision_property_table: Option<HashMap<String, String>>,
    commit_message: String,
}

impl Context {
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

            SubversionError::from_nullable_ptr(error).context(builder::Subversion)?;
        }

        Ok(DeleteResult::new(std::mem::take(
            &mut self.inner.commit_info,
        )))
    }
}
