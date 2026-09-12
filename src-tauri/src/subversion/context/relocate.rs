use super::*;

#[derive(Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RelocateOptions {
    pub wc_root_dir: String,
    pub from_prefix: String,
    pub to_prefix: String,
    pub ignore_externals: bool,
}

impl Context {
    pub fn relocate(&mut self, opts: RelocateOptions) -> error::Result<()> {
        unsafe {
            let mut pool = apr::Pool::create();
            let root = pool.string(opts.wc_root_dir)?;
            let from = pool.string(opts.from_prefix)?;
            let to = pool.string(opts.to_prefix)?;
            let error = ffi::svn_client_relocate2(
                root,
                from,
                to,
                opts.ignore_externals.into(),
                self.ctx(),
                pool.as_mut_ptr(),
            );
            SubversionError::from_nullable_ptr(error).context(builder::Subversion)?;
            Ok(())
        }
    }
}
