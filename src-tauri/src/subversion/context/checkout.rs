use derive_new::new;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

use crate::{
    apr,
    error::{self, builder},
    extensions::Canonicalization,
};

use super::{ffi, Context, Depth, Revision, RevisionNumber, SubversionError};

#[derive(new, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CheckoutOptions {
    pub url: String,
    pub path: String,
    pub peg_revision: Revision,
    pub revision: Revision,
    pub depth: Depth,
    pub ignore_externals: bool,
    pub allow_unversioned_obstructions: bool,
    pub store_pristine: Option<bool>,
}

impl Context {
    pub fn checkout(&mut self, mut opts: CheckoutOptions) -> error::Result<RevisionNumber> {
        if opts.url.is_empty() {
            return error::builder::InvalidArgument {
                detail: "Empty url",
            }
            .fail();
        }
        if opts.path.is_empty() {
            return error::builder::InvalidArgument {
                detail: "Empty path",
            }
            .fail();
        }
        let revision = opts.revision.to_opt_revision();
        let peg_revision = opts.peg_revision.to_opt_revision();

        let mut result_revision: ffi::svn_revnum_t = 0;

        let mut pool = unsafe { apr::Pool::create() };

        let path = unsafe { pool.string(opts.path.as_str()) }?;

        let url = pool.canonicalize_uri(&opts.url)?;

        if opts.depth == Depth::Unknown {
            opts.depth = Depth::Infinity
        }

        let store_pristine = opts
            .store_pristine
            .and_then(|v| {
                Some(if v {
                    ffi::svn_tristate_t_svn_tristate_true
                } else {
                    ffi::svn_tristate_t_svn_tristate_false
                })
            })
            .unwrap_or(ffi::svn_tristate_t_svn_tristate_unknown);

        tracing::info!("Start checking using c function");
        let error = unsafe {
            ffi::svn_client_checkout4(
                &mut result_revision as *mut _,
                url,
                path,
                &revision as *const _,
                &peg_revision as *const _,
                opts.depth.into(),
                opts.ignore_externals.into(),
                opts.allow_unversioned_obstructions.into(),
                std::ptr::null(),
                store_pristine,
                self.ptr,
                pool.as_mut_ptr(),
            )
        };
        tracing::info!("Finish checking using c function");

        SubversionError::from_nullable_ptr(error).context(builder::Subversion)?;

        Ok(result_revision
            .try_into()
            .expect("Failed to convert revision number"))
    }
}
