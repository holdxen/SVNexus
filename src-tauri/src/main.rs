// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;

fn main() {
    // Parse CLI arguments. Use `try_parse` so that unknown flags injected by
    // the OS (e.g. macOS `-psn_*` when launched from Finder) don't prevent
    // the application from starting.
    let cli = svnexus_lib::Cli::try_parse().ok();
    let path = cli.and_then(|c| c.path);

    svnexus_lib::run(path)
}
