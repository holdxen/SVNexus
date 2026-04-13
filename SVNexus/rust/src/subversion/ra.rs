use std::sync::Arc;

use derive_new::new;
use snafu::ResultExt;

use super::context;
use super::ffi;
use crate::apr::AutoPool;
use crate::apr::Pool;
use crate::error;
use crate::error::builder;
use crate::subversion::SVNError;
use crate::utils::Pointer;


#[derive(uniffi::Object, new)]
#[uniffi(name = "RepositoryAccessAsyncContext")]
pub struct AsyncContext {
    inner: Arc<ContextInner>
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
        }).await.context(builder::Runtime)??;

        Ok(result)
    }
}


#[derive(new)]
pub struct ContextInner {
    client: Arc<parking_lot::FairMutex<context::Context>>,
    key: i32
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
    async fn get_latest_revision_number(&self) -> error::Result<u32> {
        self.call_async(|session| {
            unsafe {
                let mut number: ffi::svn_revnum_t = 0;
                let mut pool = Pool::create();
                let error = ffi::svn_ra_get_latest_revnum(session.value, number.pointer_mut(), pool.as_mut_ptr());


                SVNError::from_nullable_ptr(error).context(builder::Svn)?;

                Ok(number.try_into().unwrap())
            }
        }).await
    }
}
