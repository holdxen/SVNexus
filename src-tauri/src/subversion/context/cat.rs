use std::collections::HashMap;

use derive_new::new;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

use crate::{
    apr,
    error::{self, builder},
    extensions::Canonicalization,
    subversion::stream::Stream,
    utils::Pointer,
};

use super::{ffi, Context, Revision, SubversionError};

#[derive(Debug, new, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CatOptions {
    path: String,
    peg_revision: Revision,
    revision: Revision,
    expand_keywords: bool,
    get_properties: bool,
}

#[derive(Debug, new, Serialize, Deserialize, Default, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CatResult {
    #[ts(type = "Uint8Array")]
    #[serde(with = "serde_bytes")]
    content: Vec<u8>,
    properties: Option<HashMap<String, String>>,
}

impl Context {
    pub fn cat(&mut self, opts: CatOptions) -> error::Result<CatResult> {
        unsafe {
            let mut properties: *mut ffi::apr_hash_t = std::ptr::null_mut();

            let mut pool = apr::Pool::create();

            let mut stream = Stream::create(Default::default());

            let path = pool.canonicalize_target(&opts.path)?;

            let peg_revision = opts.peg_revision.to_opt_revision();

            let revision = opts.revision.to_opt_revision();

            let error = ffi::svn_client_cat3(
                if opts.get_properties {
                    properties.pointer_mut()
                } else {
                    std::ptr::null_mut()
                },
                stream.ptr(),
                path,
                &peg_revision as *const _,
                &revision as *const _,
                opts.expand_keywords.into(),
                self.ptr,
                pool.as_mut_ptr(),
                pool.as_mut_ptr(),
            );

            SubversionError::from_nullable_ptr(error).context(builder::Subversion)?;
            let content = stream.take_write_buffer();

            let properties = if properties.is_null() {
                None
            } else {
                Some(pool.convert_to_hash_map(properties))
            };

            Ok(CatResult {
                content,
                properties,
            })
        }
    }
}
