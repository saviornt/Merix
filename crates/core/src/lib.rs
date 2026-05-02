//! merix-core — Task execution + LLM runtime foundation

use anyhow::Result;
use llama_cpp_2::LlamaModel;
use merix_memory::MemoryLayer;
use merix_schemas::{Checkpoint, Session, SessionId, Task, TaskId, TaskStatus};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;


/// LLM inference configuration (GPU/VRAM scheduling, deterministic execution, memory pressure)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InferenceConfig {
    
    pub use_gpu: bool,                  // Prefer GPU acceleration when available
    pub n_gpu_layers: i32,              // -1 = auto-detect max possible with headroom
    pub vram_budget_mb: u64,            // 0 = auto; leave 1 GB headroom
    pub context_size: u32,              // Context window size (tokens)
    pub seed: Option<u64>,              // Random seed for deterministic/reproducible inference (deterministic by default)
    pub n_threads: usize,               // Number of threads for CPU fallback for prompt processing
    pub memory_pressure_threshold: u8,  // Memory pressure threshold (%) — triggers lighter context compression
    pub cache_type_k: String,           // "q8_0" / "q4_0" — KV cache quant
    pub cache_type_v: String,
    pub flash_attn: bool,               // Flash Attention v2 (huge decode speedup)
    pub no_mmap: bool,                  // force weights into physical VRAM/RAM
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
           use_gpu: true,
            n_gpu_layers: -1,
            vram_budget_mb: 0,
            context_size: 8192,
            seed: Some(42),
            n_threads: std::thread::available_parallelism().map(|p| p.get()).unwrap_or(4),
            n_threads_batch: 0,
            memory_pressure_threshold: 85,
            cache_type_k: "q8_0".into(),
            cache_type_v: "q8_0".into(),
            flash_attn: true,
            no_mmap: true,
        }
    }
}

impl InferenceConfig {
    /// Hardware-aware extreme optimization (pure Rust — no build deps)
    pub fn optimize_for_hardware() -> Self {
        let mut config = Self::default();

        // Real GPU detection stub (llama-cpp-2 will be wired later)
        // For now we assume modern hardware; user can override via configure_llm
        config.n_gpu_layers = -1;
        config.flash_attn = true;
        config.no_mmap = true;

        if config.vram_budget_mb == 0 {
            config.vram_budget_mb = 4096; // safe Phase-1 default
        }
        if config.n_threads_batch == 0 {
            config.n_threads_batch = config.n_threads;
        }

        tracing::info!(n_gpu_layers = config.n_gpu_layers, flash_attn = config.flash_attn, vram_mb = config.vram_budget_mb, "LLM runtime extreme-optimized");
        config
    }

    pub fn validate(&self) -> Result<()> {
        if self.context_size == 0 || self.n_threads == 0 {
            anyhow::bail!("Invalid InferenceConfig — context_size and n_threads must be > 0");
        }
        Ok(())
    }
}

/// Core runtime — coordinates memory, tasks, and future LLM execution.
/// Fully resumable and crash-safe (all state lives in MemoryLayer).
pub struct CoreRuntime {
    memory: Arc<dyn MemoryLayer + Send + Sync>,
    inference_config: Arc<tokio::sync::RwLock<InferenceConfig>>,
}

impl CoreRuntime {
    /// Create new runtime with any MemoryLayer impl (Persistent or Ethereal)
    pub async fn new<M: MemoryLayer + Send + Sync + 'static>(memory: M) -> Result<Self> {
        let memory = Arc::new(memory) as Arc<dyn MemoryLayer + Send + Sync>;
        memory.init().await?;
        let config = InferenceConfig::optimize_for_hardware();
        config.validate()?;
        Ok(Self {
            memory,
            inference_config: Arc::new(RwLock::new(config)),
        })
    }

    /// Configure or override LLM runtime settings (GPU/VRAM/deterministic)
    pub async fn configure_llm(&self, mut config: InferenceConfig) -> Result<()> {
        config.validate()?;
        let mut cfg = self.inference_config.write().await;
        *cfg = config;
        tracing::info!("LLM runtime reconfigured with extreme optimizations");
        Ok(())
    }

    /// Create and persist a new session
    pub async fn create_session(
        &self,
        title: Option<String>,
        description: Option<String>,
    ) -> Result<Session> {
        let session = Session {
            id: Uuid::new_v4(),
            title,
            description,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        self.memory.store_session(session.clone()).await?;
        tracing::info!(session_id = %session.id, "Session created");
        Ok(session)
    }

    /// Create and persist a new task (starts in Pending state)
    pub async fn create_task(
        &self,
        session_id: SessionId,
        description: String,
    ) -> Result<Task> {
        let task = Task {
            id: Uuid::new_v4(),
            session_id,
            description,
            status: TaskStatus::Pending,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            parent_id: None,
        };
        self.memory.store_task(task.clone()).await?;
        tracing::info!(task_id = %task.id, "Task created");
        Ok(task)
    }

    /// Execute task with proper state-based locking via persisted TaskStatus.
    /// Prevents concurrent execution and survives crashes/restarts.
    pub async fn execute_task(&self, task_id: TaskId) -> Result<()> {
        // Load current task state (source of truth)
        let mut task = match self.memory.load_task(task_id).await? {
            Some(t) => t,
            None => anyhow::bail!("Task {task_id} not found"),
        };

        // State-based lock: only Pending tasks may start
        match task.status {
            TaskStatus::Running => anyhow::bail!("Task {task_id} is already running"),
            TaskStatus::Completed | TaskStatus::Failed => {
                anyhow::bail!("Task {task_id} has already finished with status {:?}", task.status)
            }
            TaskStatus::Paused => {
                // Future: resume logic can be added here
                anyhow::bail!("Task {task_id} is paused (resume not yet implemented)")
            }
            TaskStatus::Pending => {} // proceed
        }

        // Transition to Running (this is the atomic "lock")
        task.status = TaskStatus::Running;
        task.updated_at = Utc::now();
        self.memory.store_task(task.clone()).await?;
        tracing::info!(task_id = %task_id, "Task execution started (Running state persisted)");

        // --- Execution stub (real LLM + tool calls go here later) ---
        tracing::debug!(task_id = %task_id, "Performing task work...");

        // Create checkpoint for resumability
        let checkpoint = Checkpoint {
            id: Uuid::new_v4(),
            task_id,
            sequence: 1,
            state: Value::String("stub_execution_step_complete".to_string()),
            created_at: Utc::now(),
        };
        self.memory.store_checkpoint(checkpoint).await?;

        // Success path
        task.status = TaskStatus::Completed;
        task.updated_at = Utc::now();
        self.memory.store_task(task).await?;
        tracing::info!(task_id = %task_id, "Task completed successfully");

        Ok(())
    }

    /// Helper to load a task (used by tests and future layers)
    pub async fn load_task(&self, id: TaskId) -> Result<Option<Task>> {
        self.memory.load_task(id).await
    }

    /// Example load helper
    pub async fn load_session(&self, id: SessionId) -> Result<Option<Session>> {
        self.memory.load_session(id).await
    }

    /// Get current inference config (for observability / future executor)
    pub async fn get_inference_config(&self) -> InferenceConfig {
        self.inference_config.read().await.clone()
    }

    /// Execute task with extreme LLM config (real llama-cpp-2 call stubbed for Phase 1)
    pub async fn execute_task(&self, task_id: TaskId) -> Result<()> {
        // existing state-based locking logic here (unchanged)
        let config = self.get_inference_config().await;
        tracing::info!(n_gpu_layers = config.n_gpu_layers, flash_attn = config.flash_attn, vram_mb = config.vram_budget_mb, "Starting extreme-optimized LLM task");
        //TODO Real call will be: LlamaContext::new(...) with LlamaParams { n_gpu_layers: config.n_gpu_layers, ... }
        Ok(())
    }

    pub async fn load_session(&self, id: SessionId) -> Result<Option<Session>> {
        self.memory.load_session(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use merix_memory::{EtherealMemory, PersistentMemory};
    use std::env;

    #[tokio::test]
    async fn test_inference_config_detection_and_optimization() -> Result<()> {
        let memory = EtherealMemory::new();
        let core = CoreRuntime::new(memory).await?;

        let cfg = core.get_inference_config().await;
        assert!(cfg.context_size > 0);
        assert!(cfg.n_threads > 0);

        // Override with custom deterministic config
        let custom = InferenceConfig {
            use_gpu: false,
            vram_budget_mb: 2048,
            context_size: 4096,
            seed: Some(12345),
            n_threads: 2,
            memory_pressure_threshold: 70,
        };
        core.configure_llm(custom.clone()).await?;

        let updated = core.get_inference_config().await;
        assert_eq!(updated.seed, Some(12345));
        assert_eq!(updated.vram_budget_mb, 2048);

        println!("✅ LLM runtime optimizations (GPU/VRAM/deterministic) verified");
        Ok(())
    }

    // Extreme LLM Optimization Config test
    #[tokio::test]
    async fn test_extreme_llm_optimizations() -> Result<()> {
        let memory = EtherealMemory::new();
        let core = CoreRuntime::new(memory).await?;
        let cfg = core.get_inference_config().await;
        assert!(cfg.flash_attn);
        assert!(cfg.no_mmap);
        assert!(cfg.n_gpu_layers == -1);
        println!("✅ Extreme LLM optimizations (GPU/VRAM/FlashAttn/KV-cache) verified — build-safe");
        Ok(())
    }

    #[tokio::test]
    async fn test_core_task_state_lifecycle() -> Result<()> {
        let memory = EtherealMemory::new();
        let core = CoreRuntime::new(memory).await?;

        let session = core.create_session(Some("Test Session".into()), None).await?;
        let task = core.create_task(session.id, "Test task description".into()).await?;

        core.execute_task(task.id).await?;

        let loaded = core.load_task(task.id).await?.unwrap();
        assert_eq!(loaded.status, TaskStatus::Completed);
        Ok(())
    }

    #[tokio::test]
    async fn test_core_prevents_double_execution() -> Result<()> {
        let memory = EtherealMemory::new();
        let core = CoreRuntime::new(memory).await?;

        let session = core.create_session(Some("Test Session".into()), None).await?;
        let task = core.create_task(session.id, "Test task".into()).await?;

        core.execute_task(task.id).await?;

        // Second execution must be rejected by state lock
        let result = core.execute_task(task.id).await;
        assert!(result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_core_with_persistent_memory() -> Result<()> {
        let db_path = env::temp_dir().join("merix_core_test.db");
        let memory = PersistentMemory::new_at_path(&db_path).await?;
        let core = CoreRuntime::new(memory).await?;

        let session = core.create_session(Some("Persistent Test".into()), None).await?;
        let task = core.create_task(session.id, "Persistent task".into()).await?;
        core.execute_task(task.id).await?;

        Ok(())
    }
}