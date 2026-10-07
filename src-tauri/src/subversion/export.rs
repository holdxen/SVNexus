use std::sync::Arc;

use snafu::ResultExt;

use crate::{
    error::{self, builder},
    extensions::CommonExtension,
    subversion::{
        context::{
            cat::{CatOptions, CatResult},
            difference::{ClientDifferenceOptions, ClientDifferenceResult},
            log::{LogEntry, LogOptions, LogResult},
            merge::MergeOptions,
            revert::RevertOptions,
            switch::SwitchOptions,
        },
        ra,
        version::Version,
        wc,
    },
};

use super::context::*;

#[derive(Debug, Clone)]
pub struct AsyncContext {
    context: Arc<parking_lot::FairMutex<Context>>,
}

impl AsyncContext {
    pub async fn call_async<F, R>(&self, call: F) -> error::Result<R>
    where
        F: (FnOnce(parking_lot::FairMutexGuard<'_, Context>) -> error::Result<R>) + Send + 'static,
        R: Send + 'static,
    {
        let context = self.context.clone();
        let result = tokio::task::spawn_blocking(move || {
            let context = context.lock();
            call(context)
        })
        .await
        .context(builder::Runtime)??;

        Ok(result)
    }
}

impl AsyncContext {
    pub fn working_copy_context(&self) -> wc::AsyncWorkingCopyContext {
        wc::AsyncWorkingCopyContext::Context(self.context.clone())
    }

    pub async fn property_get(&self, opts: PropertyGetOptions) -> error::Result<PropertyGetResult> {
        self.call_async(|mut context| context.property_get(opts))
            .await
    }

    pub async fn open_repository_access_session(
        &self,
        url: String,
        path: Option<String>,
    ) -> error::Result<ra::AsyncContext> {
        let client = self.context.clone();
        self.call_async(|mut context| {
            let (pool, session) = context.open_repository_access_session(url, path)?;
            let c = ra::AsyncContext::new(ra::ContextInner::new(client, pool, session).into_arc());
            Ok(c)
        })
        .await
    }

    // pub async fn get_repository_root(
    //     &self,
    //     target: String,
    // ) -> error::Result<GetRepositoryRootResult> {
    //     self.call_async(|mut context| context.get_repository_root(target))
    //         .await
    // }

    #[tracing::instrument(skip(self), ret, err)]
    pub async fn lock(&self, opts: LockOptions) -> error::Result<()> {
        self.call_async(|mut context| context.lock(opts)).await
    }

    #[tracing::instrument(skip(self), ret, err)]
    pub async fn unlock(&self, opts: UnlockOptions) -> error::Result<()> {
        self.call_async(|mut context| context.unlock(opts)).await
    }

    #[tracing::instrument(skip(self))]
    pub async fn patch(&self, opts: PatchOptions) -> error::Result<()> {
        self.call_async(|mut context| context.patch(opts)).await
    }

    #[tracing::instrument(skip(self))]
    pub async fn difference(
        &self,
        opts: ClientDifferenceOptions,
    ) -> error::Result<ClientDifferenceResult> {
        self.call_async(|mut context| context.difference(opts))
            .await
    }

    pub async fn merge(&self, opts: MergeOptions) -> error::Result<()> {
        self.call_async(|mut context| context.merge(opts)).await
    }

    pub async fn switch(&self, opts: SwitchOptions) -> error::Result<RevisionNumber> {
        self.call_async(|mut context| context.switch(opts)).await
    }

    pub async fn relocate(&self, opts: RelocateOptions) -> error::Result<()> {
        self.call_async(|mut context| context.relocate(opts)).await
    }

    pub async fn mkdir(&self, opts: MkdirOptions) -> error::Result<MkdirResult> {
        self.call_async(|mut context| context.mkdir(opts)).await
    }

    pub async fn get_wc_root(&self, path: String) -> error::Result<String> {
        self.call_async(|mut context| context.get_wc_root(path))
            .await
    }

    pub async fn revision_property_list(
        &self,
        opts: RevisionPropertyListOptions,
    ) -> error::Result<RevisionPropertyListResult> {
        self.call_async(|mut context| context.revision_property_list(opts))
            .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn property_list(
        &self,
        opts: PropertyListOptions,
    ) -> error::Result<PropertyListResult> {
        self.call_async(|mut context| context.property_list(opts))
            .await
    }

    pub async fn default_wc_version(&self) -> error::Result<Version> {
        self.call_async(|mut context| context.default_wc_version())
            .await
    }

    pub async fn url_from_path(&self, path: String) -> error::Result<String> {
        self.call_async(|mut context| context.url_from_path(path))
            .await
    }

    pub async fn cleanup(&self, opts: CleanupOptions) -> error::Result<()> {
        self.call_async(|mut context| context.cleanup(opts)).await
    }

    #[tracing::instrument(skip(self))]
    pub async fn list(&self, opts: ListOptions) -> error::Result<ListResult> {
        self.call_async(|mut context| context.list(opts)).await
    }

    pub async fn info(&self, opts: InfoOptions) -> error::Result<InfoResult> {
        self.call_async(|mut context| context.info(opts)).await
    }

    pub async fn checkout(&self, opts: CheckoutOptions) -> error::Result<RevisionNumber> {
        self.call_async(|mut context| context.checkout(opts)).await
    }

    // pub async fn status_next(
    //     &self,
    //     opts: StatusOptions,
    //     receiver: Arc<dyn StatusReceiver>,
    // ) -> error::Result<()> {
    //     self.call_async(|mut context| context.status_next(opts, receiver))
    //         .await
    // }

    pub async fn status(&self, opts: StatusOptions) -> error::Result<StatusResult> {
        self.call_async(|mut context| context.status(opts)).await
    }

    pub async fn add(&self, opts: AddOptions) -> error::Result<()> {
        self.call_async(|mut context| context.add(opts)).await
    }

    pub async fn commit(&self, opts: CommitOptions) -> error::Result<CommitResult> {
        self.call_async(|mut context| context.commit(opts)).await
    }

    pub async fn cat(&self, opts: CatOptions) -> error::Result<CatResult> {
        self.call_async(|mut context| context.cat(opts)).await
    }

    pub async fn delete(&self, opts: DeleteOptions) -> error::Result<DeleteResult> {
        self.call_async(|mut context| context.delete(opts)).await
    }

    pub async fn revert(&self, opts: RevertOptions) -> error::Result<()> {
        self.call_async(|mut context| context.revert(opts)).await
    }

    pub async fn log(&self, opts: LogOptions) -> error::Result<LogResult> {
        self.call_async(|mut context| context.log(opts)).await
    }

    pub async fn log_next(
        &self,
        opts: LogOptions,
        receiver: impl (Fn(LogEntry) -> Result<(), error::FrontendError>) + Send + Sync + 'static,
    ) -> error::Result<()> {
        self.call_async(|mut context| context.log_next(opts, receiver))
            .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn import(
        &self,
        opts: ImportOptions,
        filters: Option<Vec<String>>,
    ) -> error::Result<ImportResult> {
        self.call_async(move |mut context| context.import(opts, filters))
            .await
    }

    pub async fn export(&self, opts: ExportOptions) -> error::Result<Option<RevisionNumber>> {
        self.call_async(|mut context| context.export(opts)).await
    }

    pub async fn update(&self, opts: UpdateOptions) -> error::Result<Vec<Option<RevisionNumber>>> {
        self.call_async(|mut context| context.update(opts)).await
    }

    // #[tracing::instrument(skip(self, notifier), err)]
    // pub async fn initialize_repository(
    //     &self,
    //     opts: InitializeRepositoryOptions,
    //     notifier: Arc<dyn InitializeRepositoryNotifier>,
    // ) -> error::Result<()> {
    //     self.call_async(|mut context| context.initialize_repository(opts, notifier))
    //         .await
    // }

    pub async fn conflict_walk(
        &self,
        opts: ConflictWalkOptions,
    ) -> error::Result<ConflictWalkResult> {
        self.call_async(|mut context| context.conflict_walk(opts))
            .await
    }

    pub async fn property_set(&self, opts: PropertySetOptions) -> error::Result<()> {
        self.call_async(|mut context| context.property_set(opts))
            .await
    }

    pub fn create(opts: CreateContextOptions) -> error::Result<Self> {
        tracing::info!("Creating AsyncContext with options");
        let context = ContextFactory::instance()?.create_context(opts)?;
        let context = Arc::new(parking_lot::FairMutex::new(context));
        Ok(AsyncContext { context })
    }
}
