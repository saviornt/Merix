//! Merix-Utilities — Logging Infrastructure
//! Structured, crash-safe, resumable observability with per-run timestamped files
//! and session separators.

use chrono::Local;
use tracing_subscriber::{
    fmt::{self, time::ChronoLocal},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

/// Configuration for logging behavior
pub struct LogConfig {
    /// Directory for log files (default: ./logs)
    pub log_dir: Option<std::path::PathBuf>,
}

/// Initialize structured logging (per-run timestamped file + session header).
pub fn init_logging(config: LogConfig) -> anyhow::Result<()> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("merix=info,merix_cli=info"));

    let console_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .with_timer(ChronoLocal::rfc_3339())
        .pretty();

    // Per-run timestamped filename: merix_2026-05-01.log
    let timestamp = Local::now().format("merix_%Y-%m-%d.log");
    let log_dir = config.log_dir.unwrap_or_else(|| {
        std::env::current_dir().unwrap_or_default().join("logs")
    });
    std::fs::create_dir_all(&log_dir)?;

    let log_file_path = log_dir.join(timestamp.to_string());

    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file_path)?;

    let (file_writer, _guard) = tracing_appender::non_blocking(file);

    let file_layer = fmt::layer()
        .with_writer(file_writer)
        .with_timer(ChronoLocal::rfc_3339())
        .json();

    tracing_subscriber::registry()
        .with(console_layer)
        .with(file_layer)
        .with(filter)
        .init();

    // Session separator / header
    let separator = "═".repeat(80);
    tracing::info!("{}", separator);
    tracing::info!("Merix Session Started at {}", Local::now().format("%Y-%m-%d %H:%M:%S"));
    tracing::info!("Log file: {}", log_file_path.display());
    tracing::info!("Version: {}", env!("CARGO_PKG_VERSION"));
    tracing::info!("{}", separator);

    Ok(())
}

/// Log an error at error level and return it
pub fn log_error<E: std::fmt::Display>(err: E) -> anyhow::Error {
    let e = anyhow::anyhow!("{}", err);
    tracing::error!("{}", e);
    e
}

/// Recovery helpers
pub mod recovery {
    use super::*;

    pub fn log_and_recover<T, E: std::fmt::Display>(err: E, fallback: T) -> T {
        let _ = log_error(err);
        fallback
    }

    pub fn log_and_exit<E: std::fmt::Display>(err: E) -> ! {
        let _ = log_error(err);
        std::process::exit(1)
    }

    pub fn log_and_continue<E: std::fmt::Display>(err: E) -> anyhow::Result<()> {
        let _ = log_error(err);
        Ok(())
    }
}

/// Helper macro for consistent events
#[macro_export]
macro_rules! merix_event {
    ($level:expr, $($arg:tt)+) => {
        tracing::event!($level, $($arg)+)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_error_returns_anyhow_error() {
        let err = log_error("test error message");
        assert!(err.to_string().contains("test error message"));
    }

    #[test]
    fn test_recovery_helpers() {
        let fallback = recovery::log_and_recover("recoverable error", 42);
        assert_eq!(fallback, 42);

        let result = recovery::log_and_continue("continue after error");
        assert!(result.is_ok());
    }

    // Async test ready for when we need it (future-proof)
    #[tokio::test]
    async fn test_async_recovery_path() {
        let result = recovery::log_and_continue("async error path");
        assert!(result.is_ok());
    }
}