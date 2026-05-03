use std::path::PathBuf;

/// Centralized project-wide configuration and constants.
pub struct MerixConfig;

impl MerixConfig {
    /// Application name (used for directories, logs, data, etc.)
    pub const APP_NAME: &str = "Merix";

    /// Returns whether we are running in development mode.
    fn is_dev_mode() -> bool {
        let dev_mode_env = std::env::var("MERIX_DEV_MODE")
            .map(|v| {
                let v = v.trim().to_lowercase();
                v == "1" || v == "true"
            })
            .ok();

        match dev_mode_env {
            Some(false) => false,
            Some(true) => true,
            None => cfg!(debug_assertions),
        }
    }

    /// Returns the Merix workspace root (reliable even when run from inside a crate).
    fn project_root() -> Option<PathBuf> {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let mut path = PathBuf::from(manifest_dir);
        while path.pop() {
            if path.join("Cargo.toml").exists() && path.join("crates").exists() {
                return Some(path);
            }
        }
        None
    }

    /// Returns the OS-appropriate base application data directory for production builds.
    fn os_app_data_base() -> PathBuf {
        if cfg!(target_os = "windows") {
            std::env::var_os("APPDATA")
                .map(PathBuf::from)
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default())
        } else if cfg!(target_os = "macos") {
            std::env::var_os("HOME")
                .map(|h| PathBuf::from(h).join("Library/Application Support"))
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default())
        } else {
            std::env::var_os("HOME")
                .map(|h| PathBuf::from(h).join(".config"))
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default())
        }
    }

    /// Returns the correct log directory.
    pub fn get_log_directory() -> PathBuf {
        if let Ok(dir) = std::env::var("MERIX_LOG_DIR") {
            let p = PathBuf::from(dir);
            eprintln!("Log path override: MERIX_LOG_DIR → {}", p.display());
            return p;
        }

        if Self::is_dev_mode() {
            if let Some(root) = Self::project_root() {
                let p = root.join("logs");
                std::fs::create_dir_all(&p).ok();
                eprintln!("Using development log path → {}", p.display());
                return p;
            }
            let fallback = std::env::current_dir().unwrap_or_default().join("logs");
            std::fs::create_dir_all(&fallback).ok();
            eprintln!(
                "Using fallback development log path → {}",
                fallback.display()
            );
            return fallback;
        }

        let p = Self::os_app_data_base().join(Self::APP_NAME).join("logs");
        std::fs::create_dir_all(&p).ok();
        eprintln!("Using production log path → {}", p.display());
        p
    }

    /// Returns the correct data directory for persistent storage (RocksDB, etc.).
    pub fn get_data_directory() -> PathBuf {
        if let Ok(dir) = std::env::var("MERIX_DATA_DIR") {
            let p = PathBuf::from(dir);
            eprintln!("Data path override: MERIX_DATA_DIR → {}", p.display());
            return p;
        }

        if Self::is_dev_mode() {
            if let Some(root) = Self::project_root() {
                let p = root.join("data");
                std::fs::create_dir_all(&p).ok();
                eprintln!("Using development data path → {}", p.display());
                return p;
            }
            let fallback = std::env::current_dir().unwrap_or_default().join("data");
            std::fs::create_dir_all(&fallback).ok();
            eprintln!(
                "Using fallback development data path → {}",
                fallback.display()
            );
            return fallback;
        }

        let p = Self::os_app_data_base().join(Self::APP_NAME).join("data");
        std::fs::create_dir_all(&p).ok();
        eprintln!("Using production data path → {}", p.display());
        p
    }
}
