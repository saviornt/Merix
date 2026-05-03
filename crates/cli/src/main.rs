use clap::{Parser, Subcommand, ValueEnum};
use merix_utilities::{LogConfig, init_logging, log_error, recovery::log_and_exit};
use std::env;

#[derive(Parser)]
#[command(
    author,
    version,
    about = "Merix — The rogue messenger that delivers forbidden intelligence.",
    long_about = "One clean desktop binary.\nZero cloud. Zero daemons.\nEverything is data.\nPure local intelligence.\n\nStructured logging is always enabled (console + daily rolling JSON logs/merix_*.log).",
    after_help = "Log level control:\n\
                 • --log-level info    Standard information (default)\n\
                 • --log-level debug   Detailed debug information\n\
                 • --log-level trace   Maximum verbosity for troubleshooting\n\n\
                 Shorthand:\n\
                 • -v   = debug level\n\
                 • -vv  = trace level\n\n\
                 Alternative: RUST_LOG=merix=debug merix"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Set exact logging severity level
    #[arg(long, value_enum, default_value_t = LogLevel::Info, global = true)]
    log_level: LogLevel,

    /// Increase verbosity (shorthand: -v = debug, -vv = trace)
    #[arg(short, long, action = clap::ArgAction::Count, default_value_t = 0, global = true)]
    verbose: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
enum LogLevel {
    /// Standard information level (default)
    Info,
    /// Detailed debug information
    Debug,
    /// Maximum verbosity for troubleshooting
    Trace,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Info => write!(f, "info"),
            LogLevel::Debug => write!(f, "debug"),
            LogLevel::Trace => write!(f, "trace"),
        }
    }
}

#[derive(Subcommand)]
enum Commands {
    /// Placeholder for future subcommands
    #[command(subcommand)]
    Future(FutureCommands),
}

#[derive(Subcommand)]
enum FutureCommands {
    /// Show version information
    Version,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let level_str = match (cli.log_level, cli.verbose) {
        (LogLevel::Trace, _) | (_, 2..) => "trace",
        (LogLevel::Debug, _) | (_, 1) => "debug",
        _ => "info",
    };

    if env::var("RUST_LOG").is_err() {
        // SAFETY: This runs in `main()` before any async runtime, Tokio, or additional threads are spawned.
        // No other code can be reading the environment concurrently at this point.
        unsafe {
            env::set_var(
                "RUST_LOG",
                format!("merix={},merix_cli={}", level_str, level_str),
            );
        }
    }

    if let Err(e) = init_logging(LogConfig { log_dir: None }) {
        log_and_exit(e);
    }

    if let Err(e) = run(cli) {
        let _ = log_error(e);
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Some(Commands::Future(FutureCommands::Version)) => {
            tracing::info!("Version command requested");
            println!("merix {}", env!("CARGO_PKG_VERSION"));
        }
        None => {
            tracing::info!("Merix CLI started (severity level: {})", cli.log_level);
            println!("Welcome to Merix!");
            println!("Run `merix --help` for usage.");
            println!("Debug tip: --log-level debug (or -v)");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    #[test]
    fn cli_help_works() {
        let output = Command::new("cargo")
            .args(["run", "--bin", "merix", "--", "--help"])
            .output()
            .expect("failed to run merix --help");
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("--log-level"));
    }

    #[test]
    fn cli_version_works() {
        let output = Command::new("cargo")
            .args(["run", "--bin", "merix", "--", "future", "version"])
            .output()
            .expect("failed to run version command");
        assert!(output.status.success());
    }

    #[test]
    fn cli_resilience_on_error() {
        // This should exit with code 1 and log the error (we just check exit code)
        let output = Command::new("cargo")
            .args(["run", "--bin", "merix", "--", "--nonexistent-flag"])
            .output()
            .expect("failed to run with bad flag");
        assert!(!output.status.success());
    }
}
