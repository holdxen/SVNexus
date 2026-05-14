use std::{
    collections::HashMap,
    ffi::{c_char, c_void},
};

use snafu::ResultExt;

use crate::{
    apr::{self, AprArray},
    subversion::wc::WorkingCopyConflictAction,
};
use crate::{
    error::{self, builder},
    subversion::{
        SVNError,
        context::{ContextInner, Depth, NodeKind},
        ffi, svn_no_error,
        wc::{WorkingCopyConflictReason, WorkingCopyOperation},
    },
    utils::{Boxed, CStringer, Pointer, SubversionStringer},
};

use super::Context;

#[derive(Debug, Clone, uniffi::Record)]
pub struct ConflictWalkOptions {
    local_absolute_path: String,
    depth: Depth,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct ConflictWalkResult {
    conflicts: Vec<Conflict>,
}

#[svnexus_macro::enum_converter(repr_type = ffi::svn_client_conflict_option_id_t)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, uniffi::Enum)]
pub enum ConflictOptionId {
    Undefined = ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_undefined,
    Postpone = ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_postpone,
    BaseText = ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_base_text,
    IncomingText = ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_incoming_text,
    WorkingText = ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_working_text,
    IncomingTextWhereConflicted = ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_incoming_text_where_conflicted,
    WorkingTextWhereConflicted = ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_working_text_where_conflicted,
    MergedText = ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_merged_text,
    Unspecified = ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_unspecified,
    AcceptCurrentWcState =
        ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_accept_current_wc_state,
    UpdateMoveDestination =
        ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_update_move_destination,
    UpdateAnyMovedAwayChildren = ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_update_any_moved_away_children,
    IncomingAddIgnore =
        ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_incoming_add_ignore,
    IncomingAddedFileTextMerge = ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_incoming_added_file_text_merge,
    IncomingAddedFileReplaceAndMerge = ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_incoming_added_file_replace_and_merge,
    IncomingAddedDirMerge =
        ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_incoming_added_dir_merge,
    IncomingAddedDirReplace =
        ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_incoming_added_dir_replace,
    IncomingAddedDirReplaceAndMerge = ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_incoming_added_dir_replace_and_merge,
    IncomingDeleteIgnore =
        ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_incoming_delete_ignore,
    IncomingDeleteAccept =
        ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_incoming_delete_accept,
    IncomingMoveFileTextMerge = ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_incoming_move_file_text_merge,
    IncomingMoveDirMerge =
        ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_incoming_move_dir_merge,
    LocalMoveFileTextMerge =
        ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_local_move_file_text_merge,
    LocalMoveDirMerge =
        ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_local_move_dir_merge,
    SiblingMoveFileTextMerge = ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_sibling_move_file_text_merge,
    SiblingMoveDirMerge =
        ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_sibling_move_dir_merge,
    BothMovedFileMerge =
        ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_both_moved_file_merge,
    BothMovedFileMoveMerge =
        ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_both_moved_file_move_merge,
    BothMovedDirMerge =
        ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_both_moved_dir_merge,
    BothMovedDirMoveMerge =
        ffi::svn_client_conflict_option_id_t_svn_client_conflict_option_both_moved_dir_move_merge,
}

pub struct ConflictOption {
    id: ConflictOptionId,
    label: String,
    description: String,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct ConflictPropertyValue {
    base_value: Option<String>,
    working_value: Option<String>,
    incoming_old_value: Option<String>,
    incoming_new_value: Option<String>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct ConflictInfo {
    local_absolute_path: String,
    operation: WorkingCopyOperation,
    incoming_change: WorkingCopyConflictAction,
    local_change: WorkingCopyConflictReason,
    recommended_option_id: ConflictOptionId,
}

impl From<*mut ffi::svn_client_conflict_t> for ConflictInfo {
    fn from(value: *mut ffi::svn_client_conflict_t) -> Self {
        unsafe {
            let local_absolute_path = ffi::svn_client_conflict_get_local_abspath(value)
                .to_str()
                .to_string();
            let operation = ffi::svn_client_conflict_get_operation(value)
                .try_into()
                .unwrap();
            let incoming_change = ffi::svn_client_conflict_get_incoming_change(value)
                .try_into()
                .unwrap();
            let local_change = ffi::svn_client_conflict_get_local_change(value)
                .try_into()
                .unwrap();
            let recommended_option_id = ffi::svn_client_conflict_get_recommended_option_id(value)
                .try_into()
                .unwrap();
            ConflictInfo {
                local_absolute_path,
                operation,
                incoming_change,
                local_change,
                recommended_option_id,
            }
        }
    }
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum Conflict {
    Text {
        info: ConflictInfo,
        base_absolute_path: Option<String>,
        working_absolute_path: Option<String>,
        incoming_old_absolute_path: Option<String>,
        incoming_new_absolute_path: Option<String>,
    },
    Property {
        info: ConflictInfo,
        properties: HashMap<String, ConflictPropertyValue>,
        description: Option<String>,
    },
    Tree {
        info: ConflictInfo,
        victim_node_kind: NodeKind,
    },
}

impl Context {
    pub fn conflict_walk(
        &mut self,
        opts: ConflictWalkOptions,
    ) -> error::Result<ConflictWalkResult> {
        // struct Baton {
        //     ctx: *mut ffi::svn_client_ctx_t,
        //     inner: *mut ContextInner,
        // }

        unsafe extern "C" fn conflict_walker(
            baton: *mut c_void,
            conflict: *mut ffi::svn_client_conflict_t,
            scratch_pool: *mut ffi::apr_pool_t,
        ) -> *mut ffi::svn_error_t {
            let mut is_text: ffi::svn_boolean_t = 0;
            let mut is_tree: ffi::svn_boolean_t = 0;
            let mut property_conflict: *mut ffi::apr_array_header_t = std::ptr::null_mut();

            unsafe {
                let inner = (baton as *mut ContextInner).as_mut().unwrap();

                let error = ffi::svn_client_conflict_get_conflicted(
                    is_text.pointer_mut(),
                    property_conflict.pointer_mut(),
                    is_tree.pointer_mut(),
                    conflict,
                    scratch_pool,
                    scratch_pool,
                );
                if !error.is_null() {
                    return error;
                }
                if is_text == 0 && is_tree == 0 && property_conflict.is_null() {
                    tracing::info!("Unspecified conflict");
                    return svn_no_error();
                }

                let info = ConflictInfo::from(conflict);

                if is_text != 0 {
                    // let mut options: *mut ffi::apr_array_header_t = std::ptr::null_mut();
                    // let error = ffi::svn_client_conflict_text_get_resolution_options(options.pointer_mut(), conflict, ctx, scratch_pool, scratch_pool);
                    // if !error.is_null() {
                    //     return error;
                    // }

                    let mut base_absolute_path: *const c_char = std::ptr::null_mut();
                    let mut working_absolute_path: *const c_char = std::ptr::null_mut();
                    let mut incoming_old_absolute_path: *const c_char = std::ptr::null_mut();
                    let mut incoming_new_absolute_path: *const c_char = std::ptr::null_mut();

                    let error = ffi::svn_client_conflict_text_get_contents(
                        base_absolute_path.pointer_mut(),
                        working_absolute_path.pointer_mut(),
                        incoming_old_absolute_path.pointer_mut(),
                        incoming_new_absolute_path.pointer_mut(),
                        conflict,
                        scratch_pool,
                        scratch_pool,
                    );
                    if !error.is_null() {
                        return error;
                    }

                    let base_absolute_path = base_absolute_path.to_nullable_string();
                    let working_absolute_path = working_absolute_path.to_nullable_string();
                    let incoming_old_absolute_path =
                        incoming_old_absolute_path.to_nullable_string();
                    let incoming_new_absolute_path =
                        incoming_new_absolute_path.to_nullable_string();

                    inner.conflicts.push(Conflict::Text {
                        base_absolute_path,
                        working_absolute_path,
                        incoming_old_absolute_path,
                        incoming_new_absolute_path,
                        info,
                    });
                } else if !property_conflict.is_null() {
                    let property_names = property_conflict.to_vec(|ptr| ptr);

                    let mut properties = HashMap::new();

                    for i in property_names {
                        let mut base_value: *const ffi::svn_string_t = std::ptr::null_mut();
                        let mut working_value: *const ffi::svn_string_t = std::ptr::null_mut();
                        let mut incoming_old_value: *const ffi::svn_string_t = std::ptr::null_mut();
                        let mut incoming_new_value: *const ffi::svn_string_t = std::ptr::null_mut();

                        let error = ffi::svn_client_conflict_prop_get_propvals(
                            base_value.pointer_mut(),
                            working_value.pointer_mut(),
                            incoming_old_value.pointer_mut(),
                            incoming_new_value.pointer_mut(),
                            conflict,
                            i,
                            scratch_pool,
                        );
                        if !error.is_null() {
                            return error;
                        }
                        let base_value = base_value.to_nullable_string();
                        let working_value = working_value.to_nullable_string();
                        let incoming_old_value = incoming_old_value.to_nullable_string();
                        let incoming_new_value = incoming_new_value.to_nullable_string();

                        properties.insert(
                            i.to_str().to_string(),
                            ConflictPropertyValue {
                                base_value,
                                working_value,
                                incoming_old_value,
                                incoming_new_value,
                            },
                        );
                    }
                    let mut description: *const c_char = std::ptr::null_mut();
                    let error = ffi::svn_client_conflict_prop_get_description(
                        description.pointer_mut(),
                        conflict,
                        scratch_pool,
                        scratch_pool,
                    );
                    if !error.is_null() {
                        return error;
                    }

                    let description = description.to_nullable_string();

                    inner.conflicts.push(Conflict::Property {
                        properties,
                        description,
                        info,
                    });
                } else if is_tree != 0 {
                    let victim_node_kind =
                        ffi::svn_client_conflict_tree_get_victim_node_kind(conflict)
                            .try_into()
                            .unwrap();

                    inner.conflicts.push(Conflict::Tree {
                        info,
                        victim_node_kind,
                    });
                }
            }
            svn_no_error()
        }
        unsafe {
            let mut pool = apr::Pool::create();
            let local_absolute_path = pool.string(opts.local_absolute_path)?;
            let error = ffi::svn_client_conflict_walk(
                local_absolute_path,
                opts.depth.into(),
                Some(conflict_walker),
                self.inner.inner_void_pointer_mut(),
                self.ctx(),
                pool.as_mut_ptr(),
            );
            SVNError::from_nullable_ptr(error).context(builder::Svn)?;
        };

        Ok(ConflictWalkResult {
            conflicts: std::mem::take(&mut self.inner.conflicts),
        })
    }
}
