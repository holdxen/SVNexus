use std::sync::Arc;

use snafu::{OptionExt, ResultExt};
use tokio::{
    sync::{Mutex, mpsc, oneshot},
    task::JoinHandle,
};

use crate::{
    db::{self, IndexedLogEntry},
    error::{self, builder},
    extensions::{CommonExtension, ResultExtension},
    subversion::{ra, version::Version, wc},
};

use super::context::*;

#[derive(uniffi::Object)]
pub struct AsyncContext {
    context: Arc<parking_lot::FairMutex<Context>>,
}

impl AsyncContext {
    async fn call_async<F, R>(&self, call: F) -> error::Result<R>
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

pub struct LogReceiverClosure {
    closure: parking_lot::Mutex<
        Box<dyn (FnMut(LogEntry) -> Result<(), error::CSharpError>) + Send + 'static>,
    >,
}

impl LogReceiverClosure {
    fn new(
        closure: impl FnMut(LogEntry) -> Result<(), error::CSharpError> + Send + 'static,
    ) -> Self {
        Self {
            closure: parking_lot::Mutex::new(Box::new(closure)),
        }
    }
}

impl LogReceiver for LogReceiverClosure {
    fn on_log_entry(&self, log_entry: LogEntry) -> Result<(), error::CSharpError> {
        (self.closure.lock())(log_entry)?;
        Ok(())
    }
}

// pub struct LogReceiverImpl {
//     pub sender: mpsc::UnboundedSender<LogEntry>,
//     pub joiner: Mutex<Option<JoinHandle<error::Result<()>>>>,
// }

// impl LogReceiverImpl {
//     async fn join(&self) -> error::Result<()> {
//         if let Some(joiner) = self.joiner.lock().await.take() {
//             joiner
//                 .await
//                 .with_any_context(|e| format!("Unexpect task error: {}", e))??;
//         }
//         Ok(())
//     }

//     fn new(db: Arc<db::SeaDatabaseConnection>, tail: bool, uuid: String) -> Self {
//         let (sender, mut receiver) = mpsc::unbounded_channel::<LogEntry>();

//         let t = tokio::spawn(async move {
//             let mut entries = Vec::with_capacity(100);

//             loop {
//                 let next = receiver.recv().await;
//                 if let Some(entry) = next {
//                     entries.insert(0, entry);

//                     if entries.len() >= 100 {
//                         db.insert_repository_logs(tail, uuid.clone(), std::mem::take(&mut entries))
//                             .await?;
//                     }
//                 } else {
//                     break;
//                 }
//             }
//             if !entries.is_empty() {
//                 db.insert_repository_logs(tail, uuid.clone(), std::mem::take(&mut entries))
//                     .await?;
//             }

//             error::ok(())
//         });

//         Self {
//             sender,
//             joiner: Mutex::new(t.into_option_some()),
//         }
//     }
// }

// impl LogReceiver for LogReceiverImpl {
//     fn on_log_entry(&self, log_entry: LogEntry) -> Result<(), error::CSharpError> {
//         // self.sender.send(log_entry).with_any_context(|_| "Failed to send log entry")?;
//         let result = self.sender.send(log_entry);
//         if let Err(err) = result {
//             tracing::error!("Failed to send log entry: {}", err);
//         }

//         Ok(())
//     }
// }

#[uniffi::export(async_runtime = "tokio")]
impl AsyncContext {
    pub fn working_copy_context(&self) -> wc::AsyncWorkingCopyContext {
        wc::AsyncWorkingCopyContext::Context(self.context.clone())
    }

    pub async fn load_repository_logs(
        &self,
        db: Arc<db::SeaDatabaseConnection>,
        start: Option<i64>,
        limit: Option<u64>,
        uuid: String,
        url: String,
    ) -> error::Result<Vec<IndexedLogEntry>> {
        let mut logs = db.repository_logs(&uuid, start, limit, false).await?;

        // if logs.is_empty() {
        //     let log_options = LogOptions {
        //         targets: vec![url],
        //         peg_revision: Revision::Head,
        //         limit: limit.unwrap_or_default() as u32,
        //         revsions: vec![RevisionRange::new(Revision::Head, Revision::Number(0))],
        //         discover_changed_paths: true,
        //         strict_node_history: false,
        //         include_merged_revisions: true,
        //         revisions_properties: None,
        //     };

        //     let result = self.log(log_options).await?;

        //     return db
        //         .insert_repository_logs(true, uuid, result.log_entries)
        //         .await;
        // }

        let mut count = 0u64;
        let mut parents = 0;

        let mut last_id = logs.last().map(|e| e.id);

        let mut last_revision: Option<u32> = None;

        let mut new = vec![];

        let mut database_is_empty = logs.is_empty();

        loop {
            if logs.is_empty() {
                let mut cache = vec![];
                if !database_is_empty {
                    cache = db
                        .repository_logs(&uuid, last_id, 1.into_option_some(), false)
                        .await?;
                }

                if cache.is_empty() && !database_is_empty {
                    database_is_empty = true;
                    let log_options = LogOptions {
                        targets: vec![url.clone()],
                        peg_revision: Revision::Head,
                        limit: 100,
                        revsions: vec![RevisionRange::new(
                            Revision::Number(last_revision.unwrap()), // cache broken
                            Revision::Number(0),
                        )],
                        discover_changed_paths: true,
                        strict_node_history: false,
                        include_merged_revisions: true,
                        revisions_properties: None,
                    };

                    let log_result = self.log(log_options).await?;

                    cache = db
                        .insert_repository_logs(true, uuid.clone(), log_result.log_entries)
                        .await?;

                    if !cache.is_empty() && cache.first().unwrap().entry.revision == last_revision {
                        cache.remove(0);
                    }
                }
                snafu::ensure!(
                    !cache.is_empty(),
                    builder::CacheBroken { uuid: uuid.clone() }
                );

                logs.extend(cache);
            }

            let i = logs.remove(0);

            if i.entry.revision.is_none() {
                parents -= 1;
            } else if i.entry.has_children {
                parents += 1;
                count += 1;
            } else if parents == 0 {
                count += 1;
                last_revision = i.entry.revision;
            }

            let revision = i.entry.revision;

            last_id = Some(i.id);
            new.push(i);

            if revision == 0.into_option_some() {
                break;
            }

            if parents == 0 && count.into_option_some() >= limit {
                break;
            }
        }

        // let mut last_id = logs.last().unwrap().id;

        // let mut iter = logs.into_iter();

        // let mut last_revsion = 0;

        // let mut left = vec![];

        // loop {
        //     let next = iter.next();

        //     let i = if let Some(entry) = next {
        //         entry
        //     } else {
        //         if parents == 0 && count.into_option_some() >= limit {
        //             break;
        //         }
        //         let mut cache = db
        //             .repository_logs(
        //                 &uuid,
        //                 last_id.into_option_some(),
        //                 1.into_option_some(),
        //                 false,
        //             )
        //             .await?;

        //         if cache.is_empty() {
        //             let log_options = LogOptions {
        //                 targets: vec![url.clone()],
        //                 peg_revision: Revision::Head,
        //                 limit: 100,
        //                 revsions: vec![RevisionRange::new(
        //                     Revision::Number(last_revsion),
        //                     Revision::Number(0),
        //                 )],
        //                 discover_changed_paths: true,
        //                 strict_node_history: false,
        //                 include_merged_revisions: true,
        //                 revisions_properties: None,
        //             };

        //             let log_result = self.log(log_options).await?;

        //             db.insert_repository_logs(true, uuid.clone(), log_result.log_entries)
        //                 .await?;

        //             continue;
        //         }

        //         last_id = cache.last().unwrap().id;
        //         cache.remove(0)
        //     };

        //     if i.entry.revision.is_none() {
        //         parents -= 1;
        //     } else if i.entry.has_children {
        //         parents += 1;
        //         count += 1;
        //     } else if parents == 0 {
        //         count += 1;
        //     }
        //     let revision = i.entry.revision;

        //     last_revsion = revision.unwrap_or_default();

        //     left.push(i);

        //     if revision == 0.into_option_some() {
        //         break;
        //     }

        // }

        // Ok(left)
        //

        Ok(new)

        // for i in logs.iter() {
        //     if i.entry.revision.is_none() {
        //         parents -= 1;
        //     } else if i.entry.has_children {
        //         parents += 1;
        //     } else if parents == 0 {
        //         count += 1;
        //     }
        // }
        // if parents != 0 {
        //     let last = logs.last().unwrap().id;
        //     let logs = db.repository_logs(&uuid, last.into_option_some(), 1.into_option_some(), false).await?;

        //     snafu::ensure!(logs.len() == 1, builder::CacheBroken {
        //         uuid: uuid.clone()
        //     });

        // }

        // todo!()
    }

    pub async fn update_repository_logs(
        &self,
        db: Arc<db::SeaDatabaseConnection>,
        uuid: String,
        url: String,
    ) -> error::Result<Vec<IndexedLogEntry>> {
        let logs = db
            .repository_logs(&uuid, None, 1.into_option_some(), false)
            .await?;

        if let Some(first) = logs.first() {
            let revision = first
                .entry
                .revision
                .context(builder::CacheBroken { uuid: uuid.clone() })?;

            // let log_options = LogOptions {
            //     targets: vec![url.clone()],
            //     peg_revision: Revision::Head,
            //     limit: 0,
            //     revsions: vec![RevisionRange::new(
            //         Revision::Head,
            //         Revision::Number(revision),
            //     )],
            //     discover_changed_paths: true,
            //     strict_node_history: false,
            //     include_merged_revisions: true,
            //     revisions_properties: None,
            // };

            // let log_result = self.log(log_options).await?;

            // let rev = log_result.log_entries.last().map(|e| e.revision).flatten().with_any_context(|| "Unexpected log received")?;

            // let entries = db.insert_repository_logs(false, uuid.clone(), log_result.log_entries).await?;

            let log_options = LogOptions {
                targets: vec![url],
                peg_revision: Revision::Head,
                limit: 0,
                revsions: vec![RevisionRange::new(
                    Revision::Number(revision),
                    Revision::Head,
                )],
                discover_changed_paths: true,
                strict_node_history: false,
                include_merged_revisions: true,
                revisions_properties: None,
            };

            let (tx, mut rx) = mpsc::unbounded_channel();

            let receiver = LogReceiverClosure::new(move |entry| {
                let result = tx.send(entry);
                if let Err(err) = result {
                    tracing::error!("Failed to send log entry: {}", err);
                }
                Ok(())
            });

            let joiner = tokio::spawn(async move {
                let limit = 100;

                let mut entries = Vec::with_capacity(limit);
                let mut indexed_entries = vec![];

                let mut first = true;

                loop {
                    let next = rx.recv().await;
                    if let Some(entry) = next {
                        if first && entry.revision == revision.into_option_some() {
                            first = false;
                            continue;
                        }
                        entries.insert(0, entry);

                        if entries.len() >= limit {
                            indexed_entries = db
                                .insert_repository_logs(
                                    false,
                                    uuid.clone(),
                                    std::mem::take(&mut entries),
                                )
                                .await?;
                        }
                    } else {
                        break;
                    }
                }
                if !entries.is_empty() {
                    let mut indexed = db
                        .insert_repository_logs(false, uuid.clone(), std::mem::take(&mut entries))
                        .await?;

                    indexed.extend(indexed_entries);
                    indexed.truncate(limit);

                    indexed_entries = indexed;
                }

                error::ok(indexed_entries)
            });

            self.log_next(log_options, receiver.into_arc()).await?;

            // let receiver = LogReceiverImpl::new(db, false, uuid.clone()).into_arc();

            // receiver.join().await?;
            //
            joiner
                .await
                .with_any_context(|e| format!("Unexpected task error: {}", e))?
        } else {
            let log_options = LogOptions {
                targets: vec![url],
                peg_revision: Revision::Head,
                limit: 100,
                revsions: vec![RevisionRange::new(Revision::Head, Revision::Number(0))],
                discover_changed_paths: true,
                strict_node_history: false,
                include_merged_revisions: true,
                revisions_properties: None,
            };

            let result = self.log(log_options).await?;

            db.insert_repository_logs(true, uuid, result.log_entries)
                .await
        }

        // let revision = logs
        //     .first()
        //     .map(|e| e.revision.unwrap_or_default())
        //     .unwrap_or_default();

        // self.call_async(|context| {
        //     todo!()
        // }).await;
    }

    pub async fn open_repository_access_session(
        &self,
        url: String,
        path: Option<String>,
    ) -> error::Result<ra::AsyncContext> {
        let client = self.context.clone();
        self.call_async(|mut context| {
            let k = context.open_repository_access_session(url, path)?;
            let c = ra::AsyncContext::new(ra::ContextInner::new(client, k).into_arc());
            Ok(c)
        })
        .await
    }

    pub async fn get_repository_root(
        &self,
        target: String,
    ) -> error::Result<GetRepositoryRootResult> {
        self.call_async(|mut context| context.get_repository_root(target))
            .await
    }

    pub async fn lock(&self, opts: LockOptions) -> error::Result<()> {
        self.call_async(|mut context| context.lock(opts)).await
    }

    pub async fn unlock(&self, opts: UnlockOptions) -> error::Result<()> {
        self.call_async(|mut context| context.unlock(opts)).await
    }

    pub async fn patch(&self, opts: PatchOptions) -> error::Result<()> {
        self.call_async(|mut context| context.patch(opts)).await
    }

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

    pub async fn list(&self, opts: ListOptions) -> error::Result<ListResult> {
        self.call_async(|mut context| context.list(opts)).await
    }

    pub async fn info(&self, opts: InfoOptions) -> error::Result<InfoResult> {
        self.call_async(|mut context| context.info(opts)).await
    }

    pub async fn checkout(&self, opts: CheckoutOptions) -> error::Result<RevisionNumber> {
        self.call_async(|mut context| context.checkout(opts)).await
    }

    pub async fn status_next(
        &self,
        opts: StatusOptions,
        receiver: Arc<dyn StatusReceiver>,
    ) -> error::Result<()> {
        self.call_async(|mut context| context.status_next(opts, receiver))
            .await
    }

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
        receiver: Arc<dyn LogReceiver>,
    ) -> error::Result<()> {
        self.call_async(|mut context| context.log_next(opts, receiver))
            .await
    }

    pub async fn import(&self, opts: ImportOptions) -> error::Result<ImportResult> {
        self.call_async(|mut context| context.import(opts)).await
    }

    pub async fn export(&self, opts: ExportOptions) -> error::Result<RevisionNumber> {
        self.call_async(|mut context| context.export(opts)).await
    }

    pub async fn update(&self, opts: UpdateOptions) -> error::Result<Vec<Option<RevisionNumber>>> {
        self.call_async(|mut context| context.update(opts)).await
    }

    pub async fn initialize_repository(
        &self,
        opts: InitializeRepositoryOptions,
        notifier: Arc<dyn InitializeRepositoryNotifier>,
    ) -> error::Result<()> {
        self.call_async(|mut context| context.initialize_repository(opts, notifier))
            .await
    }

    pub async fn conflict_walk(
        &self,
        opts: ConflictWalkOptions,
    ) -> error::Result<ConflictWalkResult> {
        self.call_async(|mut context| context.conflict_walk(opts))
            .await
    }

    #[uniffi::constructor]
    pub fn create(opts: CreateContextOptions) -> error::Result<Self> {
        tracing::info!("Before creating context");
        let context = ContextFactory::instance()?.create_context(opts)?;
        let context = Arc::new(parking_lot::FairMutex::new(context));
        tracing::info!("Finished creating context");
        Ok(AsyncContext { context })
    }
}
