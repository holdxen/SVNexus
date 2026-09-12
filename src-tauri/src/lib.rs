use std::backtrace::Backtrace;

use std::panic;
use tracing_appender::non_blocking::WorkerGuard;

use crate::extensions::CommonExtension;

mod app;
mod apr;
mod backend;
mod db;
mod entities;
mod error;
mod extensions;
pub mod messagepack_command;
mod platform;
mod subversion;
mod tests;
mod utils;

static GUARD: parking_lot::Mutex<Option<WorkerGuard>> = parking_lot::Mutex::new(None);

fn setup_panic_hook() {
    let default = panic::take_hook();

    panic::set_hook(Box::new(move |info| {
        let payload = info.payload();

        let message = if let Some(s) = payload.downcast_ref::<&str>() {
            *s
        } else if let Some(s) = payload.downcast_ref::<String>() {
            s.as_str()
        } else {
            "<non-string panic payload>"
        };

        let location = info
            .location()
            .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()))
            .unwrap_or_else(|| "<unknown location>".to_string());

        let backtrace = Backtrace::force_capture();

        tracing::error!("========== PANIC ==========");
        tracing::error!("message: {}", message);
        tracing::error!("location: {}", location);
        tracing::error!("backtrace:\n{}\n", backtrace);

        *GUARD.lock() = None;

        default(info)
    }));
}

fn initialize() {
    use tracing_subscriber::field::RecordFields;
    use tracing_subscriber::fmt::format::{DefaultFields, FmtSpan, Writer};
    use tracing_subscriber::fmt::FormatFields;
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};

    // 文件层专用的字段格式化器:行为完全等同 DefaultFields,只是「类型不同」。
    // tracing 把每个 span 的字段格式化结果按【格式化器类型 N】缓存进 span extensions
    // (FormattedFields<N>;见 fmt_layer.rs on_new_span:仅当 is_none() 时才格式化、否则复用)。
    // 若 stdout 层与文件层都用 DefaultFields,就会共用同一缓存槽:先派发的 stdout 层
    // (inner-first,ansi=true)把带 ANSI 的 span 字段写进缓存,文件层(ansi=false)直接复用,
    // 于是颜色转义码漏进日志文件(事件头的时间戳/级别/消息由各层各自格式化,所以不乱)。
    // 用这个独立类型让文件层拥有自己的 FormattedFields<FileFields> 缓存槽,两层互不干扰;
    // 是否带 ANSI 由传入的 Writer(各层的 is_ansi)决定,本类型只做转发。
    #[derive(Debug)]
    struct FileFields(DefaultFields);

    impl<'writer> FormatFields<'writer> for FileFields {
        fn format_fields<R: RecordFields>(
            &self,
            writer: Writer<'writer>,
            fields: R,
        ) -> std::fmt::Result {
            self.0.format_fields(writer, fields)
        }
    }

    // 默认只对 svnexus、webview 两个 target 放行 INFO(RUST_LOG 未设置时),其它 target 默认 OFF,
    // 等价于 `RUST_LOG=svnexus=info,webview=info`。设置了 RUST_LOG 则完全以它为准
    // (EnvFilter::new 内部 lossy 解析,无效指令被忽略;解析为空时才回退到内置 ERROR 默认)。
    let directives =
        std::env::var("RUST_LOG").unwrap_or_else(|_| "svnexus=info,webview=info".to_string());
    let env_filter = EnvFilter::new(directives);

    let project = app::project().expect("Failed to detect project directory");

    let file_appender = tracing_appender::rolling::daily(project.log_directory(), "svnexus.log");

    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);

    *GUARD.lock() = guard.into_option_some();

    let file_layer = fmt::layer()
        .with_writer(file_writer)
        .with_file(true) // 全局开启行号+文件名
        .with_line_number(true)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .with_ansi(false)
        // 用独立格式化器类型,使文件层拥有自己的 FormattedFields 缓存槽,
        // 不与 stdout 层(ansi=true)共用,从而避免 ANSI 转义码被写进日志文件。
        .fmt_fields(FileFields(DefaultFields::new()));

    let layer = fmt::layer()
        .with_file(true) // 全局开启行号+文件名
        .with_line_number(true)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .with_ansi(true);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(layer)
        .with(file_layer)
        .init();

    tracing::info!("Logging in {}", project.log_directory().display());

    setup_panic_hook();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    std::env::set_var("GTK_OVERLAY_SCROLLING", "0"); // ← 加这行
    initialize();
    tauri::Builder::default()
        .setup(|app| {
            // Windows: JSON 的 /**/* 无法匹配盘符路径，需通过 Rust API 遍历盘符
            #[cfg(windows)]
            {
                use tauri_plugin_fs::FsExt;
                let scope = app.fs_scope();
                for drive_letter in b'A'..=b'Z' {
                    let drive_path = format!("{}:\\", drive_letter as char);
                    let _ = scope.allow_directory(&drive_path, true);
                }
            }

            let _ = app;

            Ok(())
        })
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            backend::reload,
            backend::reply,
            // subversion
            backend::subversion_cancel,
            backend::subversion_add,
            backend::subversion_blame,
            backend::subversion_cat,
            backend::subversion_checkout,
            backend::subversion_cleanup,
            backend::subversion_commit,
            backend::subversion_conflict_walk,
            backend::subversion_copy,
            backend::subversion_create,
            backend::subversion_default_wc_version,
            backend::subversion_delete,
            backend::subversion_destroy,
            backend::subversion_difference,
            backend::subversion_export,
            backend::subversion_get_wc_root,
            backend::subversion_import,
            backend::subversion_info,
            backend::subversion_list,
            backend::subversion_lock,
            backend::subversion_log,
            backend::subversion_log_next,
            backend::subversion_merge,
            backend::subversion_mkdir,
            backend::subversion_move,
            backend::subversion_patch,
            backend::subversion_property_get,
            backend::subversion_property_list,
            backend::subversion_property_set,
            backend::subversion_relocate,
            backend::subversion_revert,
            backend::subversion_revision_property_list,
            backend::subversion_status,
            backend::subversion_switch,
            backend::subversion_unlock,
            backend::subversion_update,
            backend::subversion_upgrade,
            backend::subversion_url_from_path,
            backend::subversion_vacuum,
            backend::subversion_log_cache,
            backend::subversion_log_cache_reverse,
            // subversion ra
            backend::subversion_ra_get_locations,
            backend::subversion_ra_get_latest_revision_number,
            // subversion wc
            backend::subversion_wc_revision_status,
            backend::subversion_wc_get_replaced_file,
            // database subversion
            backend::database_revision_location,
            backend::database_update_revision_location,
            // path
            backend::path_start_with,
            backend::path_combine,
            backend::path_get_parent,
            backend::path_get_file_name,
            backend::path_strip_prefix,
            backend::path_into_parts,
            // fs
            backend::fs_read_link,
            // workspace group
            backend::database_add_workspace_group,
            backend::database_delete_workspace_group,
            backend::database_update_workspace_group,
            backend::database_workspace_groups,
            backend::database_update_repository_log,
            // workspace item
            backend::database_workspace_items,
            backend::database_add_workspace_item,
            backend::database_delete_workspace_item,
            // utils
            backend::format_size,
            // utils base64
            backend::base64_decode,
            backend::base64_encode,
            // log
            backend::log_info,
            backend::log_trace,
            backend::log_debug,
            backend::log_error,
            backend::log_warn,
            // version
            backend::extended_version
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
