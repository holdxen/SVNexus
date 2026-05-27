use super::super::ffi;
// use super::{CommitInfo, CommitItem, Context, Depth, SVNError, commit_callback, svn_no_error};
use super::*;
use crate::error::builder;
use crate::extensions::{Canonicalization, CommonExtension};
use crate::utils::{Boxed, CStringer, Pointer};
use crate::{apr, error};
use derive_new::new;
use snafu::ResultExt;
use std::collections::HashMap;
use std::ffi::{c_char, c_void};

#[derive(new, Debug, uniffi::Record)]
pub struct ImportResult {
    pub items: Vec<CommitItem>,
    pub info: CommitInfo,
}

#[uniffi::export(with_foreign)]
pub trait ImportFilter: Send + Sync + 'static {
    fn filter(
        &self,
        path: String,
        kind: NodeKind,
        special: bool,
        file_size: Option<u64>,
        mtime: i64,
    ) -> error::Result<bool, error::CSharpError>;
}

#[derive(new, Debug, uniffi::Record)]
pub struct ImportOptions {
    pub path: String,
    pub url: String,
    pub depth: Depth,
    pub no_ignore: bool,
    pub no_autoprops: bool,
    pub ignore_unknown_node_types: bool,
    pub revision_property_table: Option<HashMap<String, String>>,
    pub commit_message: String,
    // pub filters: Option<Vec<String>>,
}

impl Context {
    pub fn import_filter<'c, 'b>(
        &mut self,
        opts: ImportOptions,
        filter: Option<
            &'b mut (
                        dyn FnMut(
                /*path*/ String,
                /*kind*/ NodeKind,
                /*special*/ bool,
                /*file_size*/ Option<u64>,
                /*mtime*/ i64,
            ) -> error::Result<bool, error::CSharpError>
                            + 'c
                    ),
        >,
    ) -> error::Result<ImportResult> {
        // use ignore::gitignore::*;
        struct Filter<'a> {
            filter: &'a mut dyn FnMut(
                /*path*/ String,
                /*kind*/ NodeKind,
                /*special*/ bool,
                /*file_size*/ Option<u64>,
                /*mtime*/ i64,
            ) -> error::Result<bool, error::CSharpError>,
        }
        unsafe extern "C" fn filter_none(
            _: *mut c_void,
            _: *mut ffi::svn_boolean_t,
            _: *const c_char,
            _: *const ffi::svn_io_dirent2_t,
            _: *mut ffi::apr_pool_t,
        ) -> *mut ffi::svn_error_t {
            svn_no_error()
        }
        unsafe extern "C" fn filter_some(
            baton: *mut c_void,
            filtered: *mut ffi::svn_boolean_t,
            local_abspath: *const c_char,
            direct: *const ffi::svn_io_dirent2_t,
            _pool: *mut ffi::apr_pool_t,
        ) -> *mut ffi::svn_error_t {
            unsafe {
                let filter = &mut (baton as *mut Filter).as_mut().unwrap().filter;
                let path = local_abspath.to_str().to_string();

                let (kind, special, file_size, mtime) = direct.as_ref().unwrap().also_map(|e| {
                    (
                        NodeKind::try_from(e.kind).unwrap(),
                        e.special != 0,
                        u64::try_from(e.filesize).ok(),
                        i64::try_from(e.mtime).unwrap(),
                    )
                });

                match filter(path, kind, special, file_size, mtime) {
                    Ok(f) => *filtered = f.into(),
                    Err(e) => return e.native_error(),
                }
            }
            svn_no_error()
        }
        // unsafe extern "C" fn filter_none(
        //     _: *mut c_void,
        //     _: *mut ffi::svn_boolean_t,
        //     _: *const c_char,
        //     _: *const ffi::svn_io_dirent2_t,
        //     _: *mut ffi::apr_pool_t,
        // ) -> *mut ffi::svn_error_t {
        //     svn_no_error()
        // }

        // let mut matcher = opts
        //     .filters
        //     .map(|f| {
        //         let mut builder = GitignoreBuilder::new(&opts.path);

        //         for i in f {
        //             tracing::info!("Add line to filter: {i}");
        //             builder.add_line(None, &i)?;
        //         }

        //         Ok(Filter {
        //             matcher: builder.build()?,
        //         })
        //     })
        //     .transpose()
        //     .context(builder::Glob)?;

        // let baton = matcher
        //     .as_mut()
        //     .map(|v| v.pointer_mut())
        //     .unwrap_or_default();
        //
        let mut filter = filter.map(|e| Filter { filter: e });

        let baton = filter.as_mut().map(|e| e.pointer_mut()).unwrap_or_default();

        unsafe {
            let mut pool = apr::Pool::create();
            if opts.commit_message.as_bytes().contains(&b'\0') {
                return builder::General {
                    detail: "Invalid commit message",
                }
                .fail();
            }
            self.inner.commit_message = opts.commit_message;
            self.inner.commit_items.clear();

            let table = opts
                .revision_property_table
                .map(|t| {
                    pool.string_hash_map(
                        t.iter(),
                        |p, k| p.string(k).map(|o| o as _),
                        |p, v| p.svn_string(v).map(|o| o as _),
                    )
                })
                .transpose()?
                .unwrap_or_default();

            let error = ffi::svn_client_import5(
                pool.canonicalize_dirent(&opts.path)?,
                pool.canonicalize_uri(&opts.url)?,
                opts.depth.into(),
                opts.no_ignore.into(),
                opts.no_autoprops.into(),
                opts.ignore_unknown_node_types.into(),
                table,
                Some(if baton.is_null() {
                    filter_none
                } else {
                    filter_some
                }),
                baton as _,
                Some(commit_callback),
                self.inner.inner_void_pointer_mut(),
                self.ctx(),
                pool.as_mut_ptr(),
            );

            SVNError::from_nullable_ptr(error).context(builder::Svn)?;
        }
        Ok(ImportResult::new(
            std::mem::take(&mut self.inner.commit_items),
            std::mem::take(&mut self.inner.commit_info).unwrap_or_default(),
        ))
    }
    pub fn import(
        &mut self,
        opts: ImportOptions,
        filters: Option<Vec<String>>,
    ) -> error::Result<ImportResult> {
        use ignore::gitignore::*;
        struct Filter {
            matcher: Gitignore,
        }
        unsafe extern "C" fn filter(
            baton: *mut c_void,
            filtered: *mut ffi::svn_boolean_t,
            local_abspath: *const c_char,
            direct: *const ffi::svn_io_dirent2_t,
            _pool: *mut ffi::apr_pool_t,
        ) -> *mut ffi::svn_error_t {
            unsafe {
                let absolute_path = local_abspath.to_str();

                let f = (baton as *mut Filter).as_mut().unwrap();

                let direct = direct.as_ref().unwrap();

                let relate_to = f.matcher.path().to_str().expect("Unexpected path");

                if absolute_path.starts_with(relate_to) {
                    let matched = f.matcher.matched(
                        absolute_path
                            .trim_start_matches(relate_to)
                            .trim_start_matches("/"),
                        direct.kind == ffi::svn_node_kind_t_svn_node_dir,
                    );
                    // tracing::info!(
                    //     "absolute_path={}, relate_to={}, ignore={}, kind={}",
                    //     absolute_path,
                    //     relate_to,
                    //     matched.is_ignore(),
                    //     direct.kind
                    // );
                    *filtered = matched.is_ignore().into();
                } else {
                    tracing::error!(
                        "Unexpected import file: {}, should from: {}",
                        absolute_path,
                        relate_to
                    );
                }
            }

            svn_no_error()
        }
        unsafe extern "C" fn filter_none(
            _: *mut c_void,
            _: *mut ffi::svn_boolean_t,
            _: *const c_char,
            _: *const ffi::svn_io_dirent2_t,
            _: *mut ffi::apr_pool_t,
        ) -> *mut ffi::svn_error_t {
            svn_no_error()
        }

        let mut matcher = filters
            .map(|f| {
                let mut builder = GitignoreBuilder::new(&opts.path);

                for i in f {
                    tracing::info!("Add line to filter: {i}");
                    builder.add_line(None, &i)?;
                }

                Ok(Filter {
                    matcher: builder.build()?,
                })
            })
            .transpose()
            .context(builder::Glob)?;

        let baton = matcher
            .as_mut()
            .map(|v| v.pointer_mut())
            .unwrap_or_default();

        unsafe {
            let mut pool = apr::Pool::create();
            if opts.commit_message.as_bytes().contains(&b'\0') {
                return builder::General {
                    detail: "Invalid commit message",
                }
                .fail();
            }
            self.inner.commit_message = opts.commit_message;
            self.inner.commit_items.clear();

            let table = opts
                .revision_property_table
                .map(|t| {
                    pool.string_hash_map(
                        t.iter(),
                        |p, k| p.string(k).map(|o| o as _),
                        |p, v| p.svn_string(v).map(|o| o as _),
                    )
                })
                .transpose()?
                .unwrap_or_default();

            let error = ffi::svn_client_import5(
                pool.canonicalize_dirent(&opts.path)?,
                pool.canonicalize_uri(&opts.url)?,
                opts.depth.into(),
                opts.no_ignore.into(),
                opts.no_autoprops.into(),
                opts.ignore_unknown_node_types.into(),
                table,
                Some(if baton.is_null() { filter_none } else { filter }),
                baton as _,
                Some(commit_callback),
                self.inner.inner_void_pointer_mut(),
                self.ctx(),
                pool.as_mut_ptr(),
            );

            SVNError::from_nullable_ptr(error).context(builder::Svn)?;
        }
        Ok(ImportResult::new(
            std::mem::take(&mut self.inner.commit_items),
            std::mem::take(&mut self.inner.commit_info).unwrap_or_default(),
        ))
    }
}
