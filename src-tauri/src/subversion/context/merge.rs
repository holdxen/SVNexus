use crate::{
    apr,
    error::{self, builder},
    extensions::Canonicalization,
    utils::Pointer,
};

use super::{ffi, Context, Depth, Revision, RevisionRange, SubversionError};
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

#[derive(Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum MergeSource {
    #[serde(rename_all = "camelCase")]
    Target {
        source1: String,
        revision1: Revision,
        source2: String,
        revision2: Revision,
    },
    #[serde(rename_all = "camelCase")]
    Peg {
        source: String,
        peg_revision: Revision,
        ranges_to_merge: Option<Vec<RevisionRange>>,
    },
}

#[derive(Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MergeOptions {
    source: MergeSource,
    target: String,
    depth: Depth,
    ignore_merge_info: bool,
    ignore_ancestry: bool,
    force_delete: bool,
    record_only: bool,
    dry_run: bool,
    allow_mixed_revision: bool,
    extra_merge_options: Option<Vec<String>>,
}

impl Context {
    pub fn merge(&mut self, opts: MergeOptions) -> error::Result<()> {
        unsafe {
            let mut pool = apr::Pool::create();

            let target = pool.canonicalize_target(&opts.target)?;

            let merge_options = opts
                .extra_merge_options
                .map(|p| pool.string_array(p.len(), p.iter()))
                .transpose()?
                .unwrap_or_default();

            let error = match opts.source {
                MergeSource::Target {
                    source1,
                    revision1,
                    source2,
                    revision2,
                } => {
                    let source1 = Self::canonicalize_path_or_url(&source1, &mut pool)?;
                    let revision1 = revision1.to_opt_revision();
                    let source2 = Self::canonicalize_path_or_url(&source2, &mut pool)?;
                    let revision2 = revision2.to_opt_revision();

                    ffi::svn_client_merge5(
                        source1,
                        revision1.pointer(),
                        source2,
                        revision2.pointer(),
                        target,
                        opts.depth.into(),
                        opts.ignore_merge_info.into(),
                        opts.ignore_ancestry.into(),
                        opts.force_delete.into(),
                        opts.record_only.into(),
                        opts.dry_run.into(),
                        opts.allow_mixed_revision.into(),
                        merge_options,
                        self.ctx(),
                        pool.as_mut_ptr(),
                    )
                }
                MergeSource::Peg {
                    source,
                    peg_revision,
                    ranges_to_merge,
                } => {
                    let source = Self::canonicalize_path_or_url(&source, &mut pool)?;
                    let peg_revision = peg_revision.to_opt_revision();
                    let ranges = if let Some(ranges_to_merge) = ranges_to_merge {
                        pool.revision_range(&ranges_to_merge)
                    } else {
                        std::ptr::null_mut()
                    };

                    ffi::svn_client_merge_peg5(
                        source,
                        ranges,
                        peg_revision.pointer(),
                        target,
                        opts.depth.into(),
                        opts.ignore_merge_info.into(),
                        opts.ignore_ancestry.into(),
                        opts.force_delete.into(),
                        opts.record_only.into(),
                        opts.dry_run.into(),
                        opts.allow_mixed_revision.into(),
                        merge_options,
                        self.ctx(),
                        pool.as_mut_ptr(),
                    )
                }
            };

            SubversionError::from_nullable_ptr(error).context(builder::Subversion)?;

            Ok(())
        }
    }

    unsafe fn canonicalize_path_or_url(
        path_or_url: &str,
        pool: &mut apr::Pool,
    ) -> error::Result<*const std::ffi::c_char> {
        unsafe {
            let c_str = pool.string(path_or_url)?;
            if ffi::svn_path_is_url(c_str) != 0 {
                pool.canonicalize_uri(path_or_url)
            } else {
                pool.canonicalize_target(path_or_url)
            }
        }
    }
}
