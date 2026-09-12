use serde::{Deserialize, Serialize};
use snafu::ResultExt;

use crate::{
    apr,
    error::{self, builder},
    extensions::Canonicalization,
};

use super::{ffi, Context, SubversionError};

#[derive(Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct VacuumOptions {
    path: String,
    remove_unversioned_items: bool,
    remove_ignored_items: bool,
    fix_recorded_timestamps: bool,
    vacuum_pristines: bool,
    include_externals: bool,
}

impl Context {
    pub fn vacuum(&mut self, opts: VacuumOptions) -> error::Result<()> {
        unsafe {
            let mut pool = apr::Pool::create();

            let path = pool.canonicalize_dirent(&opts.path)?;

            let error = ffi::svn_client_vacuum(
                path,
                opts.remove_unversioned_items.into(),
                opts.remove_ignored_items.into(),
                opts.fix_recorded_timestamps.into(),
                opts.vacuum_pristines.into(),
                opts.include_externals.into(),
                self.ctx(),
                pool.as_mut_ptr(),
            );

            SubversionError::from_nullable_ptr(error).context(builder::Subversion)?;

            Ok(())
        }
    }
}
