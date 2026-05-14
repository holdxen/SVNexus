use super::*;

#[derive(uniffi::Record, Debug)]
pub struct PatchOptions {
    patch_absolute_path: String,
    wc_absolute_path: String,
    dry_run: bool,
    strip_count: u32,
    reverse: bool,
    ignore_whitespace: bool,
    remove_tempfiles: bool,
}

impl Context {
    pub fn patch(&mut self, opts: PatchOptions) -> error::Result<()> {
        unsafe extern "C" fn patch_filter(
            _baton: *mut c_void,
            _filtered: *mut ffi::svn_boolean_t,
            _canon_path_from_patchfile: *const c_char,
            _patch_abspath: *const c_char,
            _reject_abspath: *const c_char,
            _scratch_pool: *mut ffi::apr_pool_t,
        ) -> *mut ffi::svn_error_t {
            svn_no_error()
        }
        unsafe {
            let mut pool = apr::Pool::create();

            let patch = pool.canonicalize_dirent(&opts.patch_absolute_path)?;

            let wc = pool.canonicalize_dirent(&opts.wc_absolute_path)?;

            let error = ffi::svn_client_patch(
                patch,
                wc,
                opts.dry_run.into(),
                opts.strip_count.try_into().unwrap(),
                opts.reverse.into(),
                opts.ignore_whitespace.into(),
                opts.remove_tempfiles.into(),
                Some(patch_filter),
                self.inner.inner_void_pointer_mut(),
                self.ctx(),
                pool.as_mut_ptr(),
            );

            SVNError::from_nullable_ptr(error).context(builder::Svn)?;
            Ok(())
        }
    }
}
