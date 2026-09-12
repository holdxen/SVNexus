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
pub struct AddOptions {
    path: String,
    depth: Depth,
    force: bool,
    no_ignore: bool,
    no_auto_properties: bool,
    add_parents: bool,
}

impl Context {
    pub fn add(&mut self, opts: AddOptions) -> error::Result<()> {
        unsafe {
            let mut pool = apr::Pool::create();
            let path = pool.canonicalize_dirent(opts.path.as_str())?;
            let error = ffi::svn_client_add5(
                path,
                opts.depth.into(),
                opts.force.into(),
                opts.no_ignore.into(),
                opts.no_auto_properties.into(),
                opts.add_parents.into(),
                self.ctx(),
                pool.as_mut_ptr(),
            );
            SubversionError::from_nullable_ptr(error).context(builder::Subversion)
        }
    }
}
