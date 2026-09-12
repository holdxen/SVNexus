use serde::{Deserialize, Serialize};
use snafu::ResultExt;

use crate::{
    apr,
    error::{self, builder},
    extensions::Canonicalization,
};

use super::{ffi, Context, SubversionError, Version};

#[derive(Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UpgradeOptions {
    path: String,
}

#[derive(Debug, Default, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UpgradeResult {
    format_version: Option<Version>,
}

impl Context {
    pub fn upgrade(&mut self, opts: UpgradeOptions) -> error::Result<UpgradeResult> {
        unsafe {
            let mut pool = apr::Pool::create();

            let path = pool.canonicalize_dirent(&opts.path)?;

            let mut result_format_version: *const ffi::svn_version_t = std::ptr::null();

            let error = ffi::svn_client_upgrade2(
                &mut result_format_version as *mut _,
                path,
                std::ptr::null(),
                self.ctx(),
                pool.as_mut_ptr(),
                pool.as_mut_ptr(),
            );

            SubversionError::from_nullable_ptr(error).context(builder::Subversion)?;

            let format_version = if result_format_version.is_null() {
                None
            } else {
                Some(Version::from(result_format_version))
            };

            Ok(UpgradeResult { format_version })
        }
    }
}
