use super::*;

#[derive(Debug, Default, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UpdateOptions {
    paths: Vec<String>,
    revision: Revision,
    depth: Depth,
    depth_is_sticky: bool,
    ignore_externals: bool,
    allow_unver_obstructions: bool,
    adds_as_modification: bool,
    make_parents: bool,
}

impl Context {
    pub fn update(&mut self, opts: UpdateOptions) -> error::Result<Vec<Option<RevisionNumber>>> {
        unsafe {
            let mut pool = apr::Pool::create();
            let paths = pool.canonicalize_dirent_array(opts.paths.len(), opts.paths.iter())?;
            let revision = opts.revision.to_opt_revision();
            let depth = opts.depth.into();
            let mut revisions: *mut ffi::apr_array_header_t = std::ptr::null_mut();
            let error = ffi::svn_client_update4(
                revisions.pointer_mut(),
                paths,
                revision.pointer(),
                depth,
                opts.depth_is_sticky.into(),
                opts.ignore_externals.into(),
                opts.allow_unver_obstructions.into(),
                opts.adds_as_modification.into(),
                opts.make_parents.into(),
                self.ctx(),
                pool.as_mut_ptr(),
            );
            SubversionError::from_nullable_ptr(error).context(builder::Subversion)?;
            assert!(!revisions.is_null(), "Expected non-null revisions");

            use apr::AprArray;

            Ok(revisions.to_value_vec(|ptr| (*(ptr as *const ffi::svn_revnum_t)).try_into().ok()))
        }
    }
}
