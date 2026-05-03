//! merix-schemas — Domain models for Database & In-Memory data structures

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
pub use uuid::Uuid;

// Type aliases for clean IDs
pub type SessionId = Uuid;
pub type TaskId = Uuid;
pub type CheckpointId = Uuid;
pub type SkillId = Uuid;

// Local Result alias for clean, consistent error handling
pub type Result<T> = anyhow::Result<T>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Session {
    pub id: SessionId,
    pub title: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub session_id: SessionId,
    pub description: String,
    pub status: TaskStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub parent_id: Option<TaskId>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Paused,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: CheckpointId,
    pub task_id: TaskId,
    pub sequence: u32,
    pub state: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Skill {
    pub id: SkillId,
    pub name: String,
    pub description: String,
    pub code: String,
    pub version: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InferenceConfig {
    pub use_gpu: bool,
    pub n_gpu_layers: i32,
    pub vram_budget_mb: u64,
    pub context_size: u32,
    pub seed: Option<u64>,
    pub n_threads: usize,
    pub n_threads_batch: usize,
    pub memory_pressure_threshold: u8,
    pub cache_type_k: String,
    pub cache_type_v: String,
    pub flash_attn: bool,
    pub no_mmap: bool,
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            use_gpu: true,
            n_gpu_layers: -1,
            vram_budget_mb: 0,
            context_size: 8192,
            seed: Some(42),
            n_threads: std::thread::available_parallelism()
                .map(|p| p.get())
                .unwrap_or(4),
            n_threads_batch: 0,
            memory_pressure_threshold: 85,
            cache_type_k: "q8_0".into(),
            cache_type_v: "q8_0".into(),
            flash_attn: true,
            no_mmap: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schemas_models_compile() {
        let _session_id: SessionId = Uuid::new_v4();
        let _task_id: TaskId = Uuid::new_v4();
        let _config = InferenceConfig::default();
    }
}
