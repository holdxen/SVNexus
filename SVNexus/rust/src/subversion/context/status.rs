use super::*;

#[derive(Debug, uniffi::Record)]
pub struct StatusOptions {
    path: String,
    revision: Revision,
    depth: Depth,
    get_all: bool,
    check_out_of_date: bool,
    check_working_copy: bool,
    no_ignore: bool,
    ignore_externals: bool,
    depth_as_sticky: bool,
    changelist: Option<Vec<String>>,
}

#[uniffi::export(with_foreign)]
pub trait StatusReceiver: Send + Sync + 'static {
    fn on_status_entry(&self, entry: StatusEntry) -> Result<(), error::CSharpError>;
}

#[derive(Debug, new, uniffi::Record)]
pub struct StatusResult {
    entries: Vec<StatusEntry>,
}

#[derive(Debug, uniffi::Record, svnexus_macro::ToDebugString)]
pub struct StatusEntry {
    path: String,
    node_kind: NodeKind,
    local_absolute_path: String,
    file_size: Option<u64>,
    versioned: bool,
    conflicted: bool,
    node_status: WorkingCopyStatus,
    text_status: WorkingCopyStatus,
    property_status: WorkingCopyStatus,
    wc_is_locked: bool,
    copied: bool,
    repository_root_url: Option<String>,
    repository_uuid: Option<String>,
    repository_relpath: Option<String>,
    revision: Option<RevisionNumber>,
    last_changed_revision: Option<RevisionNumber>,
    last_changed_date: i64,
    last_changed_author: Option<String>,
    switched: bool,
    file_external: bool,
    lock: Option<Lock>,
    changelist: Option<String>,
    depth: Depth,
    out_of_date_kind: NodeKind,
    repository_node_status: WorkingCopyStatus,
    repository_text_status: WorkingCopyStatus,
    repository_property_status: WorkingCopyStatus,
    repository_lock: Option<Lock>,
    out_of_date_changed_revision: Option<RevisionNumber>,
    out_of_date_changed_date: Option<i64>,
    out_of_date_changed_author: Option<String>,
    moved_from_absolute_path: Option<String>,
    moved_to_absolute_path: Option<String>,
}

impl StatusEntry {
    unsafe fn from_path_and_ptr(
        path: *const c_char,
        entry: *const ffi::svn_client_status_t,
    ) -> Self {
        unsafe {
            let path = path.to_str().to_string();

            let entry = entry.as_ref().unwrap();

            let node_kind = NodeKind::try_from(entry.kind).unwrap();

            let local_absolute_path = entry.local_abspath.to_str().to_string();

            // let file_size: u64 = entry.filesize.try_into().unwrap();
            let file_size = if entry.filesize == -1 {
                None
            } else {
                Some(entry.filesize.try_into().unwrap())
            };

            let versioned = entry.versioned != 0;

            let conflicted = entry.conflicted != 0;

            let node_status = WorkingCopyStatus::try_from(entry.node_status).unwrap();

            let text_status = WorkingCopyStatus::try_from(entry.text_status).unwrap();

            let property_status = WorkingCopyStatus::try_from(entry.prop_status).unwrap();

            let wc_is_locked = entry.wc_is_locked != 0;

            let copied = entry.copied != 0;

            let repository_root_url = entry.repos_root_url.to_nullable_string();

            let repository_uuid = entry.repos_uuid.to_nullable_string();

            let repository_relpath = entry.repos_relpath.to_nullable_string();

            let revision = RevisionNumber::try_from(entry.revision).ok();

            let last_changed_revision = entry.changed_rev.try_into().ok();

            let last_changed_date: i64 = entry.changed_date.try_into().unwrap();

            let last_changed_author = entry.changed_author.to_nullable_string();

            let switched = entry.switched != 0;

            let file_external = entry.file_external != 0;

            let lock = entry.lock.map(Lock::from);

            let changelist = entry.changelist.to_nullable_string();

            let depth = Depth::try_from(entry.depth).unwrap();

            let out_of_date_kind = NodeKind::try_from(entry.ood_kind).unwrap();

            let repository_node_status =
                WorkingCopyStatus::try_from(entry.repos_node_status).unwrap();

            let repository_text_status =
                WorkingCopyStatus::try_from(entry.repos_text_status).unwrap();

            let repository_property_status =
                WorkingCopyStatus::try_from(entry.repos_prop_status).unwrap();

            let repository_lock = entry.repos_lock.map(Lock::from);

            let out_of_date_changed_revision = entry.ood_changed_rev.try_into().ok();

            let out_of_date_changed_date = if entry.ood_changed_date == 0 {
                None
            } else {
                Some(entry.ood_changed_date.try_into().unwrap())
            };

            let out_of_date_changed_author = entry.ood_changed_author.to_nullable_string();

            let moved_from_absolute_path = entry.moved_from_abspath.to_nullable_string();

            let moved_to_absolute_path = entry.moved_to_abspath.to_nullable_string();

            Self {
                path,
                node_kind,
                local_absolute_path,
                file_size,
                versioned,
                conflicted,
                node_status,
                text_status,
                property_status,
                wc_is_locked,
                copied,
                repository_root_url,
                repository_uuid,
                repository_relpath,
                revision,
                last_changed_revision,
                last_changed_date,
                last_changed_author,
                switched,
                file_external,
                lock,
                changelist,
                depth,
                out_of_date_kind,
                repository_node_status,
                repository_text_status,
                repository_property_status,
                repository_lock,
                out_of_date_changed_revision,
                out_of_date_changed_date,
                out_of_date_changed_author,
                moved_from_absolute_path,
                moved_to_absolute_path,
            }
        }
    }
}

impl StatusEntry {
    fn fix_node_kind(&mut self) -> error::Result<()> {
        if self.node_status != WorkingCopyStatus::Unversioned {
            return Ok(());
        }

        let metadata = std::fs::metadata(self.path.as_str())?;

        if metadata.is_symlink() {
            self.node_kind = NodeKind::Symlink;
        } else if metadata.is_dir() {
            self.node_kind = NodeKind::Directory
        } else if metadata.is_file() {
            self.node_kind = NodeKind::File;
        }

        Ok(())
    }
}

impl Context {
    pub fn status_next(
        &mut self,
        opts: StatusOptions,
        receiver: Arc<dyn StatusReceiver>,
    ) -> error::Result<()> {
        unsafe extern "C" fn status_receiver(
            baton: *mut c_void,
            path: *const c_char,
            status: *const ffi::svn_client_status_t,
            _pool: *mut ffi::apr_pool_t,
        ) -> *mut ffi::svn_error_t {
            unsafe {
                let error = on_cancel(baton);
                if !error.is_null() {
                    return error;
                }

                let this = (baton as *mut ContextInner).as_mut().unwrap();

                let mut entry = StatusEntry::from_path_and_ptr(path, status);

                let result = entry.fix_node_kind();
                if let Err(err) = result {
                    tracing::warn!("Failed to detect node kind of {}:{}", entry.path, err);
                }

                this.status_receiver
                    .as_ref()
                    .expect("status receiver must be set")
                    .on_status_entry(entry)
                    .native_error()
            }
        }

        let mut result_revision: ffi::svn_revnum_t = 0;

        let revision = opts.revision.to_opt_revision();

        // tracing::info!("status options={:#?}", opts);

        unsafe {
            let mut pool = apr::Pool::create();
            let path = pool.canonicalize_dirent(opts.path.as_str())?;
            let changelist = opts
                .changelist
                .as_ref()
                .map(|e| pool.string_array(e.len(), e.iter()))
                .transpose()?
                .unwrap_or_default();

            self.inner.status_receiver = Some(receiver);

            let error = ffi::svn_client_status6(
                &mut result_revision as *mut _,
                self.ptr,
                path,
                &revision as *const _,
                opts.depth.into(),
                opts.get_all.into(),
                opts.check_out_of_date.into(),
                opts.check_working_copy.into(),
                opts.no_ignore.into(),
                opts.ignore_externals.into(),
                opts.depth_as_sticky.into(),
                changelist,
                Some(status_receiver),
                &mut *self.inner as *mut ContextInner as *mut c_void,
                pool.as_mut_ptr(),
            );

            self.inner.status_receiver.take();

            SVNError::from_nullable_ptr(error).context(builder::Svn)?;
        };

        Ok(())
    }

    pub fn status(&mut self, opts: StatusOptions) -> error::Result<StatusResult> {
        unsafe extern "C" fn status_receiver(
            baton: *mut c_void,
            path: *const c_char,
            status: *const ffi::svn_client_status_t,
            _pool: *mut ffi::apr_pool_t,
        ) -> *mut ffi::svn_error_t {
            unsafe {
                let error = on_cancel(baton);
                if !error.is_null() {
                    return error;
                }

                let this = (baton as *mut ContextInner).as_mut().unwrap();

                let mut entry = StatusEntry::from_path_and_ptr(path, status);

                let result = entry.fix_node_kind();
                if let Err(err) = result {
                    tracing::warn!("Failed to detect node kind of {}:{}", entry.path, err);
                }

                this.status_entries.push(entry);
            }
            // tracing::info!("translated entry: {:#?}", entry);

            svn_no_error()
        }

        let mut result_revision: ffi::svn_revnum_t = 0;

        let revision = opts.revision.to_opt_revision();

        // let mut changelist =
        // unsafe { apr::Array::from_string_list(opts.changelist.len(), opts.changelist)? };

        self.inner.status_entries.clear();

        unsafe {
            let mut pool = apr::Pool::create();
            let path = pool.canonicalize_dirent(opts.path.as_str())?;
            let changelist = opts
                .changelist
                .as_ref()
                .map(|e| pool.string_array(e.len(), e.iter()))
                .transpose()?
                .unwrap_or_default();

            let is_absolute = ffi::svn_dirent_is_absolute(path) != 0;
            if !is_absolute {
                return builder::InvalidArgument {
                    detail: format!("path is not absolute: {}", opts.path),
                }
                .fail();
            }

            let error = ffi::svn_client_status6(
                &mut result_revision as *mut _,
                self.ptr,
                path,
                &revision as *const _,
                opts.depth.into(),
                opts.get_all.into(),
                opts.check_out_of_date.into(),
                opts.check_working_copy.into(),
                opts.no_ignore.into(),
                opts.ignore_externals.into(),
                opts.depth_as_sticky.into(),
                changelist,
                Some(status_receiver),
                &mut *self.inner as *mut ContextInner as *mut c_void,
                pool.as_mut_ptr(),
            );

            SVNError::from_nullable_ptr(error).context(builder::Svn)?;
        };

        let entries = std::mem::take(&mut self.inner.status_entries);

        Ok(StatusResult::new(entries))
    }
}
