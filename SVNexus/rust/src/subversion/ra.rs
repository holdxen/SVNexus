use std::collections::HashMap;
use std::sync::Arc;

use derive_new::new;
use snafu::ResultExt;

use super::context;
use super::ffi;
use crate::apr::AprPool;
use crate::apr::AutoPool;
use crate::apr::Pool;
use crate::error;
use crate::error::builder;
use crate::subversion::SVNError;
use crate::utils::Pointer;
use crate::utils::SubversionStringer;
use crate::utils::CStringer;

#[derive(uniffi::Object, new)]
#[uniffi(name = "RepositoryAccessAsyncContext")]
pub struct AsyncContext {
    inner: Arc<ContextInner>,
}

impl AsyncContext {
    async fn call_async<F, R>(&self, call: F) -> error::Result<R>
    where
        F: (FnOnce(&mut AutoPool<*mut ffi::svn_ra_session_t>) -> error::Result<R>) + Send + 'static,
        R: Send + 'static,
    {
        let context = self.inner.clone();
        let result = tokio::task::spawn_blocking(move || {
            let mut client = context.client.lock();
            let session = client.ra_session(context.key)?;
            call(session)
        })
        .await
        .context(builder::Runtime)??;

        Ok(result)
    }
}

#[derive(new)]
pub struct ContextInner {
    client: Arc<parking_lot::FairMutex<context::Context>>,
    key: i32,
}

impl Drop for ContextInner {
    fn drop(&mut self) {
        let mut client = self.client.lock();
        client.remove_ra_session(self.key);
    }
}

// #[uniffi::export(async_runtime = "tokio")]
#[uniffi::export(name = "RepositoryAccessAsyncContext", async_runtime = "tokio")]
impl AsyncContext {
    pub async fn get_latest_revision_number(&self) -> error::Result<u32> {
        self.call_async(|session| unsafe {
            let mut number: ffi::svn_revnum_t = 0;
            let mut pool = Pool::create();
            let error = ffi::svn_ra_get_latest_revnum(
                session.value,
                number.pointer_mut(),
                pool.as_mut_ptr(),
            );

            SVNError::from_nullable_ptr(error).context(builder::Svn)?;

            Ok(number.try_into().unwrap())
        })
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_locations(&self, path: String, revision: u32, location_revisions: Vec<u32>) -> error::Result<HashMap<u32, String>> {
        self.call_async(move |session| unsafe {
            let mut pool = Pool::create();
            let path = pool.string(path)?;
            let mut locations: *mut ffi::apr_hash_t = std::ptr::null_mut();

            let len = location_revisions.len();
            let location_revisions = location_revisions.into_iter().map(|rev| rev as ffi::svn_revnum_t);

            tracing::info!("CRASH DEBUG");
            let location_revisions = pool.value_array(len, location_revisions)?;

            tracing::info!("CRASH DEBUG");

            let error = ffi::svn_ra_get_locations(
                session.value,
                locations.pointer_mut(),
                path,
                revision.try_into().unwrap(),
                location_revisions,
                pool.as_mut_ptr(),
            );
            tracing::info!("CRASH DEBUG");

            tracing::info!("CRASH DEBUG");
            SVNError::from_nullable_ptr(error).context(builder::Svn)?;
            tracing::info!("CRASH DEBUG");

            let locations = pool.as_mut_ptr().hash_map(locations, |(k, v)| {
                (
                    u32::try_from(*(k as *const ffi::svn_revnum_t)).unwrap(),
                    (v as *const std::ffi::c_char)
                        .to_str()
                        .to_string(),
                )
            });
            tracing::info!("CRASH DEBUG");

            Ok(locations)
        }).await
    }
}
