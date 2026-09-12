use std::ffi::c_char;

use serde::{Deserialize, Serialize};
use snafu::ResultExt;

use crate::{
    apr,
    error::{self, builder},
    utils::Pointer,
};

use super::{ffi, Context, Depth, Revision, RevisionNumber, SubversionError};

#[derive(Debug, Clone, Copy, strum::Display, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum NativeEOL {
    LF,
    CRLF,
    CR,
    None,
}

impl NativeEOL {
    fn is_none(self) -> bool {
        matches!(self, NativeEOL::None)
    }
}

#[derive(Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ExportOptions {
    from_path_or_url: String,
    to_path: String,
    peg_revision: Revision,
    revision: Revision,
    r#override: bool,
    ignore_externals: bool,
    ignore_keywords: bool,
    depth: Depth,
    native_eol: NativeEOL,
}

impl Context {
    pub fn export(&mut self, opts: ExportOptions) -> error::Result<Option<RevisionNumber>> {
        unsafe {
            let mut pool = apr::Pool::create();
            let native_eol = if opts.native_eol.is_none() {
                std::ptr::null_mut()
            } else {
                pool.string(opts.native_eol.to_string())?
            };

            let mut revision_number: ffi::svn_revnum_t = 0;

            let from_path_or_url = opts.from_path_or_url.replace('\\', "/");

            let mut from_path_or_url = pool.string(&from_path_or_url)? as *const c_char;
            let to_path = pool.string(opts.to_path)?;
            let to_path = ffi::svn_dirent_canonicalize(to_path, pool.as_mut_ptr());
            let peg_revision = opts.peg_revision.to_opt_revision();
            let revision = opts.revision.to_opt_revision();

            if ffi::svn_path_is_url(from_path_or_url) != 0 {
                from_path_or_url =
                    Self::check_url(from_path_or_url, &opts.from_path_or_url, &mut pool)?;
            }

            let error = ffi::svn_client_export5(
                revision_number.pointer_mut(),
                from_path_or_url,
                to_path,
                peg_revision.pointer(),
                revision.pointer(),
                opts.r#override.into(),
                opts.ignore_externals.into(),
                opts.ignore_keywords.into(),
                opts.depth.into(),
                native_eol,
                self.ctx(),
                pool.as_mut_ptr(),
            );
            SubversionError::from_nullable_ptr(error).context(builder::Subversion)?;
            Ok(RevisionNumber::try_from(revision_number).ok())
        }
    }
}
