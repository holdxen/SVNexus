use super::*;

#[derive(Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CleanupOptions {
    pub path: String,
    break_locks: bool,
    fix_recorded_timestamps: bool,
    clear_dav_cache: bool,
    vacuum_pristines: bool,
    include_externals: bool,
}

impl Context {
    pub fn cleanup(&mut self, opts: CleanupOptions) -> error::Result<()> {
        unsafe {
            let mut pool = apr::Pool::create();

            let path = pool.string(opts.path)?;

            let path = ffi::svn_dirent_canonicalize(path, pool.as_mut_ptr());

            let error = ffi::svn_client_cleanup2(
                path,
                opts.break_locks.into(),
                opts.fix_recorded_timestamps.into(),
                opts.clear_dav_cache.into(),
                opts.vacuum_pristines.into(),
                opts.include_externals.into(),
                self.ctx(),
                pool.as_mut_ptr(),
            );
            SubversionError::from_nullable_ptr(error).context(builder::Subversion)?;
        }

        Ok(())
    }
}
