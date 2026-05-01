use std::{sync::Arc, vec};

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

#[derive(uniffi::Object, Debug)]
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

    pub async fn build_repository_logs(
        &self,
        db: Arc<db::SeaDatabaseConnection>,
        uuid: String,
        url: String,
    ) -> error::Result<()> {
        db.truncate_repository_logs(&uuid).await?;

        let (tx, mut rx) = mpsc::unbounded_channel();

        let receiver = LogReceiverClosure::new(move |entry| {
            let result = tx.send(entry);
            if let Err(err) = result {
                tracing::error!("Failed to send log entry: {}", err);
            }
            Ok(())
        });

        let joiner = tokio::spawn(async move {
            let limit = 1024;

            let mut entries = Vec::with_capacity(limit);
            loop {
                let next = rx.recv().await;
                if let Some(entry) = next {
                    entries.insert(0, entry);

                    if entries.len() >= limit {
                        db.insert_repository_logs(true, uuid.clone(), std::mem::take(&mut entries))
                            .await?;
                    }
                } else {
                    break;
                }
            }
            if !entries.is_empty() {
                // tracing::info!("Inserting remaining {:#?} entries into database", entries);
                db.insert_repository_logs(true, uuid.clone(), std::mem::take(&mut entries))
                    .await?;
            }

            error::ok(())
        });

        let log_options = LogOptions {
            targets: vec![url.clone()],
            peg_revision: Revision::Head,
            limit: 0,
            revisions: vec![RevisionRange::new(Revision::Head, Revision::Number(0))],
            discover_changed_paths: true,
            strict_node_history: false,
            include_merged_revisions: true,
            revisions_properties: None,
        };

        self.log_next(log_options, receiver.into_arc()).await?;

        joiner
            .await
            .with_any_context(|e| format!("Unexpected task error: {}", e))?
    }

    #[tracing::instrument(skip(self, db))]
    pub async fn log_cache_fill_local(
        &self,
        db: Arc<db::SeaDatabaseConnection>,
        uuid: &str,
        path: &str,
        start: Option<u32>,
        limit: Option<u32>,
    ) -> error::Result<Vec<IndexedLogEntry>> {

        let start = if let Some(start) = start {
            Revision::Number(start)
        } else {
            Revision::Head
        };

        let log_options = LogOptions {
            targets: vec![path.to_string()],
            peg_revision: Revision::Working,
            limit: limit.unwrap_or_default(),
            revisions: vec![RevisionRange::new(start, Revision::Number(1))],
            discover_changed_paths: false,
            strict_node_history: false,
            include_merged_revisions: false,
            revisions_properties: vec![].into_option_some(),
        };

        let log_result = self.log(log_options).await?;

        let mut revisions: Vec<_> = log_result
            .log_entries
            .iter()
            .map(|v| v.revision)
            .collect();

        let mut logs = db
            .repository_logs_with_revisions(uuid, revisions.iter().filter_map(|v| *v).collect())
            .await?;

        for i in logs.iter() {
            if let Some(index) = revisions.iter().position(|v| *v == i.entry.revision) {
                revisions[index] = None;
            }
        }

        revisions.push(None);

        let mut tmp = vec![];

        for i in revisions {
            let start;
            let end;
            if let Some(revision) = i {
                tmp.push(revision);
                continue;
            } else {
                if tmp.is_empty() {
                    continue;
                }
                start = *tmp.first().unwrap();
                end = *tmp.last().unwrap();
                tmp.clear();
            }

            tracing::info!("Log from repository: start={}, end={}", start, end);
            let log_options = LogOptions {
                targets: vec![path.to_string()],
                peg_revision: Revision::Working,
                limit: 0,
                revisions: vec![RevisionRange::new(
                    Revision::Number(start),
                    Revision::Number(end),
                )],
                discover_changed_paths: true,
                strict_node_history: false,
                include_merged_revisions: false,
                revisions_properties: None,
            };

            let log_result = self.log(log_options).await?;

            let extra = db
                .insert_repository_logs(true, uuid.to_string(), log_result.log_entries)
                .await?;

            logs.extend(extra);
        }

        // if !revisions.is_empty() {
        //     let ranges = revisions
        //         .into_iter()
        //         .map(|v| RevisionRange::new(Revision::Number(v), Revision::Number(v)))
        //         .collect();

        //     let log_options = LogOptions {
        //         targets: vec![path.to_string()],
        //         peg_revision: Revision::Working,
        //         limit: limit.unwrap_or_default(),
        //         revisions: ranges,
        //         discover_changed_paths: true,
        //         strict_node_history: false,
        //         include_merged_revisions: false,
        //         revisions_properties: None,
        //     };

        //     let extra = self.log(log_options).await?;

        //     let extra = db
        //         .insert_repository_logs(true, uuid.to_string(), extra.log_entries)
        //         .await?;

        //     logs.extend(extra);
        // }

        logs.sort_by(|v1, v2| v2.entry.revision.cmp(&v1.entry.revision));

        Ok(logs)
    }

    pub async fn log_cache_fill(
        &self,
        db: Arc<db::SeaDatabaseConnection>,
        uuid: &str,
        url: &str,
        start: Option<u32>,
        limit: Option<u32>,
    ) -> error::Result<Vec<IndexedLogEntry>> {
        if limit.is_some_and(|v| v == 0) {
            return Ok(Default::default());
        }

        tracing::info!("log cache fill: uuid={}, url={}", uuid, url);

        let mut logs = db.repository_logs(uuid, start, limit, true).await?;

        tracing::info!("Cache from database: {}", logs.len());
        tracing::info!(
            "Cache from database: first={:#?}, last={:#?}",
            logs.first(),
            logs.last()
        );

        let start = if let Some(start) = start {
            start
        } else {
            let ra = self
                .open_repository_access_session(url.to_string(), None)
                .await?;
            ra.get_latest_revision_number().await?
        };

        // 10 9 8 7 6 5 4 3 2 1 0
        tracing::info!("start={}", start);

        let end = if let Some(limit) = limit {
            start.saturating_sub(limit)
        } else {
            0
        };

        tracing::info!("end={}", end);

        let mut targets: Vec<_> = (end + 1..=start)
            .rev()
            .map(|v| v.into_option_some())
            .collect();

        // tracing::info!("initial targets: {:?}", targets);

        for i in &logs {
            // targets.retain(|v| *v == i.entry.revision);
            if let Some(index) = targets.iter().position(|v| *v == i.entry.revision) {
                targets[index] = None;
            }
        }

        let targets: Vec<Vec<_>> = targets
            .split(|x| x.is_none())
            .filter(|chunk| !chunk.is_empty())
            .map(|chunk| {
                chunk
                    .iter()
                    .filter_map(|&x| x) // Some(i32) -> i32，None 会被过滤
                    .collect()
            })
            .collect();

        // tracing::info!("left targets: {:?}", targets);

        for i in targets {
            let start = *i.first().unwrap();
            let end = *i.last().unwrap();
            tracing::info!("Log from repository: start={}, end={}", start, end);
            let log_options = LogOptions {
                targets: vec![url.to_string()],
                peg_revision: Revision::Head,
                limit: 0,
                revisions: vec![RevisionRange::new(
                    Revision::Number(start),
                    Revision::Number(end),
                )],
                discover_changed_paths: true,
                strict_node_history: false,
                include_merged_revisions: false,
                revisions_properties: None,
            };

            let log_result = self.log(log_options).await?;

            let inserted = db
                .insert_repository_logs(true, uuid.to_string(), log_result.log_entries)
                .await?;

            logs.extend(inserted);
        }

        logs.sort_by(|v1, v2| v2.entry.revision.cmp(&v1.entry.revision));

        Ok(logs)
    }

    pub async fn log_cache(
        &self,
        db: Arc<db::SeaDatabaseConnection>,
        uuid: &str,
        url: &str,
        start: Option<u32>,
        limit: Option<u32>,
    ) -> error::Result<Vec<IndexedLogEntry>> {
        if limit.is_some_and(|v| v == 0) {
            return Ok(Default::default());
        }

        let mut logs = db.repository_logs(uuid, start, limit, true).await?;

        let mut try_again = false;
        if logs.is_empty() {
            try_again = true;
        } else if let Some(limit) = limit {
            if (logs.len() as u32) < limit
                && logs.last().unwrap().entry.revision != 0.into_option_some()
            {
                try_again = true;
            }
        }

        if try_again {
            let log_options = LogOptions {
                targets: vec![url.to_string()],
                peg_revision: Revision::Head,
                limit: limit.unwrap_or(0),
                revisions: vec![RevisionRange::new(
                    if let Some(start) = start {
                        Revision::Number(start)
                    } else {
                        Revision::Head
                    },
                    Revision::Number(0),
                )],
                discover_changed_paths: true,
                strict_node_history: false,
                include_merged_revisions: false,
                revisions_properties: None,
            };

            let log_result = self.log(log_options).await?;

            db.delete_repository_logs(uuid, logs.iter().map(|e| e.id).collect())
                .await?;

            logs = db
                .insert_repository_logs(true, uuid.to_string(), log_result.log_entries)
                .await?;
        }
        Ok(logs)
    }

    // pub async fn update_and_repository_logs_from_database(
    //     &self,
    //     db: Arc<db::SeaDatabaseConnection>,
    //     uuid: String,
    //     start: Option<i64>,
    //     limit: Option<u64>,
    // ) -> error::Result<Vec<IndexedLogEntry>> {
    //     let mut logs = db.repository_logs(&uuid, start, limit, false).await?;

    //     if logs.is_empty() {
    //         let log_options = LogOptions {
    //             targets: vec![],
    //             peg_revision: Revision::Head,
    //             limit: limit.unwrap_or(0).try_into().unwrap_or(0),
    //             revisions: vec![RevisionRange::new(Revision::Head, Revision::Number(0))],
    //             discover_changed_paths: true,
    //             strict_node_history: false,
    //             include_merged_revisions: false,
    //             revisions_properties: None,
    //         };

    //         let log_result = self.log(log_options).await?;

    //         logs = db
    //             .insert_repository_logs(true, uuid, log_result.log_entries)
    //             .await?;
    //     } else {
    //     }

    //     todo!()
    // }
    // pub async fn load_repository_logs_from_database(
    //     &self,
    //     db: Arc<db::SeaDatabaseConnection>,
    //     uuid: String,
    //     start: Option<u32>,
    //     limit: Option<u64>,
    // ) -> error::Result<Vec<IndexedLogEntry>> {
    //     let mut logs = db.repository_logs(&uuid, start, limit, false).await?;

    //     if logs.is_empty() {
    //         let log_options = LogOptions {
    //             targets: vec![],
    //             peg_revision: Revision::Head,
    //             limit: limit.unwrap_or(0).try_into().unwrap_or(0),
    //             revisions: vec![RevisionRange::new(Revision::Head, Revision::Number(0))],
    //             discover_changed_paths: true,
    //             strict_node_history: false,
    //             include_merged_revisions: false,
    //             revisions_properties: None,
    //         };

    //         let log_result = self.log(log_options).await?;

    //         logs = db
    //             .insert_repository_logs(true, uuid, log_result.log_entries)
    //             .await?;
    //     } else {
    //         let revision = logs.last().unwrap().entry.revision.unwrap_or_default();
    //         if (logs.len() as u64) < limit.unwrap_or(0) && revision != 0 {
    //             let log_options = LogOptions {
    //                 targets: vec![],
    //                 peg_revision: Revision::Head,
    //                 limit: (if let Some(limit) = limit {
    //                     limit - logs.len() as u64
    //                 } else {
    //                     0
    //                 })
    //                 .try_into()
    //                 .unwrap(),
    //                 revisions: vec![RevisionRange::new(
    //                     Revision::Number(revision - 1),
    //                     Revision::Number(0),
    //                 )],
    //                 discover_changed_paths: true,
    //                 strict_node_history: false,
    //                 include_merged_revisions: false,
    //                 revisions_properties: None,
    //             };

    //             let log_result = self.log(log_options).await?;

    //             logs.extend(
    //                 db.insert_repository_logs(true, uuid, log_result.log_entries)
    //                     .await?,
    //             );
    //         }
    //     }

    //     Ok(logs)
    // }

    // pub async fn load_repository_logs(
    //     &self,
    //     db: Arc<db::SeaDatabaseConnection>,
    //     start: Option<i64>,
    //     limit: Option<u64>,
    //     uuid: String,
    //     url: String,
    // ) -> error::Result<Vec<IndexedLogEntry>> {
    //     tracing::info!(
    //         "Loading repository logs from database with start={:?}, limit={:?}",
    //         start,
    //         limit
    //     );
    //     let mut logs = db.repository_logs(&uuid, start, limit, false).await?;

    //     tracing::info!("Cachce from database: {}", logs.len());

    //     let mut count = 0u64;
    //     let mut parents = 0;

    //     let mut last_id = logs.last().map(|e| e.id);

    //     let mut last_revision: Option<u32> = None;

    //     let mut new = vec![];

    //     // let mut database_is_empty = logs.len() <;

    //     let mut database_is_empty = limit.is_none_or(|_| logs.is_empty());

    //     loop {
    //         if logs.is_empty() {
    //             let mut cache = vec![];
    //             if !database_is_empty {
    //                 cache = db
    //                     .repository_logs(&uuid, last_id, 100.into_option_some(), false)
    //                     .await?;
    //             }

    //             if cache.is_empty() {
    //                 database_is_empty = true;
    //                 let log_options = LogOptions {
    //                     targets: vec![url.clone()],
    //                     peg_revision: Revision::Head,
    //                     limit: 100,
    //                     revisions: vec![RevisionRange::new(
    //                         if let Some(revision) = last_revision {
    //                             Revision::Number(revision + 1)
    //                         } else {
    //                             Revision::Head
    //                         },
    //                         Revision::Number(0),
    //                     )],
    //                     discover_changed_paths: true,
    //                     strict_node_history: false,
    //                     include_merged_revisions: true,
    //                     revisions_properties: None,
    //                 };

    //                 let log_result = self.log(log_options).await?;

    //                 cache = db
    //                     .insert_repository_logs(true, uuid.clone(), log_result.log_entries)
    //                     .await?;
    //             }
    //             snafu::ensure!(
    //                 !cache.is_empty(),
    //                 builder::CacheBroken { uuid: uuid.clone() }
    //             );

    //             logs.extend(cache);
    //         }

    //         let i = logs.remove(0);

    //         if i.entry.revision.is_none() {
    //             parents -= 1;
    //         } else if i.entry.has_children {
    //             parents += 1;
    //             count += 1;
    //         } else if parents == 0 {
    //             count += 1;
    //             last_revision = i.entry.revision;
    //         }

    //         let revision = i.entry.revision;

    //         last_id = Some(i.id);
    //         new.push(i);

    //         if revision == 0.into_option_some() {
    //             break;
    //         }

    //         if parents == 0 && count.into_option_some() >= limit {
    //             break;
    //         }
    //     }

    //     Ok(new)
    // }

    // pub async fn update_repository_logs(
    //     &self,
    //     db: Arc<db::SeaDatabaseConnection>,
    //     uuid: String,
    //     url: String,
    // ) -> error::Result<Vec<IndexedLogEntry>> {
    //     let logs = db
    //         .repository_logs(&uuid, None, 1.into_option_some(), false)
    //         .await?;

    //     if let Some(first) = logs.first() {
    //         let revision = first
    //             .entry
    //             .revision
    //             .context(builder::CacheBroken { uuid: uuid.clone() })?;

    //         let log_options = LogOptions {
    //             targets: vec![url],
    //             peg_revision: Revision::Head,
    //             limit: 0,
    //             revisions: vec![RevisionRange::new(
    //                 Revision::Number(revision + 1),
    //                 Revision::Head,
    //             )],
    //             discover_changed_paths: true,
    //             strict_node_history: false,
    //             include_merged_revisions: true,
    //             revisions_properties: None,
    //         };

    //         let (tx, mut rx) = mpsc::unbounded_channel();

    //         let receiver = LogReceiverClosure::new(move |entry| {
    //             let result = tx.send(entry);
    //             if let Err(err) = result {
    //                 tracing::error!("Failed to send log entry: {}", err);
    //             }
    //             Ok(())
    //         });

    //         let joiner = tokio::spawn(async move {
    //             let limit = 100;

    //             let mut entries = Vec::with_capacity(limit);
    //             let mut indexed_entries = vec![];
    //             loop {
    //                 let next = rx.recv().await;
    //                 if let Some(entry) = next {
    //                     entries.insert(0, entry);

    //                     if entries.len() >= limit {
    //                         indexed_entries = db
    //                             .insert_repository_logs(
    //                                 false,
    //                                 uuid.clone(),
    //                                 std::mem::take(&mut entries),
    //                             )
    //                             .await?;
    //                     }
    //                 } else {
    //                     break;
    //                 }
    //             }
    //             if !entries.is_empty() {
    //                 // tracing::info!("Inserting remaining {:#?} entries into database", entries);
    //                 let mut indexed = db
    //                     .insert_repository_logs(false, uuid.clone(), std::mem::take(&mut entries))
    //                     .await?;

    //                 indexed.extend(indexed_entries);
    //                 indexed.truncate(limit);

    //                 indexed_entries = indexed;
    //             }

    //             error::ok(indexed_entries)
    //         });

    //         self.log_next(log_options, receiver.into_arc()).await?;

    //         joiner
    //             .await
    //             .with_any_context(|e| format!("Unexpected task error: {}", e))?
    //     } else {
    //         let log_options = LogOptions {
    //             targets: vec![url],
    //             peg_revision: Revision::Head,
    //             limit: 100,
    //             revisions: vec![RevisionRange::new(Revision::Head, Revision::Number(0))],
    //             discover_changed_paths: true,
    //             strict_node_history: false,
    //             include_merged_revisions: true,
    //             revisions_properties: None,
    //         };

    //         let result = self.log(log_options).await?;

    //         db.insert_repository_logs(true, uuid, result.log_entries)
    //             .await
    //     }

    // }

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

    #[tracing::instrument(skip(self), ret, err)]
    pub async fn lock(&self, opts: LockOptions) -> error::Result<()> {
        self.call_async(|mut context| context.lock(opts)).await
    }

    #[tracing::instrument(skip(self), ret, err)]
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

    pub async fn property_set(&self, opts: PropertySetOptions) -> error::Result<()> {
        self.call_async(|mut context| context.property_set(opts)).await
    }

    #[uniffi::constructor]
    pub fn create(opts: CreateContextOptions) -> error::Result<Self> {
        tracing::info!("Creating AsyncContext with options");
        let context = ContextFactory::instance()?.create_context(opts)?;
        let context = Arc::new(parking_lot::FairMutex::new(context));
        Ok(AsyncContext { context })
    }
}
