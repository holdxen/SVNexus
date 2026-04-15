pub mod context;
pub mod export;
pub mod ra;
pub mod stream;
mod utils;
pub mod version;
pub mod wc;
mod property;

#[cfg(test)]
mod tests;

use crate::{apr, utils::CStringer};
use serde::{Deserialize, Serialize};

#[allow(bad_style)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(dead_code)]
#[allow(clippy::upper_case_acronyms)]
#[allow(unnecessary_transmutes)]
#[allow(unsafe_op_in_unsafe_fn)]
pub mod ffi {
    pub use crate::apr::ffi::*;
    include!(concat!(env!("OUT_DIR"), "/subversion.rs"));
}

// pub struct WorkingCopyNotify {
//     path: String,
//     action: ffi::svn_wc_notify_action_t,
//     kind: ffi::svn_node_kind_t,
//     mime_type: Option<String>,
// }

// pub struct WCNotify {
//     path: String,
//     action: ffi::svn_wc_notify_action_t,
//     kind: ffi::svn_node_kind_t,
//     mime_type: Option<String>,
// }

#[derive(Debug, uniffi::Object, svnexus_macro::IsMethods)]
pub struct SvnErrnoConstants {
    pub warning: i32,
    pub bad_containing_pool: i32,
    pub bad_filename: i32,
    pub bad_url: i32,
    pub bad_date: i32,
    pub bad_mime_type: i32,
    pub bad_property_value: i32,
    pub bad_version_file_format: i32,
    pub bad_relative_path: i32,
    pub bad_uuid: i32,
    pub bad_config_value: i32,
    pub bad_server_specification: i32,
    pub bad_checksum_kind: i32,
    pub bad_checksum_parse: i32,
    pub bad_token: i32,
    pub bad_changelist_name: i32,
    pub bad_atomic: i32,
    pub bad_compression_method: i32,
    pub bad_property_value_eol: i32,
    pub xml_attrib_not_found: i32,
    pub xml_missing_ancestry: i32,
    pub xml_unknown_encoding: i32,
    pub xml_malformed: i32,
    pub xml_unescapable_data: i32,
    pub xml_unexpected_element: i32,
    pub io_inconsistent_eol: i32,
    pub io_unknown_eol: i32,
    pub io_corrupt_eol: i32,
    pub io_unique_names_exhausted: i32,
    pub io_pipe_frame_error: i32,
    pub io_pipe_read_error: i32,
    pub io_write_error: i32,
    pub io_pipe_write_error: i32,
    pub stream_unexpected_eof: i32,
    pub stream_malformed_data: i32,
    pub stream_unrecognized_data: i32,
    pub stream_seek_not_supported: i32,
    pub stream_not_supported: i32,
    pub node_unknown_kind: i32,
    pub node_unexpected_kind: i32,
    pub entry_not_found: i32,
    pub entry_exists: i32,
    pub entry_missing_revision: i32,
    pub entry_missing_url: i32,
    pub entry_attribute_invalid: i32,
    pub entry_forbidden: i32,
    pub wc_obstructed_update: i32,
    pub wc_unwind_mismatch: i32,
    pub wc_unwind_empty: i32,
    pub wc_unwind_not_empty: i32,
    pub wc_locked: i32,
    pub wc_not_locked: i32,
    pub wc_invalid_lock: i32,
    pub wc_not_working_copy: i32,
    pub wc_not_directory: i32,
    pub wc_not_file: i32,
    pub wc_bad_adm_log: i32,
    pub wc_path_not_found: i32,
    pub wc_not_up_to_date: i32,
    pub wc_left_local_mod: i32,
    pub wc_schedule_conflict: i32,
    pub wc_path_found: i32,
    pub wc_found_conflict: i32,
    pub wc_corrupt: i32,
    pub wc_corrupt_text_base: i32,
    pub wc_node_kind_change: i32,
    pub wc_invalid_op_on_cwd: i32,
    pub wc_bad_adm_log_start: i32,
    pub wc_unsupported_format: i32,
    pub wc_bad_path: i32,
    pub wc_invalid_schedule: i32,
    pub wc_invalid_relocation: i32,
    pub wc_invalid_switch: i32,
    pub wc_mismatched_changelist: i32,
    pub wc_conflict_resolver_failure: i32,
    pub wc_copyfrom_path_not_found: i32,
    pub wc_changelist_move: i32,
    pub wc_cannot_delete_file_external: i32,
    pub wc_cannot_move_file_external: i32,
    pub wc_db_error: i32,
    pub wc_missing: i32,
    pub wc_not_symlink: i32,
    pub wc_path_unexpected_status: i32,
    pub wc_upgrade_required: i32,
    pub wc_cleanup_required: i32,
    pub wc_invalid_operation_depth: i32,
    pub wc_path_access_denied: i32,
    pub wc_mixed_revisions: i32,
    pub wc_duplicate_externals_target: i32,
    pub wc_incompatible_settings: i32,
    pub wc_deprecated_api_store_pristine: i32,
    pub wc_pristine_dehydrated: i32,
    pub fs_general: i32,
    pub fs_cleanup: i32,
    pub fs_already_open: i32,
    pub fs_not_open: i32,
    pub fs_corrupt: i32,
    pub fs_path_syntax: i32,
    pub fs_no_such_revision: i32,
    pub fs_no_such_transaction: i32,
    pub fs_no_such_entry: i32,
    pub fs_no_such_representation: i32,
    pub fs_no_such_string: i32,
    pub fs_no_such_copy: i32,
    pub fs_transaction_not_mutable: i32,
    pub fs_not_found: i32,
    pub fs_id_not_found: i32,
    pub fs_not_id: i32,
    pub fs_not_directory: i32,
    pub fs_not_file: i32,
    pub fs_not_single_path_component: i32,
    pub fs_not_mutable: i32,
    pub fs_already_exists: i32,
    pub fs_root_dir: i32,
    pub fs_not_txn_root: i32,
    pub fs_not_revision_root: i32,
    pub fs_conflict: i32,
    pub fs_rep_changed: i32,
    pub fs_rep_not_mutable: i32,
    pub fs_malformed_skel: i32,
    pub fs_txn_out_of_date: i32,
    pub fs_berkeley_db: i32,
    pub fs_berkeley_db_deadlock: i32,
    pub fs_transaction_dead: i32,
    pub fs_transaction_not_dead: i32,
    pub fs_unknown_fs_type: i32,
    pub fs_no_user: i32,
    pub fs_path_already_locked: i32,
    pub fs_path_not_locked: i32,
    pub fs_bad_lock_token: i32,
    pub fs_no_lock_token: i32,
    pub fs_lock_owner_mismatch: i32,
    pub fs_no_such_lock: i32,
    pub fs_lock_expired: i32,
    pub fs_out_of_date: i32,
    pub fs_unsupported_format: i32,
    pub fs_rep_being_written: i32,
    pub fs_txn_name_too_long: i32,
    pub fs_no_such_node_origin: i32,
    pub fs_unsupported_upgrade: i32,
    pub fs_no_such_checksum_rep: i32,
    pub fs_prop_basevalue_mismatch: i32,
    pub fs_incorrect_editor_completion: i32,
    pub fs_packed_revprop_read_failure: i32,
    pub fs_revprop_cache_init_failure: i32,
    pub fs_malformed_txn_id: i32,
    pub fs_index_corruption: i32,
    pub fs_index_revision: i32,
    pub fs_index_overflow: i32,
    pub fs_container_index: i32,
    pub fs_index_inconsistent: i32,
    pub fs_lock_operation_failed: i32,
    pub fs_unsupported_type: i32,
    pub fs_container_size: i32,
    pub fs_malformed_noderev_id: i32,
    pub fs_invalid_generation: i32,
    pub fs_corrupt_revprop_manifest: i32,
    pub fs_corrupt_proplist: i32,
    pub fs_ambiguous_checksum_rep: i32,
    pub fs_unrecognized_ioctl_code: i32,
    pub fs_rep_sharing_not_allowed: i32,
    pub fs_rep_sharing_not_supported: i32,
    pub repos_locked: i32,
    pub repos_hook_failure: i32,
    pub repos_bad_args: i32,
    pub repos_no_data_for_report: i32,
    pub repos_bad_revision_report: i32,
    pub repos_unsupported_version: i32,
    pub repos_disabled_feature: i32,
    pub repos_post_commit_hook_failed: i32,
    pub repos_post_lock_hook_failed: i32,
    pub repos_post_unlock_hook_failed: i32,
    pub repos_unsupported_upgrade: i32,
    pub ra_illegal_url: i32,
    pub ra_not_authorized: i32,
    pub ra_unknown_auth: i32,
    pub ra_not_implemented: i32,
    pub ra_out_of_date: i32,
    pub ra_no_repos_uuid: i32,
    pub ra_unsupported_abi_version: i32,
    pub ra_not_locked: i32,
    pub ra_partial_replay_not_supported: i32,
    pub ra_uuid_mismatch: i32,
    pub ra_repos_root_url_mismatch: i32,
    pub ra_session_url_mismatch: i32,
    pub ra_cannot_create_tunnel: i32,
    pub ra_cannot_create_session: i32,
    pub ra_dav_sock_init: i32,
    pub ra_dav_creating_request: i32,
    pub ra_dav_request_failed: i32,
    pub ra_dav_options_req_failed: i32,
    pub ra_dav_props_not_found: i32,
    pub ra_dav_already_exists: i32,
    pub ra_dav_invalid_config_value: i32,
    pub ra_dav_path_not_found: i32,
    pub ra_dav_proppatch_failed: i32,
    pub ra_dav_malformed_data: i32,
    pub ra_dav_response_header_badness: i32,
    pub ra_dav_relocated: i32,
    pub ra_dav_conn_timeout: i32,
    pub ra_dav_forbidden: i32,
    pub ra_dav_precondition_failed: i32,
    pub ra_dav_method_not_allowed: i32,
    pub ra_local_repos_not_found: i32,
    pub ra_local_repos_open_failed: i32,
    pub svndiff_invalid_header: i32,
    pub svndiff_corrupt_window: i32,
    pub svndiff_backward_view: i32,
    pub svndiff_invalid_ops: i32,
    pub svndiff_unexpected_end: i32,
    pub svndiff_invalid_compressed_data: i32,
    pub apmod_missing_path_to_fs: i32,
    pub apmod_malformed_uri: i32,
    pub apmod_activity_not_found: i32,
    pub apmod_bad_baseline: i32,
    pub apmod_connection_aborted: i32,
    pub client_versioned_path_required: i32,
    pub client_ra_access_required: i32,
    pub client_bad_revision: i32,
    pub client_duplicate_commit_url: i32,
    pub client_is_binary_file: i32,
    pub client_invalid_externals_description: i32,
    pub client_modified: i32,
    pub client_is_directory: i32,
    pub client_revision_range: i32,
    pub client_invalid_relocation: i32,
    pub client_revision_author_contains_newline: i32,
    pub client_property_name: i32,
    pub client_unrelated_resources: i32,
    pub client_missing_lock_token: i32,
    pub client_multiple_sources_disallowed: i32,
    pub client_no_versioned_parent: i32,
    pub client_not_ready_to_merge: i32,
    pub client_file_external_overwrite_versioned: i32,
    pub client_patch_bad_strip_count: i32,
    pub client_cycle_detected: i32,
    pub client_merge_update_required: i32,
    pub client_invalid_mergeinfo_no_mergetracking: i32,
    pub client_no_lock_token: i32,
    pub client_forbidden_by_server: i32,
    pub client_conflict_option_not_applicable: i32,
    pub base: i32,
    pub plugin_load_failure: i32,
    pub malformed_file: i32,
    pub incomplete_data: i32,
    pub incorrect_params: i32,
    pub unversioned_resource: i32,
    pub test_failed: i32,
    pub unsupported_feature: i32,
    pub bad_prop_kind: i32,
    pub illegal_target: i32,
    pub delta_md5_checksum_absent: i32,
    pub dir_not_empty: i32,
    pub external_program: i32,
    pub swig_py_exception_set: i32,
    pub checksum_mismatch: i32,
    pub cancelled: i32,
    pub invalid_diff_option: i32,
    pub property_not_found: i32,
    pub no_auth_file_path: i32,
    pub version_mismatch: i32,
    pub mergeinfo_parse_error: i32,
    pub cease_invocation: i32,
    pub revnum_parse_failure: i32,
    pub iter_break: i32,
    pub unknown_changelist: i32,
    pub reserved_filename_specified: i32,
    pub unknown_capability: i32,
    pub test_skipped: i32,
    pub no_apr_memcache: i32,
    pub atomic_init_failure: i32,
    pub sqlite_error: i32,
    pub sqlite_readonly: i32,
    pub sqlite_unsupported_schema: i32,
    pub sqlite_busy: i32,
    pub sqlite_resetting_for_rollback: i32,
    pub sqlite_constraint: i32,
    pub too_many_memcached_servers: i32,
    pub malformed_version_string: i32,
    pub corrupted_atomic_storage: i32,
    pub utf8proc_error: i32,
    pub utf8_glob: i32,
    pub corrupt_packed_data: i32,
    pub composed_error: i32,
    pub invalid_input: i32,
    pub sqlite_rollback_failed: i32,
    pub lz4_compression_failed: i32,
    pub lz4_decompression_failed: i32,
    pub canonicalization_failed: i32,
    pub cl_arg_parsing_error: i32,
    pub cl_insufficient_args: i32,
    pub cl_mutually_exclusive_args: i32,
    pub cl_adm_dir_reserved: i32,
    pub cl_log_message_is_versioned_file: i32,
    pub cl_log_message_is_pathname: i32,
    pub cl_commit_in_added_dir: i32,
    pub cl_no_external_editor: i32,
    pub cl_bad_log_message: i32,
    pub cl_unnecessary_log_message: i32,
    pub cl_no_external_merge_tool: i32,
    pub cl_error_processing_externals: i32,
    pub cl_repos_verify_failed: i32,
    pub ra_svn_cmd_err: i32,
    pub ra_svn_unknown_cmd: i32,
    pub ra_svn_connection_closed: i32,
    pub ra_svn_io_error: i32,
    pub ra_svn_malformed_data: i32,
    pub ra_svn_repos_not_found: i32,
    pub ra_svn_bad_version: i32,
    pub ra_svn_no_mechanisms: i32,
    pub ra_svn_edit_aborted: i32,
    pub ra_svn_request_size: i32,
    pub ra_svn_response_size: i32,
    pub authn_creds_unavailable: i32,
    pub authn_no_provider: i32,
    pub authn_providers_exhausted: i32,
    pub authn_creds_not_saved: i32,
    pub authn_failed: i32,
    pub authz_root_unreadable: i32,
    pub authz_unreadable: i32,
    pub authz_partially_readable: i32,
    pub authz_invalid_config: i32,
    pub authz_unwritable: i32,
    pub diff_datasource_modified: i32,
    pub diff_unexpected_data: i32,
    pub ra_serf_sspi_initialisation_failed: i32,
    pub ra_serf_ssl_cert_untrusted: i32,
    pub ra_serf_gssapi_initialisation_failed: i32,
    pub ra_serf_wrapped_error: i32,
    pub ra_serf_stream_bucket_read_error: i32,
    pub assertion_fail: i32,
    pub assertion_only_tracing_links: i32,
    pub asn1_out_of_data: i32,
    pub asn1_unexpected_tag: i32,
    pub asn1_invalid_length: i32,
    pub asn1_length_mismatch: i32,
    pub asn1_invalid_data: i32,
    pub x509_feature_unavailable: i32,
    pub x509_cert_invalid_pem: i32,
    pub x509_cert_invalid_format: i32,
    pub x509_cert_invalid_version: i32,
    pub x509_cert_invalid_serial: i32,
    pub x509_cert_invalid_alg: i32,
    pub x509_cert_invalid_name: i32,
    pub x509_cert_invalid_date: i32,
    pub x509_cert_invalid_pubkey: i32,
    pub x509_cert_invalid_signature: i32,
    pub x509_cert_invalid_extensions: i32,
    pub x509_cert_unknown_version: i32,
    pub x509_cert_unknown_pk_alg: i32,
    pub x509_cert_sig_mismatch: i32,
    pub x509_cert_verify_failed: i32,
}

#[uniffi::export]
impl SvnErrnoConstants {
    /// Creates a new instance of SvnErrnoConstants,
    /// initialized with values from the ffi module.

    #[uniffi::constructor]
    pub fn new() -> Self {
        SvnErrnoConstants {
            warning: ffi::svn_errno_t_SVN_WARNING as _,
            bad_containing_pool: ffi::svn_errno_t_SVN_ERR_BAD_CONTAINING_POOL as _,
            bad_filename: ffi::svn_errno_t_SVN_ERR_BAD_FILENAME as _,
            bad_url: ffi::svn_errno_t_SVN_ERR_BAD_URL as _,
            bad_date: ffi::svn_errno_t_SVN_ERR_BAD_DATE as _,
            bad_mime_type: ffi::svn_errno_t_SVN_ERR_BAD_MIME_TYPE as _,
            bad_property_value: ffi::svn_errno_t_SVN_ERR_BAD_PROPERTY_VALUE as _,
            bad_version_file_format: ffi::svn_errno_t_SVN_ERR_BAD_VERSION_FILE_FORMAT as _,
            bad_relative_path: ffi::svn_errno_t_SVN_ERR_BAD_RELATIVE_PATH as _,
            bad_uuid: ffi::svn_errno_t_SVN_ERR_BAD_UUID as _,
            bad_config_value: ffi::svn_errno_t_SVN_ERR_BAD_CONFIG_VALUE as _,
            bad_server_specification: ffi::svn_errno_t_SVN_ERR_BAD_SERVER_SPECIFICATION as _,
            bad_checksum_kind: ffi::svn_errno_t_SVN_ERR_BAD_CHECKSUM_KIND as _,
            bad_checksum_parse: ffi::svn_errno_t_SVN_ERR_BAD_CHECKSUM_PARSE as _,
            bad_token: ffi::svn_errno_t_SVN_ERR_BAD_TOKEN as _,
            bad_changelist_name: ffi::svn_errno_t_SVN_ERR_BAD_CHANGELIST_NAME as _,
            bad_atomic: ffi::svn_errno_t_SVN_ERR_BAD_ATOMIC as _,
            bad_compression_method: ffi::svn_errno_t_SVN_ERR_BAD_COMPRESSION_METHOD as _,
            bad_property_value_eol: ffi::svn_errno_t_SVN_ERR_BAD_PROPERTY_VALUE_EOL as _,
            xml_attrib_not_found: ffi::svn_errno_t_SVN_ERR_XML_ATTRIB_NOT_FOUND as _,
            xml_missing_ancestry: ffi::svn_errno_t_SVN_ERR_XML_MISSING_ANCESTRY as _,
            xml_unknown_encoding: ffi::svn_errno_t_SVN_ERR_XML_UNKNOWN_ENCODING as _,
            xml_malformed: ffi::svn_errno_t_SVN_ERR_XML_MALFORMED as _,
            xml_unescapable_data: ffi::svn_errno_t_SVN_ERR_XML_UNESCAPABLE_DATA as _,
            xml_unexpected_element: ffi::svn_errno_t_SVN_ERR_XML_UNEXPECTED_ELEMENT as _,
            io_inconsistent_eol: ffi::svn_errno_t_SVN_ERR_IO_INCONSISTENT_EOL as _,
            io_unknown_eol: ffi::svn_errno_t_SVN_ERR_IO_UNKNOWN_EOL as _,
            io_corrupt_eol: ffi::svn_errno_t_SVN_ERR_IO_CORRUPT_EOL as _,
            io_unique_names_exhausted: ffi::svn_errno_t_SVN_ERR_IO_UNIQUE_NAMES_EXHAUSTED as _,
            io_pipe_frame_error: ffi::svn_errno_t_SVN_ERR_IO_PIPE_FRAME_ERROR as _,
            io_pipe_read_error: ffi::svn_errno_t_SVN_ERR_IO_PIPE_READ_ERROR as _,
            io_write_error: ffi::svn_errno_t_SVN_ERR_IO_WRITE_ERROR as _,
            io_pipe_write_error: ffi::svn_errno_t_SVN_ERR_IO_PIPE_WRITE_ERROR as _,
            stream_unexpected_eof: ffi::svn_errno_t_SVN_ERR_STREAM_UNEXPECTED_EOF as _,
            stream_malformed_data: ffi::svn_errno_t_SVN_ERR_STREAM_MALFORMED_DATA as _,
            stream_unrecognized_data: ffi::svn_errno_t_SVN_ERR_STREAM_UNRECOGNIZED_DATA as _,
            stream_seek_not_supported: ffi::svn_errno_t_SVN_ERR_STREAM_SEEK_NOT_SUPPORTED as _,
            stream_not_supported: ffi::svn_errno_t_SVN_ERR_STREAM_NOT_SUPPORTED as _,
            node_unknown_kind: ffi::svn_errno_t_SVN_ERR_NODE_UNKNOWN_KIND as _,
            node_unexpected_kind: ffi::svn_errno_t_SVN_ERR_NODE_UNEXPECTED_KIND as _,
            entry_not_found: ffi::svn_errno_t_SVN_ERR_ENTRY_NOT_FOUND as _,
            entry_exists: ffi::svn_errno_t_SVN_ERR_ENTRY_EXISTS as _,
            entry_missing_revision: ffi::svn_errno_t_SVN_ERR_ENTRY_MISSING_REVISION as _,
            entry_missing_url: ffi::svn_errno_t_SVN_ERR_ENTRY_MISSING_URL as _,
            entry_attribute_invalid: ffi::svn_errno_t_SVN_ERR_ENTRY_ATTRIBUTE_INVALID as _,
            entry_forbidden: ffi::svn_errno_t_SVN_ERR_ENTRY_FORBIDDEN as _,
            wc_obstructed_update: ffi::svn_errno_t_SVN_ERR_WC_OBSTRUCTED_UPDATE as _,
            wc_unwind_mismatch: ffi::svn_errno_t_SVN_ERR_WC_UNWIND_MISMATCH as _,
            wc_unwind_empty: ffi::svn_errno_t_SVN_ERR_WC_UNWIND_EMPTY as _,
            wc_unwind_not_empty: ffi::svn_errno_t_SVN_ERR_WC_UNWIND_NOT_EMPTY as _,
            wc_locked: ffi::svn_errno_t_SVN_ERR_WC_LOCKED as _,
            wc_not_locked: ffi::svn_errno_t_SVN_ERR_WC_NOT_LOCKED as _,
            wc_invalid_lock: ffi::svn_errno_t_SVN_ERR_WC_INVALID_LOCK as _,
            wc_not_working_copy: ffi::svn_errno_t_SVN_ERR_WC_NOT_WORKING_COPY as _,
            wc_not_directory: ffi::svn_errno_t_SVN_ERR_WC_NOT_DIRECTORY as _,
            wc_not_file: ffi::svn_errno_t_SVN_ERR_WC_NOT_FILE as _,
            wc_bad_adm_log: ffi::svn_errno_t_SVN_ERR_WC_BAD_ADM_LOG as _,
            wc_path_not_found: ffi::svn_errno_t_SVN_ERR_WC_PATH_NOT_FOUND as _,
            wc_not_up_to_date: ffi::svn_errno_t_SVN_ERR_WC_NOT_UP_TO_DATE as _,
            wc_left_local_mod: ffi::svn_errno_t_SVN_ERR_WC_LEFT_LOCAL_MOD as _,
            wc_schedule_conflict: ffi::svn_errno_t_SVN_ERR_WC_SCHEDULE_CONFLICT as _,
            wc_path_found: ffi::svn_errno_t_SVN_ERR_WC_PATH_FOUND as _,
            wc_found_conflict: ffi::svn_errno_t_SVN_ERR_WC_FOUND_CONFLICT as _,
            wc_corrupt: ffi::svn_errno_t_SVN_ERR_WC_CORRUPT as _,
            wc_corrupt_text_base: ffi::svn_errno_t_SVN_ERR_WC_CORRUPT_TEXT_BASE as _,
            wc_node_kind_change: ffi::svn_errno_t_SVN_ERR_WC_NODE_KIND_CHANGE as _,
            wc_invalid_op_on_cwd: ffi::svn_errno_t_SVN_ERR_WC_INVALID_OP_ON_CWD as _,
            wc_bad_adm_log_start: ffi::svn_errno_t_SVN_ERR_WC_BAD_ADM_LOG_START as _,
            wc_unsupported_format: ffi::svn_errno_t_SVN_ERR_WC_UNSUPPORTED_FORMAT as _,
            wc_bad_path: ffi::svn_errno_t_SVN_ERR_WC_BAD_PATH as _,
            wc_invalid_schedule: ffi::svn_errno_t_SVN_ERR_WC_INVALID_SCHEDULE as _,
            wc_invalid_relocation: ffi::svn_errno_t_SVN_ERR_WC_INVALID_RELOCATION as _,
            wc_invalid_switch: ffi::svn_errno_t_SVN_ERR_WC_INVALID_SWITCH as _,
            wc_mismatched_changelist: ffi::svn_errno_t_SVN_ERR_WC_MISMATCHED_CHANGELIST as _,
            wc_conflict_resolver_failure: ffi::svn_errno_t_SVN_ERR_WC_CONFLICT_RESOLVER_FAILURE
                as _,
            wc_copyfrom_path_not_found: ffi::svn_errno_t_SVN_ERR_WC_COPYFROM_PATH_NOT_FOUND as _,
            wc_changelist_move: ffi::svn_errno_t_SVN_ERR_WC_CHANGELIST_MOVE as _,
            wc_cannot_delete_file_external: ffi::svn_errno_t_SVN_ERR_WC_CANNOT_DELETE_FILE_EXTERNAL
                as _,
            wc_cannot_move_file_external: ffi::svn_errno_t_SVN_ERR_WC_CANNOT_MOVE_FILE_EXTERNAL
                as _,
            wc_db_error: ffi::svn_errno_t_SVN_ERR_WC_DB_ERROR as _,
            wc_missing: ffi::svn_errno_t_SVN_ERR_WC_MISSING as _,
            wc_not_symlink: ffi::svn_errno_t_SVN_ERR_WC_NOT_SYMLINK as _,
            wc_path_unexpected_status: ffi::svn_errno_t_SVN_ERR_WC_PATH_UNEXPECTED_STATUS as _,
            wc_upgrade_required: ffi::svn_errno_t_SVN_ERR_WC_UPGRADE_REQUIRED as _,
            wc_cleanup_required: ffi::svn_errno_t_SVN_ERR_WC_CLEANUP_REQUIRED as _,
            wc_invalid_operation_depth: ffi::svn_errno_t_SVN_ERR_WC_INVALID_OPERATION_DEPTH as _,
            wc_path_access_denied: ffi::svn_errno_t_SVN_ERR_WC_PATH_ACCESS_DENIED as _,
            wc_mixed_revisions: ffi::svn_errno_t_SVN_ERR_WC_MIXED_REVISIONS as _,
            wc_duplicate_externals_target: ffi::svn_errno_t_SVN_ERR_WC_DUPLICATE_EXTERNALS_TARGET
                as _,
            wc_incompatible_settings: ffi::svn_errno_t_SVN_ERR_WC_INCOMPATIBLE_SETTINGS as _,
            wc_deprecated_api_store_pristine:
                ffi::svn_errno_t_SVN_ERR_WC_DEPRECATED_API_STORE_PRISTINE as _,
            wc_pristine_dehydrated: ffi::svn_errno_t_SVN_ERR_WC_PRISTINE_DEHYDRATED as _,
            fs_general: ffi::svn_errno_t_SVN_ERR_FS_GENERAL as _,
            fs_cleanup: ffi::svn_errno_t_SVN_ERR_FS_CLEANUP as _,
            fs_already_open: ffi::svn_errno_t_SVN_ERR_FS_ALREADY_OPEN as _,
            fs_not_open: ffi::svn_errno_t_SVN_ERR_FS_NOT_OPEN as _,
            fs_corrupt: ffi::svn_errno_t_SVN_ERR_FS_CORRUPT as _,
            fs_path_syntax: ffi::svn_errno_t_SVN_ERR_FS_PATH_SYNTAX as _,
            fs_no_such_revision: ffi::svn_errno_t_SVN_ERR_FS_NO_SUCH_REVISION as _,
            fs_no_such_transaction: ffi::svn_errno_t_SVN_ERR_FS_NO_SUCH_TRANSACTION as _,
            fs_no_such_entry: ffi::svn_errno_t_SVN_ERR_FS_NO_SUCH_ENTRY as _,
            fs_no_such_representation: ffi::svn_errno_t_SVN_ERR_FS_NO_SUCH_REPRESENTATION as _,
            fs_no_such_string: ffi::svn_errno_t_SVN_ERR_FS_NO_SUCH_STRING as _,
            fs_no_such_copy: ffi::svn_errno_t_SVN_ERR_FS_NO_SUCH_COPY as _,
            fs_transaction_not_mutable: ffi::svn_errno_t_SVN_ERR_FS_TRANSACTION_NOT_MUTABLE as _,
            fs_not_found: ffi::svn_errno_t_SVN_ERR_FS_NOT_FOUND as _,
            fs_id_not_found: ffi::svn_errno_t_SVN_ERR_FS_ID_NOT_FOUND as _,
            fs_not_id: ffi::svn_errno_t_SVN_ERR_FS_NOT_ID as _,
            fs_not_directory: ffi::svn_errno_t_SVN_ERR_FS_NOT_DIRECTORY as _,
            fs_not_file: ffi::svn_errno_t_SVN_ERR_FS_NOT_FILE as _,
            fs_not_single_path_component: ffi::svn_errno_t_SVN_ERR_FS_NOT_SINGLE_PATH_COMPONENT
                as _,
            fs_not_mutable: ffi::svn_errno_t_SVN_ERR_FS_NOT_MUTABLE as _,
            fs_already_exists: ffi::svn_errno_t_SVN_ERR_FS_ALREADY_EXISTS as _,
            fs_root_dir: ffi::svn_errno_t_SVN_ERR_FS_ROOT_DIR as _,
            fs_not_txn_root: ffi::svn_errno_t_SVN_ERR_FS_NOT_TXN_ROOT as _,
            fs_not_revision_root: ffi::svn_errno_t_SVN_ERR_FS_NOT_REVISION_ROOT as _,
            fs_conflict: ffi::svn_errno_t_SVN_ERR_FS_CONFLICT as _,
            fs_rep_changed: ffi::svn_errno_t_SVN_ERR_FS_REP_CHANGED as _,
            fs_rep_not_mutable: ffi::svn_errno_t_SVN_ERR_FS_REP_NOT_MUTABLE as _,
            fs_malformed_skel: ffi::svn_errno_t_SVN_ERR_FS_MALFORMED_SKEL as _,
            fs_txn_out_of_date: ffi::svn_errno_t_SVN_ERR_FS_TXN_OUT_OF_DATE as _,
            fs_berkeley_db: ffi::svn_errno_t_SVN_ERR_FS_BERKELEY_DB as _,
            fs_berkeley_db_deadlock: ffi::svn_errno_t_SVN_ERR_FS_BERKELEY_DB_DEADLOCK as _,
            fs_transaction_dead: ffi::svn_errno_t_SVN_ERR_FS_TRANSACTION_DEAD as _,
            fs_transaction_not_dead: ffi::svn_errno_t_SVN_ERR_FS_TRANSACTION_NOT_DEAD as _,
            fs_unknown_fs_type: ffi::svn_errno_t_SVN_ERR_FS_UNKNOWN_FS_TYPE as _,
            fs_no_user: ffi::svn_errno_t_SVN_ERR_FS_NO_USER as _,
            fs_path_already_locked: ffi::svn_errno_t_SVN_ERR_FS_PATH_ALREADY_LOCKED as _,
            fs_path_not_locked: ffi::svn_errno_t_SVN_ERR_FS_PATH_NOT_LOCKED as _,
            fs_bad_lock_token: ffi::svn_errno_t_SVN_ERR_FS_BAD_LOCK_TOKEN as _,
            fs_no_lock_token: ffi::svn_errno_t_SVN_ERR_FS_NO_LOCK_TOKEN as _,
            fs_lock_owner_mismatch: ffi::svn_errno_t_SVN_ERR_FS_LOCK_OWNER_MISMATCH as _,
            fs_no_such_lock: ffi::svn_errno_t_SVN_ERR_FS_NO_SUCH_LOCK as _,
            fs_lock_expired: ffi::svn_errno_t_SVN_ERR_FS_LOCK_EXPIRED as _,
            fs_out_of_date: ffi::svn_errno_t_SVN_ERR_FS_OUT_OF_DATE as _,
            fs_unsupported_format: ffi::svn_errno_t_SVN_ERR_FS_UNSUPPORTED_FORMAT as _,
            fs_rep_being_written: ffi::svn_errno_t_SVN_ERR_FS_REP_BEING_WRITTEN as _,
            fs_txn_name_too_long: ffi::svn_errno_t_SVN_ERR_FS_TXN_NAME_TOO_LONG as _,
            fs_no_such_node_origin: ffi::svn_errno_t_SVN_ERR_FS_NO_SUCH_NODE_ORIGIN as _,
            fs_unsupported_upgrade: ffi::svn_errno_t_SVN_ERR_FS_UNSUPPORTED_UPGRADE as _,
            fs_no_such_checksum_rep: ffi::svn_errno_t_SVN_ERR_FS_NO_SUCH_CHECKSUM_REP as _,
            fs_prop_basevalue_mismatch: ffi::svn_errno_t_SVN_ERR_FS_PROP_BASEVALUE_MISMATCH as _,
            fs_incorrect_editor_completion: ffi::svn_errno_t_SVN_ERR_FS_INCORRECT_EDITOR_COMPLETION
                as _,
            fs_packed_revprop_read_failure: ffi::svn_errno_t_SVN_ERR_FS_PACKED_REVPROP_READ_FAILURE
                as _,
            fs_revprop_cache_init_failure: ffi::svn_errno_t_SVN_ERR_FS_REVPROP_CACHE_INIT_FAILURE
                as _,
            fs_malformed_txn_id: ffi::svn_errno_t_SVN_ERR_FS_MALFORMED_TXN_ID as _,
            fs_index_corruption: ffi::svn_errno_t_SVN_ERR_FS_INDEX_CORRUPTION as _,
            fs_index_revision: ffi::svn_errno_t_SVN_ERR_FS_INDEX_REVISION as _,
            fs_index_overflow: ffi::svn_errno_t_SVN_ERR_FS_INDEX_OVERFLOW as _,
            fs_container_index: ffi::svn_errno_t_SVN_ERR_FS_CONTAINER_INDEX as _,
            fs_index_inconsistent: ffi::svn_errno_t_SVN_ERR_FS_INDEX_INCONSISTENT as _,
            fs_lock_operation_failed: ffi::svn_errno_t_SVN_ERR_FS_LOCK_OPERATION_FAILED as _,
            fs_unsupported_type: ffi::svn_errno_t_SVN_ERR_FS_UNSUPPORTED_TYPE as _,
            fs_container_size: ffi::svn_errno_t_SVN_ERR_FS_CONTAINER_SIZE as _,
            fs_malformed_noderev_id: ffi::svn_errno_t_SVN_ERR_FS_MALFORMED_NODEREV_ID as _,
            fs_invalid_generation: ffi::svn_errno_t_SVN_ERR_FS_INVALID_GENERATION as _,
            fs_corrupt_revprop_manifest: ffi::svn_errno_t_SVN_ERR_FS_CORRUPT_REVPROP_MANIFEST as _,
            fs_corrupt_proplist: ffi::svn_errno_t_SVN_ERR_FS_CORRUPT_PROPLIST as _,
            fs_ambiguous_checksum_rep: ffi::svn_errno_t_SVN_ERR_FS_AMBIGUOUS_CHECKSUM_REP as _,
            fs_unrecognized_ioctl_code: ffi::svn_errno_t_SVN_ERR_FS_UNRECOGNIZED_IOCTL_CODE as _,
            fs_rep_sharing_not_allowed: ffi::svn_errno_t_SVN_ERR_FS_REP_SHARING_NOT_ALLOWED as _,
            fs_rep_sharing_not_supported: ffi::svn_errno_t_SVN_ERR_FS_REP_SHARING_NOT_SUPPORTED
                as _,
            repos_locked: ffi::svn_errno_t_SVN_ERR_REPOS_LOCKED as _,
            repos_hook_failure: ffi::svn_errno_t_SVN_ERR_REPOS_HOOK_FAILURE as _,
            repos_bad_args: ffi::svn_errno_t_SVN_ERR_REPOS_BAD_ARGS as _,
            repos_no_data_for_report: ffi::svn_errno_t_SVN_ERR_REPOS_NO_DATA_FOR_REPORT as _,
            repos_bad_revision_report: ffi::svn_errno_t_SVN_ERR_REPOS_BAD_REVISION_REPORT as _,
            repos_unsupported_version: ffi::svn_errno_t_SVN_ERR_REPOS_UNSUPPORTED_VERSION as _,
            repos_disabled_feature: ffi::svn_errno_t_SVN_ERR_REPOS_DISABLED_FEATURE as _,
            repos_post_commit_hook_failed: ffi::svn_errno_t_SVN_ERR_REPOS_POST_COMMIT_HOOK_FAILED
                as _,
            repos_post_lock_hook_failed: ffi::svn_errno_t_SVN_ERR_REPOS_POST_LOCK_HOOK_FAILED as _,
            repos_post_unlock_hook_failed: ffi::svn_errno_t_SVN_ERR_REPOS_POST_UNLOCK_HOOK_FAILED
                as _,
            repos_unsupported_upgrade: ffi::svn_errno_t_SVN_ERR_REPOS_UNSUPPORTED_UPGRADE as _,
            ra_illegal_url: ffi::svn_errno_t_SVN_ERR_RA_ILLEGAL_URL as _,
            ra_not_authorized: ffi::svn_errno_t_SVN_ERR_RA_NOT_AUTHORIZED as _,
            ra_unknown_auth: ffi::svn_errno_t_SVN_ERR_RA_UNKNOWN_AUTH as _,
            ra_not_implemented: ffi::svn_errno_t_SVN_ERR_RA_NOT_IMPLEMENTED as _,
            ra_out_of_date: ffi::svn_errno_t_SVN_ERR_RA_OUT_OF_DATE as _,
            ra_no_repos_uuid: ffi::svn_errno_t_SVN_ERR_RA_NO_REPOS_UUID as _,
            ra_unsupported_abi_version: ffi::svn_errno_t_SVN_ERR_RA_UNSUPPORTED_ABI_VERSION as _,
            ra_not_locked: ffi::svn_errno_t_SVN_ERR_RA_NOT_LOCKED as _,
            ra_partial_replay_not_supported:
                ffi::svn_errno_t_SVN_ERR_RA_PARTIAL_REPLAY_NOT_SUPPORTED as _,
            ra_uuid_mismatch: ffi::svn_errno_t_SVN_ERR_RA_UUID_MISMATCH as _,
            ra_repos_root_url_mismatch: ffi::svn_errno_t_SVN_ERR_RA_REPOS_ROOT_URL_MISMATCH as _,
            ra_session_url_mismatch: ffi::svn_errno_t_SVN_ERR_RA_SESSION_URL_MISMATCH as _,
            ra_cannot_create_tunnel: ffi::svn_errno_t_SVN_ERR_RA_CANNOT_CREATE_TUNNEL as _,
            ra_cannot_create_session: ffi::svn_errno_t_SVN_ERR_RA_CANNOT_CREATE_SESSION as _,
            ra_dav_sock_init: ffi::svn_errno_t_SVN_ERR_RA_DAV_SOCK_INIT as _,
            ra_dav_creating_request: ffi::svn_errno_t_SVN_ERR_RA_DAV_CREATING_REQUEST as _,
            ra_dav_request_failed: ffi::svn_errno_t_SVN_ERR_RA_DAV_REQUEST_FAILED as _,
            ra_dav_options_req_failed: ffi::svn_errno_t_SVN_ERR_RA_DAV_OPTIONS_REQ_FAILED as _,
            ra_dav_props_not_found: ffi::svn_errno_t_SVN_ERR_RA_DAV_PROPS_NOT_FOUND as _,
            ra_dav_already_exists: ffi::svn_errno_t_SVN_ERR_RA_DAV_ALREADY_EXISTS as _,
            ra_dav_invalid_config_value: ffi::svn_errno_t_SVN_ERR_RA_DAV_INVALID_CONFIG_VALUE as _,
            ra_dav_path_not_found: ffi::svn_errno_t_SVN_ERR_RA_DAV_PATH_NOT_FOUND as _,
            ra_dav_proppatch_failed: ffi::svn_errno_t_SVN_ERR_RA_DAV_PROPPATCH_FAILED as _,
            ra_dav_malformed_data: ffi::svn_errno_t_SVN_ERR_RA_DAV_MALFORMED_DATA as _,
            ra_dav_response_header_badness: ffi::svn_errno_t_SVN_ERR_RA_DAV_RESPONSE_HEADER_BADNESS
                as _,
            ra_dav_relocated: ffi::svn_errno_t_SVN_ERR_RA_DAV_RELOCATED as _,
            ra_dav_conn_timeout: ffi::svn_errno_t_SVN_ERR_RA_DAV_CONN_TIMEOUT as _,
            ra_dav_forbidden: ffi::svn_errno_t_SVN_ERR_RA_DAV_FORBIDDEN as _,
            ra_dav_precondition_failed: ffi::svn_errno_t_SVN_ERR_RA_DAV_PRECONDITION_FAILED as _,
            ra_dav_method_not_allowed: ffi::svn_errno_t_SVN_ERR_RA_DAV_METHOD_NOT_ALLOWED as _,
            ra_local_repos_not_found: ffi::svn_errno_t_SVN_ERR_RA_LOCAL_REPOS_NOT_FOUND as _,
            ra_local_repos_open_failed: ffi::svn_errno_t_SVN_ERR_RA_LOCAL_REPOS_OPEN_FAILED as _,
            svndiff_invalid_header: ffi::svn_errno_t_SVN_ERR_SVNDIFF_INVALID_HEADER as _,
            svndiff_corrupt_window: ffi::svn_errno_t_SVN_ERR_SVNDIFF_CORRUPT_WINDOW as _,
            svndiff_backward_view: ffi::svn_errno_t_SVN_ERR_SVNDIFF_BACKWARD_VIEW as _,
            svndiff_invalid_ops: ffi::svn_errno_t_SVN_ERR_SVNDIFF_INVALID_OPS as _,
            svndiff_unexpected_end: ffi::svn_errno_t_SVN_ERR_SVNDIFF_UNEXPECTED_END as _,
            svndiff_invalid_compressed_data:
                ffi::svn_errno_t_SVN_ERR_SVNDIFF_INVALID_COMPRESSED_DATA as _,
            apmod_missing_path_to_fs: ffi::svn_errno_t_SVN_ERR_APMOD_MISSING_PATH_TO_FS as _,
            apmod_malformed_uri: ffi::svn_errno_t_SVN_ERR_APMOD_MALFORMED_URI as _,
            apmod_activity_not_found: ffi::svn_errno_t_SVN_ERR_APMOD_ACTIVITY_NOT_FOUND as _,
            apmod_bad_baseline: ffi::svn_errno_t_SVN_ERR_APMOD_BAD_BASELINE as _,
            apmod_connection_aborted: ffi::svn_errno_t_SVN_ERR_APMOD_CONNECTION_ABORTED as _,
            client_versioned_path_required: ffi::svn_errno_t_SVN_ERR_CLIENT_VERSIONED_PATH_REQUIRED
                as _,
            client_ra_access_required: ffi::svn_errno_t_SVN_ERR_CLIENT_RA_ACCESS_REQUIRED as _,
            client_bad_revision: ffi::svn_errno_t_SVN_ERR_CLIENT_BAD_REVISION as _,
            client_duplicate_commit_url: ffi::svn_errno_t_SVN_ERR_CLIENT_DUPLICATE_COMMIT_URL as _,
            client_is_binary_file: ffi::svn_errno_t_SVN_ERR_CLIENT_IS_BINARY_FILE as _,
            client_invalid_externals_description:
                ffi::svn_errno_t_SVN_ERR_CLIENT_INVALID_EXTERNALS_DESCRIPTION as _,
            client_modified: ffi::svn_errno_t_SVN_ERR_CLIENT_MODIFIED as _,
            client_is_directory: ffi::svn_errno_t_SVN_ERR_CLIENT_IS_DIRECTORY as _,
            client_revision_range: ffi::svn_errno_t_SVN_ERR_CLIENT_REVISION_RANGE as _,
            client_invalid_relocation: ffi::svn_errno_t_SVN_ERR_CLIENT_INVALID_RELOCATION as _,
            client_revision_author_contains_newline:
                ffi::svn_errno_t_SVN_ERR_CLIENT_REVISION_AUTHOR_CONTAINS_NEWLINE as _,
            client_property_name: ffi::svn_errno_t_SVN_ERR_CLIENT_PROPERTY_NAME as _,
            client_unrelated_resources: ffi::svn_errno_t_SVN_ERR_CLIENT_UNRELATED_RESOURCES as _,
            client_missing_lock_token: ffi::svn_errno_t_SVN_ERR_CLIENT_MISSING_LOCK_TOKEN as _,
            client_multiple_sources_disallowed:
                ffi::svn_errno_t_SVN_ERR_CLIENT_MULTIPLE_SOURCES_DISALLOWED as _,
            client_no_versioned_parent: ffi::svn_errno_t_SVN_ERR_CLIENT_NO_VERSIONED_PARENT as _,
            client_not_ready_to_merge: ffi::svn_errno_t_SVN_ERR_CLIENT_NOT_READY_TO_MERGE as _,
            client_file_external_overwrite_versioned:
                ffi::svn_errno_t_SVN_ERR_CLIENT_FILE_EXTERNAL_OVERWRITE_VERSIONED as _,
            client_patch_bad_strip_count: ffi::svn_errno_t_SVN_ERR_CLIENT_PATCH_BAD_STRIP_COUNT
                as _,
            client_cycle_detected: ffi::svn_errno_t_SVN_ERR_CLIENT_CYCLE_DETECTED as _,
            client_merge_update_required: ffi::svn_errno_t_SVN_ERR_CLIENT_MERGE_UPDATE_REQUIRED
                as _,
            client_invalid_mergeinfo_no_mergetracking:
                ffi::svn_errno_t_SVN_ERR_CLIENT_INVALID_MERGEINFO_NO_MERGETRACKING as _,
            client_no_lock_token: ffi::svn_errno_t_SVN_ERR_CLIENT_NO_LOCK_TOKEN as _,
            client_forbidden_by_server: ffi::svn_errno_t_SVN_ERR_CLIENT_FORBIDDEN_BY_SERVER as _,
            client_conflict_option_not_applicable:
                ffi::svn_errno_t_SVN_ERR_CLIENT_CONFLICT_OPTION_NOT_APPLICABLE as _,
            base: ffi::svn_errno_t_SVN_ERR_BASE as _,
            plugin_load_failure: ffi::svn_errno_t_SVN_ERR_PLUGIN_LOAD_FAILURE as _,
            malformed_file: ffi::svn_errno_t_SVN_ERR_MALFORMED_FILE as _,
            incomplete_data: ffi::svn_errno_t_SVN_ERR_INCOMPLETE_DATA as _,
            incorrect_params: ffi::svn_errno_t_SVN_ERR_INCORRECT_PARAMS as _,
            unversioned_resource: ffi::svn_errno_t_SVN_ERR_UNVERSIONED_RESOURCE as _,
            test_failed: ffi::svn_errno_t_SVN_ERR_TEST_FAILED as _,
            unsupported_feature: ffi::svn_errno_t_SVN_ERR_UNSUPPORTED_FEATURE as _,
            bad_prop_kind: ffi::svn_errno_t_SVN_ERR_BAD_PROP_KIND as _,
            illegal_target: ffi::svn_errno_t_SVN_ERR_ILLEGAL_TARGET as _,
            delta_md5_checksum_absent: ffi::svn_errno_t_SVN_ERR_DELTA_MD5_CHECKSUM_ABSENT as _,
            dir_not_empty: ffi::svn_errno_t_SVN_ERR_DIR_NOT_EMPTY as _,
            external_program: ffi::svn_errno_t_SVN_ERR_EXTERNAL_PROGRAM as _,
            swig_py_exception_set: ffi::svn_errno_t_SVN_ERR_SWIG_PY_EXCEPTION_SET as _,
            checksum_mismatch: ffi::svn_errno_t_SVN_ERR_CHECKSUM_MISMATCH as _,
            cancelled: ffi::svn_errno_t_SVN_ERR_CANCELLED as _,
            invalid_diff_option: ffi::svn_errno_t_SVN_ERR_INVALID_DIFF_OPTION as _,
            property_not_found: ffi::svn_errno_t_SVN_ERR_PROPERTY_NOT_FOUND as _,
            no_auth_file_path: ffi::svn_errno_t_SVN_ERR_NO_AUTH_FILE_PATH as _,
            version_mismatch: ffi::svn_errno_t_SVN_ERR_VERSION_MISMATCH as _,
            mergeinfo_parse_error: ffi::svn_errno_t_SVN_ERR_MERGEINFO_PARSE_ERROR as _,
            cease_invocation: ffi::svn_errno_t_SVN_ERR_CEASE_INVOCATION as _,
            revnum_parse_failure: ffi::svn_errno_t_SVN_ERR_REVNUM_PARSE_FAILURE as _,
            iter_break: ffi::svn_errno_t_SVN_ERR_ITER_BREAK as _,
            unknown_changelist: ffi::svn_errno_t_SVN_ERR_UNKNOWN_CHANGELIST as _,
            reserved_filename_specified: ffi::svn_errno_t_SVN_ERR_RESERVED_FILENAME_SPECIFIED as _,
            unknown_capability: ffi::svn_errno_t_SVN_ERR_UNKNOWN_CAPABILITY as _,
            test_skipped: ffi::svn_errno_t_SVN_ERR_TEST_SKIPPED as _,
            no_apr_memcache: ffi::svn_errno_t_SVN_ERR_NO_APR_MEMCACHE as _,
            atomic_init_failure: ffi::svn_errno_t_SVN_ERR_ATOMIC_INIT_FAILURE as _,
            sqlite_error: ffi::svn_errno_t_SVN_ERR_SQLITE_ERROR as _,
            sqlite_readonly: ffi::svn_errno_t_SVN_ERR_SQLITE_READONLY as _,
            sqlite_unsupported_schema: ffi::svn_errno_t_SVN_ERR_SQLITE_UNSUPPORTED_SCHEMA as _,
            sqlite_busy: ffi::svn_errno_t_SVN_ERR_SQLITE_BUSY as _,
            sqlite_resetting_for_rollback: ffi::svn_errno_t_SVN_ERR_SQLITE_RESETTING_FOR_ROLLBACK
                as _,
            sqlite_constraint: ffi::svn_errno_t_SVN_ERR_SQLITE_CONSTRAINT as _,
            too_many_memcached_servers: ffi::svn_errno_t_SVN_ERR_TOO_MANY_MEMCACHED_SERVERS as _,
            malformed_version_string: ffi::svn_errno_t_SVN_ERR_MALFORMED_VERSION_STRING as _,
            corrupted_atomic_storage: ffi::svn_errno_t_SVN_ERR_CORRUPTED_ATOMIC_STORAGE as _,
            utf8proc_error: ffi::svn_errno_t_SVN_ERR_UTF8PROC_ERROR as _,
            utf8_glob: ffi::svn_errno_t_SVN_ERR_UTF8_GLOB as _,
            corrupt_packed_data: ffi::svn_errno_t_SVN_ERR_CORRUPT_PACKED_DATA as _,
            composed_error: ffi::svn_errno_t_SVN_ERR_COMPOSED_ERROR as _,
            invalid_input: ffi::svn_errno_t_SVN_ERR_INVALID_INPUT as _,
            sqlite_rollback_failed: ffi::svn_errno_t_SVN_ERR_SQLITE_ROLLBACK_FAILED as _,
            lz4_compression_failed: ffi::svn_errno_t_SVN_ERR_LZ4_COMPRESSION_FAILED as _,
            lz4_decompression_failed: ffi::svn_errno_t_SVN_ERR_LZ4_DECOMPRESSION_FAILED as _,
            canonicalization_failed: ffi::svn_errno_t_SVN_ERR_CANONICALIZATION_FAILED as _,
            cl_arg_parsing_error: ffi::svn_errno_t_SVN_ERR_CL_ARG_PARSING_ERROR as _,
            cl_insufficient_args: ffi::svn_errno_t_SVN_ERR_CL_INSUFFICIENT_ARGS as _,
            cl_mutually_exclusive_args: ffi::svn_errno_t_SVN_ERR_CL_MUTUALLY_EXCLUSIVE_ARGS as _,
            cl_adm_dir_reserved: ffi::svn_errno_t_SVN_ERR_CL_ADM_DIR_RESERVED as _,
            cl_log_message_is_versioned_file:
                ffi::svn_errno_t_SVN_ERR_CL_LOG_MESSAGE_IS_VERSIONED_FILE as _,
            cl_log_message_is_pathname: ffi::svn_errno_t_SVN_ERR_CL_LOG_MESSAGE_IS_PATHNAME as _,
            cl_commit_in_added_dir: ffi::svn_errno_t_SVN_ERR_CL_COMMIT_IN_ADDED_DIR as _,
            cl_no_external_editor: ffi::svn_errno_t_SVN_ERR_CL_NO_EXTERNAL_EDITOR as _,
            cl_bad_log_message: ffi::svn_errno_t_SVN_ERR_CL_BAD_LOG_MESSAGE as _,
            cl_unnecessary_log_message: ffi::svn_errno_t_SVN_ERR_CL_UNNECESSARY_LOG_MESSAGE as _,
            cl_no_external_merge_tool: ffi::svn_errno_t_SVN_ERR_CL_NO_EXTERNAL_MERGE_TOOL as _,
            cl_error_processing_externals: ffi::svn_errno_t_SVN_ERR_CL_ERROR_PROCESSING_EXTERNALS
                as _,
            cl_repos_verify_failed: ffi::svn_errno_t_SVN_ERR_CL_REPOS_VERIFY_FAILED as _,
            ra_svn_cmd_err: ffi::svn_errno_t_SVN_ERR_RA_SVN_CMD_ERR as _,
            ra_svn_unknown_cmd: ffi::svn_errno_t_SVN_ERR_RA_SVN_UNKNOWN_CMD as _,
            ra_svn_connection_closed: ffi::svn_errno_t_SVN_ERR_RA_SVN_CONNECTION_CLOSED as _,
            ra_svn_io_error: ffi::svn_errno_t_SVN_ERR_RA_SVN_IO_ERROR as _,
            ra_svn_malformed_data: ffi::svn_errno_t_SVN_ERR_RA_SVN_MALFORMED_DATA as _,
            ra_svn_repos_not_found: ffi::svn_errno_t_SVN_ERR_RA_SVN_REPOS_NOT_FOUND as _,
            ra_svn_bad_version: ffi::svn_errno_t_SVN_ERR_RA_SVN_BAD_VERSION as _,
            ra_svn_no_mechanisms: ffi::svn_errno_t_SVN_ERR_RA_SVN_NO_MECHANISMS as _,
            ra_svn_edit_aborted: ffi::svn_errno_t_SVN_ERR_RA_SVN_EDIT_ABORTED as _,
            ra_svn_request_size: ffi::svn_errno_t_SVN_ERR_RA_SVN_REQUEST_SIZE as _,
            ra_svn_response_size: ffi::svn_errno_t_SVN_ERR_RA_SVN_RESPONSE_SIZE as _,
            authn_creds_unavailable: ffi::svn_errno_t_SVN_ERR_AUTHN_CREDS_UNAVAILABLE as _,
            authn_no_provider: ffi::svn_errno_t_SVN_ERR_AUTHN_NO_PROVIDER as _,
            authn_providers_exhausted: ffi::svn_errno_t_SVN_ERR_AUTHN_PROVIDERS_EXHAUSTED as _,
            authn_creds_not_saved: ffi::svn_errno_t_SVN_ERR_AUTHN_CREDS_NOT_SAVED as _,
            authn_failed: ffi::svn_errno_t_SVN_ERR_AUTHN_FAILED as _,
            authz_root_unreadable: ffi::svn_errno_t_SVN_ERR_AUTHZ_ROOT_UNREADABLE as _,
            authz_unreadable: ffi::svn_errno_t_SVN_ERR_AUTHZ_UNREADABLE as _,
            authz_partially_readable: ffi::svn_errno_t_SVN_ERR_AUTHZ_PARTIALLY_READABLE as _,
            authz_invalid_config: ffi::svn_errno_t_SVN_ERR_AUTHZ_INVALID_CONFIG as _,
            authz_unwritable: ffi::svn_errno_t_SVN_ERR_AUTHZ_UNWRITABLE as _,
            diff_datasource_modified: ffi::svn_errno_t_SVN_ERR_DIFF_DATASOURCE_MODIFIED as _,
            diff_unexpected_data: ffi::svn_errno_t_SVN_ERR_DIFF_UNEXPECTED_DATA as _,
            ra_serf_sspi_initialisation_failed:
                ffi::svn_errno_t_SVN_ERR_RA_SERF_SSPI_INITIALISATION_FAILED as _,
            ra_serf_ssl_cert_untrusted: ffi::svn_errno_t_SVN_ERR_RA_SERF_SSL_CERT_UNTRUSTED as _,
            ra_serf_gssapi_initialisation_failed:
                ffi::svn_errno_t_SVN_ERR_RA_SERF_GSSAPI_INITIALISATION_FAILED as _,
            ra_serf_wrapped_error: ffi::svn_errno_t_SVN_ERR_RA_SERF_WRAPPED_ERROR as _,
            ra_serf_stream_bucket_read_error:
                ffi::svn_errno_t_SVN_ERR_RA_SERF_STREAM_BUCKET_READ_ERROR as _,
            assertion_fail: ffi::svn_errno_t_SVN_ERR_ASSERTION_FAIL as _,
            assertion_only_tracing_links: ffi::svn_errno_t_SVN_ERR_ASSERTION_ONLY_TRACING_LINKS
                as _,
            asn1_out_of_data: ffi::svn_errno_t_SVN_ERR_ASN1_OUT_OF_DATA as _,
            asn1_unexpected_tag: ffi::svn_errno_t_SVN_ERR_ASN1_UNEXPECTED_TAG as _,
            asn1_invalid_length: ffi::svn_errno_t_SVN_ERR_ASN1_INVALID_LENGTH as _,
            asn1_length_mismatch: ffi::svn_errno_t_SVN_ERR_ASN1_LENGTH_MISMATCH as _,
            asn1_invalid_data: ffi::svn_errno_t_SVN_ERR_ASN1_INVALID_DATA as _,
            x509_feature_unavailable: ffi::svn_errno_t_SVN_ERR_X509_FEATURE_UNAVAILABLE as _,
            x509_cert_invalid_pem: ffi::svn_errno_t_SVN_ERR_X509_CERT_INVALID_PEM as _,
            x509_cert_invalid_format: ffi::svn_errno_t_SVN_ERR_X509_CERT_INVALID_FORMAT as _,
            x509_cert_invalid_version: ffi::svn_errno_t_SVN_ERR_X509_CERT_INVALID_VERSION as _,
            x509_cert_invalid_serial: ffi::svn_errno_t_SVN_ERR_X509_CERT_INVALID_SERIAL as _,
            x509_cert_invalid_alg: ffi::svn_errno_t_SVN_ERR_X509_CERT_INVALID_ALG as _,
            x509_cert_invalid_name: ffi::svn_errno_t_SVN_ERR_X509_CERT_INVALID_NAME as _,
            x509_cert_invalid_date: ffi::svn_errno_t_SVN_ERR_X509_CERT_INVALID_DATE as _,
            x509_cert_invalid_pubkey: ffi::svn_errno_t_SVN_ERR_X509_CERT_INVALID_PUBKEY as _,
            x509_cert_invalid_signature: ffi::svn_errno_t_SVN_ERR_X509_CERT_INVALID_SIGNATURE as _,
            x509_cert_invalid_extensions: ffi::svn_errno_t_SVN_ERR_X509_CERT_INVALID_EXTENSIONS
                as _,
            x509_cert_unknown_version: ffi::svn_errno_t_SVN_ERR_X509_CERT_UNKNOWN_VERSION as _,
            x509_cert_unknown_pk_alg: ffi::svn_errno_t_SVN_ERR_X509_CERT_UNKNOWN_PK_ALG as _,
            x509_cert_sig_mismatch: ffi::svn_errno_t_SVN_ERR_X509_CERT_SIG_MISMATCH as _,
            x509_cert_verify_failed: ffi::svn_errno_t_SVN_ERR_X509_CERT_VERIFY_FAILED as _,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record, Serialize, Deserialize)]
pub struct ErrorInfo {
    code: i32,
    msg: String,
    file: String,
    line: i32,
}

#[derive(Debug, Clone, uniffi::Record, Serialize, Deserialize)]
pub struct SVNError {
    pub msg: String,
    pub code: i32,
    pub info: Vec<ErrorInfo>,
}

impl std::error::Error for SVNError {}

impl std::fmt::Display for SVNError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", self.msg, self.code)
    }
}

impl SVNError {
    fn to_error(&self) -> *mut ffi::svn_error_t {
        unsafe {
            let mut pool = apr::Pool::create();
            let msg = pool.string(self.msg.as_str()).unwrap_or_default();
            ffi::svn_error_create(self.code, Default::default(), msg as _)
        }
    }

    fn from_nullable_ptr(err: *mut ffi::svn_error_t) -> Result<(), Self> {
        if err.is_null() {
            Ok(())
        } else {
            let result = Err(Self::from_not_null_ptr(err));

            unsafe {
                ffi::svn_error_clear(err);
            }

            result
        }
    }

    fn from_not_null_ptr(err: *const ffi::svn_error_t) -> Self {
        unsafe {
            assert!(!err.is_null());

            let mut buf = vec![0; 512];

            let msg = ffi::svn_err_best_message(err, buf.as_mut_ptr(), buf.len() as _);

            let msg = msg.to_str().to_string();

            let code = err.as_ref().unwrap().apr_err as i32;

            // let err = unsafe { &*err };

            let mut next = err;

            let mut info = vec![];

            while !next.is_null() {
                let err = next.as_ref().unwrap();

                let msg = err.message.to_nullable_string().unwrap_or_default();

                let file = err.file.to_nullable_string().unwrap_or_default();
                let error_info = ErrorInfo {
                    code: err.apr_err as _,
                    msg,
                    file,
                    line: err.line as _,
                };
                info.push(error_info);

                next = err.child;
            }

            Self { msg, code, info }
        }
    }
}

const fn svn_no_error() -> *mut ffi::svn_error_t {
    ffi::SVN_NO_ERROR as *mut _
}

pub fn encode_base64(bytes: &[u8], break_lines: bool) -> String {
    let mut stream = stream::Stream::create(Default::default());

    let mut input = stream.base64(break_lines);

    input.write(bytes).unwrap();

    input.close().unwrap();

    String::from_utf8(stream.take_write_buffer()).unwrap_or_default()

    // unsafe {
    //     let mut pool = apr::Pool::create();
    //     let string = ffi::svn_string_create(
    //         CString::from_str(string).unwrap().as_ptr(),
    //         pool.as_mut_ptr(),
    //     );

    //     let base64 = ffi::svn_base64_encode_string2(string, 1, pool.as_mut_ptr())
    //         .as_ref()
    //         .unwrap();
    //     std::slice::from_raw_parts(base64.data as *const u8, base64.len.try_into().unwrap())
    //         .to_vec()
    // }
}
