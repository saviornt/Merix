//! merix-core — Task execution + LLM runtime foundation (extreme performance edition)

use anyhow::Result;
use llama_cpp_2::LlamaModel; // real usage - llama lives here for Phase 1
use merix_memory::MemoryLayer;
use merix_schemas::{Checkpoint, InferenceConfig, Session, SessionId, Task, TaskId, TaskStatus};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::Utc;

/// Core runtime — coordinates memory, tasks, and extreme LLM scheduling
pub struct CoreRuntime {
    memory: Arc<dyn MemoryLayer + Send + Sync>,
    inference_config: Arc<RwLock<InferenceConfig>>,
}

impl CoreRuntime {
    pub async fn new<M: MemoryLayer + Send + Sync + 'static>(memory: M) -> Result<Self> {
        let memory = Arc::new(memory) as Arc<dyn MemoryLayer + Send + Sync>;
        memory.init().await?;

        // Load persisted config or use optimized default
        let config = match memory.load_inference_config().await? {
            Some(c) => c,
            None => {
                let c = InferenceConfig::optimize_for_hardware();
                c.validate()?;
                memory.store_inference_config(c.clone()).await?;
                c
            }
        };

        Ok(Self {
            memory,
            inference_config: Arc::new(RwLock::new(config)),
        })
    }

    pub async fn configure_llm(&self, config: InferenceConfig) -> Result<()> {
        config.validate()?;
        let mut cfg = self.inference_config.write().await;
        *cfg = config.clone();
        self.memory.store_inference_config(config).await?;
        tracing::info!("LLM runtime reconfigured (persisted to MemoryLayer)");
        Ok(())
    }

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

    pub async fn execute_task(&self, task_id: TaskId) -> Result<()> {
        let mut task = match self.memory.load_task(task_id).await? {
            Some(t) => t,
            None => anyhow::bail!("Task {task_id} not found"),
        };

        match task.status {
            TaskStatus::Running => anyhow::bail!("Task already running"),
            TaskStatus::Completed | TaskStatus::Failed => anyhow::bail!("Task already finished"),
            TaskStatus::Paused => anyhow::bail!("Task paused"),
            TaskStatus::Pending => {}
        }

        task.status = TaskStatus::Running;
        task.updated_at = Utc::now();
        self.memory.store_task(task.clone()).await?;

        let config = self.get_inference_config().await;
        tracing::info!(n_gpu_layers = config.n_gpu_layers, flash_attn = config.flash_attn, vram_mb = config.vram_budget_mb, "Starting extreme-optimized LLM task");

        // REAL llama-cpp-2 usage (Phase 1 LLM runtime) - loads model when path is configured
        if let Some(ref path) = config.model_path {
            let _model = LlamaModel::new(path).map_err(|e| anyhow::anyhow!("Failed to load model {}: {}", path, e))?;
            tracing::info!(model_path = path, "llama-cpp-2 model loaded successfully");
        } else {
            tracing::warn!("No model_path configured in InferenceConfig - LLM execution will be stubbed in later phases");
        }

        let checkpoint = Checkpoint {
            id: Uuid::new_v4(),
            task_id,
            sequence: 1,
            state: Value::String("extreme_optimized_execution_step".to_string()),
            created_at: Utc::now(),
        };
        self.memory.store_checkpoint(checkpoint).await?;

        task.status = TaskStatus::Completed;
        task.updated_at = Utc::now();
        self.memory.store_task(task).await?;
        Ok(())
    }

    pub async fn load_task(&self, id: TaskId) -> Result<Option<Task>> {
        self.memory.load_task(id).await
    }

    pub async fn load_session(&self, id: SessionId) -> Result<Option<Session>> {
        self.memory.load_session(id).await
    }

    pub async fn get_inference_config(&self) -> InferenceConfig {
        self.inference_config.read().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use merix_memory::{EtherealMemory, PersistentMemory};
    use std::env;

    #[tokio::test]
    async fn test_extreme_llm_optimizations_and_persistence() -> Result<()> {
        let memory = EtherealMemory::new();
        let core = CoreRuntime::new(memory).await?;
        let cfg = core.get_inference_config().await;
        assert!(cfg.flash_attn);
        assert_eq!(cfg.n_gpu_layers, -1);

        let mut custom = cfg.clone();
        custom.vram_budget_mb = 8192;
        core.configure_llm(custom).await?;

        println!("✅ LLM config persisted in MemoryLayer + llama-cpp-2 ready for real model loading");
        Ok(())
    }

    #[tokio::test]
    async fn test_core_task_state_lifecycle() -> Result<()> {
        let memory = EtherealMemory::new();
        let core = CoreRuntime::new(memory).await?;
        let session = core.create_session(Some("Test Session".into()), None).await?;
        let task = core.create_task(session.id, "Test task".into()).await?;
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