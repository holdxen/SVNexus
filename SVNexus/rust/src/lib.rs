use std::{any::Any, backtrace::Backtrace};

use crate::db::{WorkspaceHistory, WorkspaceHistoryGroup};
use std::panic;
use tracing_appender::non_blocking::WorkerGuard;

mod app;
mod apr;
mod db;
mod entities;
mod error;
mod extensions;
mod subversion;
mod tests;
mod utils;

uniffi::setup_scaffolding!();

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

#[uniffi::export]
fn engine_initialize() {
    use tracing_subscriber::fmt::format::FmtSpan;
    use tracing_subscriber::{EnvFilter, fmt, prelude::*};

    let env_filter = EnvFilter::from_default_env();

    let project = app::project().expect("Failed to detect project directory");

    let file_appender = tracing_appender::rolling::daily(project.log_directory(), "svnexus.log");

    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);

    *GUARD.lock() = Some(guard);

    let file_layer = fmt::layer()
        .with_writer(file_writer)
        .with_file(true) // 全局开启行号+文件名
        .with_line_number(true)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .with_ansi(false);

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

#[derive(uniffi::Object)]
pub struct AnyValue(Box<dyn Any + Send + Sync + 'static>);

impl AnyValue {
    fn new(value: impl Any + Send + Sync + 'static) -> Self {
        Self(Box::new(value))
    }
}

#[uniffi::export]
impl AnyValue {
    #[uniffi::constructor]
    fn from_workspace_history_group(group: WorkspaceHistoryGroup) -> Self {
        Self::new(group)
    }

    #[uniffi::constructor]
    fn from_workspace_history(history: WorkspaceHistory) -> Self {
        Self::new(history)
    }

    fn to_workspace_history(&self) -> Option<WorkspaceHistory> {
        if let Some(value) = self.0.downcast_ref::<WorkspaceHistory>() {
            Some(value.clone())
        } else {
            None
        }
    }

    fn to_workspace_history_group(&self) -> Option<WorkspaceHistoryGroup> {
        if let Some(value) = self.0.downcast_ref::<WorkspaceHistoryGroup>() {
            Some(value.clone())
        } else {
            None
        }
    }
}
