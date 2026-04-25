use std::any::Any;

use crate::db::{WorkspaceHistory, WorkspaceHistoryGroup};

mod apr;
mod db;
mod entities;
mod error;
mod extensions;
mod subversion;
mod tests;
mod utils;

uniffi::setup_scaffolding!();

#[uniffi::export]
fn engine_initialize() {
    use tracing_subscriber::fmt::format::FmtSpan;
    use tracing_subscriber::{EnvFilter, fmt, prelude::*};

    let env_filter = EnvFilter::from_default_env();

    let layer = fmt::layer()
        .with_file(true) // 全局开启行号+文件名
        .with_line_number(true)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .with_ansi(true);

    // let normal_layer = fmt::layer()
    //     .with_file(true) // 全局开启行号+文件名
    //     .with_line_number(true)
    //     .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
    //     // ↓ 把你原来的所有其他配置（颜色、时间、target、pretty 等）全部复制到这里
    //     .with_ansi(true)
    //     // .with_timer(...)           // 你原来的 timer 配置
    //     // ... 其他配置保持一致
    //     .with_filter(filter_fn(|metadata| {
    //         let is_event = metadata.fields().field("message").is_some(); // 可靠判断：只有 event 才有 message 字段
    //         if is_event {
    //             metadata.target() != "uniffi"
    //         } else {
    //             true
    //         }
    //     }));

    // let no_location_layer = fmt::layer()
    //     .with_span_events(FmtSpan::NONE)
    //     .with_file(true) // 这条 layer 不显示文件名和行号
    //     .with_line_number(true)
    //     // ↓ 其他配置必须和上面完全一致（颜色、时间等），防止格式差异
    //     .with_ansi(true)
    //     // .with_timer(...)           // 同上
    //     // ...
    //     .with_filter(filter_fn(|metadata| {
    //         let is_event = metadata.fields().field("message").is_some(); // 可靠判断：只有 event 才有 message 字段
    //         if is_event {
    //             metadata.target() == "uniffi"
    //         } else {
    //             true // 所有 span 都放行，让它知道当前 span 上下文
    //         }
    //     }));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(layer)
        .init();
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
