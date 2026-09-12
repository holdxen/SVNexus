use serde::{Deserialize, Serialize};
use snafu::ResultExt;

use crate::{
    apr,
    error::{self, builder},
    extensions::Canonicalization,
};

use super::{ffi, Context, Depth, SubversionError};

#[derive(Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RevertOptions {
    paths: Vec<String>,
    depth: Depth,
    changelists: Option<Vec<String>>,
    clear_changelists: bool,
    metadata_only: bool,
    added_keep_local: bool,
}

impl Context {
    pub fn revert(&mut self, opts: RevertOptions) -> error::Result<()> {
        unsafe {
            let mut pool = apr::Pool::create();
            let error = ffi::svn_client_revert4(
                pool.canonicalize_dirent_array(opts.paths.len(), opts.paths.iter())?,
                opts.depth.into(),
                opts.changelists
                    .map(|c| pool.string_array(c.len(), c.iter()))
                    .transpose()?
                    .unwrap_or_default(),
                opts.clear_changelists.into(),
                opts.metadata_only.into(),
                opts.added_keep_local.into(),
                self.ctx(),
                pool.as_mut_ptr(),
            );
            SubversionError::from_nullable_ptr(error).context(builder::Subversion)?;
        }

        Ok(())
    }
}
