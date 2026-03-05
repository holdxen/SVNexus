mod apr;
mod error;
mod subversion;
mod tests;
mod utils;

uniffi::setup_scaffolding!();


#[uniffi::export]
fn engine_initialize() {
    tracing_subscriber::fmt().with_target(false).init();
    tracing::info!("Engine is initialized");
}
