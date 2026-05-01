use std::path::PathBuf;

/// Centralized project-wide configuration and constants.
pub struct MerixConfig;

impl MerixConfig {
    /// Application name (used for directories, logs, etc.)
    pub const APP_NAME: &str = "Merix";

    /// Returns the correct log directory.
    /// - Debug builds (`cargo run`) → project root ./logs (developer default)
    /// - Release builds → OS production path (%APPDATA%/Merix/logs on Windows)
    /// - MERIX_DEV_MODE=0 forces production even in debug builds
    pub fn get_log_directory() -> PathBuf {
        // Highest priority override
        if let Ok(dir) = std::env::var("MERIX_LOG_DIR") {
            return PathBuf::from(dir);
        }

        // Explicit dev-mode env var check
        let dev_mode_env = std::env::var("MERIX_DEV_MODE")
            .map(|v| {
                let v = v.trim().to_lowercase();
                v == "1" || v == "true"
            })
            .ok();

        let is_dev = match dev_mode_env {
            Some(false) => false,
            Some(true) => true,
            None => cfg!(debug_assertions),
        };

        if is_dev {
            // Use compile-time workspace root
            let manifest_dir = env!("CARGO_MANIFEST_DIR");
            let mut path = PathBuf::from(manifest_dir);
            // Walk up to workspace root
            while path.pop() {
                if path.join("Cargo.toml").exists() && path.join("crates").exists() {
                    return path.join("logs");
                }
            }
            // Fallback if walk fails
            return std::env::current_dir().unwrap_or_default().join("logs");
        }

        // Production: OS-appropriate application data directory
        let base = if cfg!(target_os = "windows") {
            std::env::var_os("APPDATA")
                .map(PathBuf::from)
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default())
        } else if cfg!(target_os = "macos") {
            std::env::var_os("HOME")
                .map(|h| PathBuf::from(h).join("Library/Application Support"))
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default())
        } else {
            // Linux and other Unix-like
            std::env::var_os("HOME")
                .map(|h| PathBuf::from(h).join(".config"))
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default())
        };

        base.join(Self::APP_NAME).join("logs")
    }
}