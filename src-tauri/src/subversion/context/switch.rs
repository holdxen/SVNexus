use derive_new::new;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

use crate::{
    apr,
    error::{self, builder},
    utils::Pointer,
};

use super::{ffi, Context, Depth, Revision, RevisionNumber, SubversionError};

#[derive(new, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SwitchOptions {
    path: String,
    url: String,
    peg_revision: Revision,
    revision: Revision,
    depth: Depth,
    depth_is_sticky: bool,
    ignore_externals: bool,
    allow_unversioned_obstructions: bool,
    ignore_ancestry: bool,
}

impl Context {
    pub fn switch(&mut self, opts: SwitchOptions) -> error::Result<RevisionNumber> {
        unsafe {
            let mut pool = apr::Pool::create();
            let path = pool.string(opts.path)?;
            let url = pool.string(opts.url)?;
            let peg_revision = opts.peg_revision.to_opt_revision();
            let revision = opts.revision.to_opt_revision();
            let depth = opts.depth.into();
            let depth_is_sticky = opts.depth_is_sticky.into();
            let ignore_externals = opts.ignore_externals.into();
            let allow_unversioned_obstructions = opts.allow_unversioned_obstructions.into();
            let ignore_ancestry = opts.ignore_ancestry.into();
            let mut result: ffi::svn_revnum_t = 0;
            let error = ffi::svn_client_switch3(
                result.pointer_mut(),
                path,
                url,
                peg_revision.pointer(),
                revision.pointer(),
                depth,
                depth_is_sticky,
                ignore_externals,
                allow_unversioned_obstructions,
                ignore_ancestry,
                self.ctx(),
                pool.as_mut_ptr(),
            );
            SubversionError::from_nullable_ptr(error).context(builder::Subversion)?;

            Ok(result.try_into().expect("Unexpected revision"))
        }
    }
}
