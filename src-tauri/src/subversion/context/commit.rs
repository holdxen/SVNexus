use std::collections::HashMap;
use std::ffi::CStr;

use derive_new::new;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

use crate::{
    apr,
    error::{self, builder},
    extensions::Canonicalization,
    utils::{Boxed, CStringer},
};

use super::*;

#[derive(new, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CommitResult {
    items: Vec<CommitItem>,
    info: Option<super::CommitInfo>,
}

#[derive(Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CommitItem {
    path: Option<String>,
    kind: NodeKind,
    url: String,
    #[ts(type = "number | null")]
    revision: Option<RevisionNumber>,
    copy_from_url: Option<String>,
    #[ts(type = "number | null")]
    copy_from_revision: Option<RevisionNumber>,
    state: u8,
    incoming_property_changes: HashMap<String, Option<String>>,
    outgoing_property_changes: HashMap<String, Option<String>>,
    session_real_path: Option<String>,
    move_from_absolute_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CommitOptions {
    targets: Vec<String>,
    depth: super::Depth,
    keep_locks: bool,
    keep_changelist: bool,
    commit_as_operations: bool,
    include_file_externals: bool,
    include_dir_externals: bool,
    changelists: Option<Vec<String>>,
    revision_property_table: Option<HashMap<String, String>>,
    commit_message: String,
}

impl CommitItem {
    // pub(super) unsafe fn from_items(items: *const ffi::apr_array_header_t) -> Vec<Self> {
    //     let items = unsafe { &*items };

    //     let mut ret = Vec::with_capacity(items.nelts.try_into().expect("Failed to convert size"));

    //     for i in 0..items.nelts {
    //         let elements = items.elts as *const *const ffi::svn_client_commit_item3_t;

    //         let element = unsafe {
    //             let offset = elements.offset(i as _);
    //             if (*offset).is_null() {
    //                 continue;
    //             }
    //             *offset
    //         };

    //         let item = unsafe { Self::from_item(element) };

    //         ret.push(item);
    //     }

    //     ret
    // }

    pub(super) unsafe fn from_item(item: *const ffi::svn_client_commit_item3_t) -> Self {
        unsafe {
            let item = item.as_ref().expect("Failed to get reference");

            tracing::info!("item path={:?}", CStr::from_ptr(item.path));
            let path = item.path.to_nullable_string();

            let kind = NodeKind::try_from(item.kind).expect("Unexpected failure");

            let url = item.url.to_str().to_string();

            let copy_from_url = item.copyfrom_url.to_nullable_string();

            let mut incoming_property_changes = HashMap::new();
            if !item.incoming_prop_changes.is_null() {
                let prop = item
                    .incoming_prop_changes
                    .as_ref()
                    .expect("Failed to get reference to property array");

                incoming_property_changes.reserve(prop.nelts as _);

                for i in 0..prop.nelts {
                    let elements = prop.elts as *const *const ffi::svn_prop_t;

                    let offset = elements.offset(i as _);
                    if (*offset).is_null() {
                        continue;
                    }
                    let element = &**offset;

                    let key = element.name.to_str().to_string();

                    // value 为 NULL 表示该属性被删除（见 libsvn_client/ra.c）
                    let value = if element.value.is_null() {
                        None
                    } else {
                        let slice = std::slice::from_raw_parts(
                            (*element.value).data as *const u8,
                            (*element.value).len,
                        );
                        Some(String::from_utf8(slice.to_vec()).expect("Invalid UTF-8 data"))
                    };

                    incoming_property_changes.insert(key, value);
                }
            }

            let mut outgoing_property_changes = HashMap::new();
            if !item.outgoing_prop_changes.is_null() {
                let prop = &*item.outgoing_prop_changes;

                outgoing_property_changes.reserve(prop.nelts as _);

                for i in 0..prop.nelts {
                    let elements = prop.elts as *const *const ffi::svn_prop_t;

                    let offset = elements.offset(i as _);
                    if (*offset).is_null() {
                        continue;
                    }
                    let element = &**offset;

                    let key = element.name.to_str().to_string();

                    // value 为 NULL 表示该属性被删除
                    let value = if element.value.is_null() {
                        None
                    } else {
                        let slice = std::slice::from_raw_parts(
                            (*element.value).data as *const u8,
                            (*element.value).len,
                        );
                        Some(String::from_utf8(slice.to_vec()).expect("Invalid UTF-8 data"))
                    };

                    outgoing_property_changes.insert(key, value);
                }
            }

            let revision = item.revision.try_into().ok();
            let state = item.state_flags;

            let session_real_path = item.session_relpath.to_nullable_string();

            let copy_from_revision = if item.copyfrom_url.is_null() || item.copyfrom_rev < 0 {
                None
            } else {
                Some(
                    item.copyfrom_rev
                        .try_into()
                        .expect("Failed to convert revision number"),
                )
            };

            let move_from_absolute_path = item.moved_from_abspath.to_nullable_string();

            Self {
                path,
                kind,
                url,
                revision,
                copy_from_url,
                copy_from_revision,
                state,
                incoming_property_changes,
                outgoing_property_changes,
                session_real_path,
                move_from_absolute_path,
            }
        }
    }
}

impl Context {
    pub fn commit(&mut self, opts: CommitOptions) -> error::Result<CommitResult> {
        tracing::info!("Commit optinos: {:#?}", opts);
        let result = unsafe {
            if opts.commit_message.as_bytes().contains(&b'\0') {
                return builder::InvalidArgument {
                    detail: "Invalid commit message",
                }
                .fail();
            }
            let mut pool = apr::Pool::create();

            let targets =
                pool.canonicalize_dirent_array(opts.targets.len(), opts.targets.iter())?;

            let changelists = opts
                .changelists
                .map(|e| pool.string_array(e.len(), e.iter()))
                .transpose()?
                .unwrap_or_default();

            let revision_properties_table = opts
                .revision_property_table
                .as_ref()
                .map(|e| {
                    pool.string_hash_map(
                        e.iter(),
                        |p, k| p.string(k).map(|o| o as _),
                        |p, v| p.svn_string(v).map(|o| o as _),
                    )
                })
                .transpose()?
                .unwrap_or_default();

            tracing::info!("Table is {:?}", revision_properties_table);

            self.inner.commit_items.clear();
            self.inner.commit_message = opts.commit_message;

            let error = ffi::svn_client_commit6(
                targets,
                opts.depth.into(),
                opts.keep_locks.into(),
                opts.keep_changelist.into(),
                opts.commit_as_operations.into(),
                opts.include_file_externals.into(),
                opts.include_dir_externals.into(),
                changelists,
                revision_properties_table,
                Some(commit_callback),
                self.inner.inner_void_pointer_mut(),
                self.ctx(),
                pool.as_mut_ptr(),
            );

            self.inner.commit_message.clear();

            SubversionError::from_nullable_ptr(error).context(builder::Subversion)?;

            self.take_commit_result()
        };

        Ok(result)
    }
}
