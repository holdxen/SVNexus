use std::collections::HashMap;
use std::sync::Arc;

use derive_new::new;
use snafu::ResultExt;

use super::context;
use super::ffi;
use crate::apr::AprPool;
use crate::apr::Pool;
use crate::error;
use crate::error::builder;
use crate::extensions::*;
use crate::subversion::SubversionError;
use crate::utils::CStringer;
use crate::utils::Pointer;

#[derive(new)]
pub struct AsyncContext {
    inner: Arc<ContextInner>,
}

impl AsyncContext {
    async fn call_async<F, R>(&self, call: F) -> error::Result<R>
    where
        F: (FnOnce(*mut ffi::svn_ra_session_t) -> error::Result<R>) + Send + 'static,
        R: Send + 'static,
    {
        let context = self.inner.clone();
        let result = tokio::task::spawn_blocking(move || {
            let _client = context.client.lock();
            call(context.session)
        })
        .await
        .context(builder::Runtime)??;

        Ok(result)
    }
}

#[derive(new)]
pub struct ContextInner {
    client: Arc<parking_lot::FairMutex<context::Context>>,
    pool: *mut ffi::apr_pool_t,
    session: *mut ffi::svn_ra_session_t,
}

/// SAFETY: every operation of this struct will be locked
unsafe impl Send for ContextInner {}
unsafe impl Sync for ContextInner {}

impl Drop for ContextInner {
    fn drop(&mut self) {
        let _client = self.client.lock();
        unsafe {
            ffi::apr_pool_destroy(self.pool);
        }
    }
}

impl AsyncContext {
    pub async fn get_latest_revision_number(&self) -> error::Result<u32> {
        self.call_async(|session| unsafe {
            let mut number: ffi::svn_revnum_t = 0;
            let mut pool = Pool::create();
            let error =
                ffi::svn_ra_get_latest_revnum(session, number.pointer_mut(), pool.as_mut_ptr());

            SubversionError::from_nullable_ptr(error).context(builder::Subversion)?;

            Ok(number.try_into().expect("Failed to convert number"))
        })
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_locations(
        &self,
        path: String,
        revision: u32,
        location_revisions: Vec<u32>,
    ) -> error::Result<HashMap<u32, String>> {
        self.call_async(move |session| unsafe {
            let mut pool = Pool::create();

            let path = pool.canonicalize_relpath(&path)?;

            let mut locations: *mut ffi::apr_hash_t = std::ptr::null_mut();

            let len = location_revisions.len();
            let location_revisions = location_revisions
                .into_iter()
                .map(|rev| rev as ffi::svn_revnum_t);

            let location_revisions = pool.value_array(len, location_revisions)?;

            let error = ffi::svn_ra_get_locations(
                session,
                locations.pointer_mut(),
                path,
                revision.try_into().expect("Failed to convert revision number"),
                location_revisions,
                pool.as_mut_ptr(),
            );

            SubversionError::from_nullable_ptr(error).context(builder::Subversion)?;

            let locations = pool.as_mut_ptr().hash_map(locations, |(k, v)| {
                (
                    u32::try_from(*(k as *const ffi::svn_revnum_t)).expect("Unexpected failure"),
                    (v as *const std::ffi::c_char).to_str().to_string(),
                )
            });
            Ok(locations)
        })
        .await
    }
}
