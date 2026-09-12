use std::collections::HashMap;
use std::ffi::{c_char, c_void, CStr};

use serde::{Deserialize, Serialize};
use snafu::ResultExt;

use crate::{
    apr::{self},
    error::{self, builder},
    extensions::Canonicalization,
    utils::Pointer,
};

use super::{ffi, Context, DifferenceFileOptions, Revision, RevisionNumber, SubversionError};

#[derive(Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BlameOptions {
    path: String,
    peg_revision: Revision,
    start_revision: Revision,
    end_revision: Revision,
    difference_options: Option<DifferenceFileOptions>,
    ignore_mime_type: bool,
    include_merged_revisions: bool,
}

#[derive(Debug, Default, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BlameResult {
    entries: Vec<BlameEntry>,
}

#[derive(Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BlameEntry {
    line_no: i64,
    revision: Option<RevisionNumber>,
    author: Option<String>,
    date: Option<String>,
    line: String,
    merged_revision: Option<RevisionNumber>,
    merged_author: Option<String>,
    merged_date: Option<String>,
    merged_path: Option<String>,
    local_change: bool,
}

impl Context {
    pub fn blame(&mut self, opts: BlameOptions) -> error::Result<BlameResult> {
        unsafe {
            let mut pool = apr::Pool::create();

            let path = pool.canonicalize_target(&opts.path)?;
            let peg_revision = opts.peg_revision.to_opt_revision();
            let start_revision = opts.start_revision.to_opt_revision();
            let end_revision = opts.end_revision.to_opt_revision();

            let diff_options = if let Some(diff_opts) = opts.difference_options {
                let file_options = ffi::svn_diff_file_options_create(pool.as_mut_ptr());
                diff_opts.setup(
                    file_options
                        .as_mut()
                        .expect("Failed to get mutable reference to file options"),
                );
                file_options
            } else {
                std::ptr::null_mut()
            };

            let mut result = BlameResult::default();

            unsafe extern "C" fn receiver(
                baton: *mut c_void,
                _start_revnum: ffi::svn_revnum_t,
                _end_revnum: ffi::svn_revnum_t,
                line_no: ffi::apr_int64_t,
                revision: ffi::svn_revnum_t,
                rev_props: *mut ffi::apr_hash_t,
                merged_revision: ffi::svn_revnum_t,
                merged_rev_props: *mut ffi::apr_hash_t,
                merged_path: *const c_char,
                line: *const c_char,
                local_change: ffi::svn_boolean_t,
                _pool: *mut ffi::apr_pool_t,
            ) -> *mut ffi::svn_error_t {
                unsafe {
                    let result = (baton as *mut BlameResult)
                        .as_mut()
                        .expect("Failed to cast baton to mutable reference");

                    let rev_props_map = if rev_props.is_null() {
                        HashMap::new()
                    } else {
                        let mut pool = apr::Pool::create();
                        pool.convert_to_hash_map(rev_props)
                    };

                    let merged_rev_props_map = if merged_rev_props.is_null() {
                        HashMap::new()
                    } else {
                        let mut pool = apr::Pool::create();
                        pool.convert_to_hash_map(merged_rev_props)
                    };

                    let line_str = if line.is_null() {
                        String::new()
                    } else {
                        CStr::from_ptr(line)
                            .to_str()
                            .unwrap_or_default()
                            .to_string()
                    };

                    let entry = BlameEntry {
                        line_no: line_no as i64,
                        revision: if revision >= 0 {
                            Some(revision.try_into().expect("Failed to convert revision"))
                        } else {
                            None
                        },
                        author: rev_props_map.get("svn:author").cloned(),
                        date: rev_props_map.get("svn:date").cloned(),
                        line: line_str,
                        merged_revision: if merged_revision >= 0 {
                            Some(
                                merged_revision
                                    .try_into()
                                    .expect("Failed to convert merged revision"),
                            )
                        } else {
                            None
                        },
                        merged_author: merged_rev_props_map.get("svn:author").cloned(),
                        merged_date: merged_rev_props_map.get("svn:date").cloned(),
                        merged_path: if merged_path.is_null() {
                            None
                        } else {
                            Some(
                                CStr::from_ptr(merged_path)
                                    .to_str()
                                    .unwrap_or_default()
                                    .to_string(),
                            )
                        },
                        local_change: local_change != 0,
                    };

                    result.entries.push(entry);
                }

                super::svn_no_error()
            }

            let error = ffi::svn_client_blame5(
                path,
                &peg_revision as *const _,
                &start_revision as *const _,
                &end_revision as *const _,
                diff_options,
                opts.ignore_mime_type as ffi::svn_boolean_t,
                opts.include_merged_revisions as ffi::svn_boolean_t,
                Some(receiver),
                result.pointer_mut() as *mut c_void,
                self.ctx(),
                pool.as_mut_ptr(),
            );

            SubversionError::from_nullable_ptr(error).context(builder::Subversion)?;

            Ok(result)
        }
    }
}
