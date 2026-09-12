use derive_new::new;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

use crate::{
    apr,
    error::{self, builder},
    extensions::Canonicalization,
};

use super::{ffi, Context, SubversionError};

#[derive(new, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LockOptions {
    targets: Vec<String>,
    comment: Option<String>,
    steal_lock: bool,
}

#[derive(new, Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UnlockOptions {
    targets: Vec<String>,
    break_lock: bool,
}

impl Context {
    pub fn lock(&mut self, opts: LockOptions) -> error::Result<()> {
        unsafe {
            let mut pool = apr::Pool::create();
            let targets =
                pool.canonicalize_target_array(opts.targets.len(), opts.targets.iter())?;

            let comment = opts
                .comment
                .map(|v| pool.string(v))
                .transpose()?
                .unwrap_or_default();

            let error = ffi::svn_client_lock(
                targets,
                comment,
                opts.steal_lock.into(),
                self.ctx(),
                pool.as_mut_ptr(),
            );
            SubversionError::from_nullable_ptr(error).context(builder::Subversion)?;
        }
        Ok(())
    }

    pub fn unlock(&mut self, opts: UnlockOptions) -> error::Result<()> {
        unsafe {
            let mut pool = apr::Pool::create();
            let targets =
                pool.canonicalize_target_array(opts.targets.len(), opts.targets.iter())?;

            let error = ffi::svn_client_unlock(
                targets,
                opts.break_lock.into(),
                self.ctx(),
                pool.as_mut_ptr(),
            );
            SubversionError::from_nullable_ptr(error).context(builder::Subversion)?;
        }
        Ok(())
    }
}
