use std::path::Path;

use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

// Initialise tracing with two layers:
//   * stdout — human-readable, for local development
//   * file   — JSON, rolled daily at UTC midnight, kept in $LOG_DIR (default `logs/`).
//     Files are named `website_editor.log.YYYY-MM-DD`.
//
// Returns the `WorkerGuard` of the non-blocking writer. The caller MUST keep
// it alive (e.g. by binding it in `main`) until shutdown — when the guard
// drops, any buffered log lines are flushed.
#[must_use = "the WorkerGuard must be held for the lifetime of the program"]
pub fn configure() -> WorkerGuard {
    let log_dir = std::env::var("LOG_DIR").unwrap_or_else(|_| "logs".to_string());
    if let Err(e) = std::fs::create_dir_all(Path::new(&log_dir)) {
        eprintln!("failed to create log directory `{log_dir}`: {e}");
    }

    let file_appender = rolling::daily(&log_dir, "website_editor.log");
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let file_layer = fmt::layer()
        .json()
        .with_current_span(true)
        .with_span_list(false)
        .with_writer(file_writer);

    let stdout_layer = fmt::layer().with_writer(std::io::stdout);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(file_layer)
        .with(stdout_layer)
        .init();

    guard
}
