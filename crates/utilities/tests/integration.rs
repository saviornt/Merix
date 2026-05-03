#[cfg(test)]
mod tests {
    use merix_utilities::{LogConfig, init_logging};
    use std::env;

    #[tokio::test]
    async fn integration_test_logging_creates_timestamped_file() {
        // SAFETY: This is a single-threaded integration test; no other threads are reading the environment.
        unsafe {
            env::set_var("RUST_LOG", "merix=debug");
        }

        let test_dir = std::env::temp_dir().join("merix_integration_test_logs");

        let result = init_logging(LogConfig {
            log_dir: Some(test_dir.clone()),
        });

        assert!(result.is_ok(), "init_logging should succeed");

        // Verify a timestamped log file was created
        let files: Vec<_> = std::fs::read_dir(&test_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().starts_with("merix_"))
            .collect();

        assert!(!files.is_empty(), "should have created a merix_*.log file");
    }
}
