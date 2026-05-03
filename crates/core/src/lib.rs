//! merix-core — Task execution + LLM runtime foundation

use anyhow::Result;
use merix_llama::LlamaRuntime;
use merix_memory::{MemoryLayer};
use merix_schemas::{Checkpoint, InferenceConfig, Session, SessionId, Task, TaskId, TaskStatus};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::Utc;

/// Core runtime — coordinates memory, tasks, LlamaRuntime, and high-level orchestration.
/// Fully resumable and crash-safe.
pub struct CoreRuntime {
    memory: Arc<dyn MemoryLayer + Send + Sync>,
    llama: Arc<RwLock<LlamaRuntime>>,
    inference_config: Arc<RwLock<InferenceConfig>>,
}

impl CoreRuntime {
    pub async fn new<M: MemoryLayer + Send + Sync + 'static>(memory: M) -> Result<Self> {
        let memory = Arc::new(memory) as Arc<dyn MemoryLayer + Send + Sync>;
        memory.init().await?;

        let llama = LlamaRuntime::new()?;
        let config = match memory.load_inference_config().await? {
            Some(c) => c,
            None => {
                let c = InferenceConfig::default();
                memory.store_inference_config(c.clone()).await?;
                c
            }
        };

        Ok(Self {
            memory,
            llama: Arc::new(RwLock::new(llama)),
            inference_config: Arc::new(RwLock::new(config)),
        })
    }

    pub async fn configure_llm(&self, config: InferenceConfig) -> Result<()> {
        let mut cfg = self.inference_config.write().await;
        *cfg = config.clone();
        self.memory.store_inference_config(config).await?;
        tracing::info!("LLM runtime reconfigured (persisted)");
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

    /// Executes a task with real model loading via merix-llama.
    /// Model path comes from CLI or integration/unit tests.
    pub async fn execute_task(&self, task_id: TaskId, model_path: Option<&str>) -> Result<()> {
        let mut task = match self.memory.load_task(task_id).await? {
            Some(t) => t,
            None => anyhow::bail!("Task {task_id} not found"),
        };

        if !matches!(task.status, TaskStatus::Pending) {
            anyhow::bail!("Task cannot be executed (status: {:?})", task.status);
        }

        task.status = TaskStatus::Running;
        task.updated_at = Utc::now();
        self.memory.store_task(task.clone()).await?;

        let config = self.get_inference_config().await;
        tracing::info!(
            use_gpu = config.use_gpu,
            n_gpu_layers = config.n_gpu_layers,
            vram_mb = config.vram_budget_mb,
            model_path = model_path.unwrap_or("<none>"),
            "Executing task with LlamaRuntime"
        );

        {
            let mut llama = self.llama.write().await;
            if let Some(path) = model_path {
                llama.load_model(path)?;
                tracing::info!(model_path = path, "Model successfully loaded into LlamaRuntime (CUDA ready)");
            } else {
                tracing::warn!("No model_path provided - LLM execution stubbed");
            }
        }

        let checkpoint = Checkpoint {
            id: Uuid::new_v4(),
            task_id,
            sequence: 1,
            state: Value::String("task_executed_via_llama_runtime".to_string()),
            created_at: Utc::now(),
        };
        self.memory.store_checkpoint(checkpoint).await?;

        task.status = TaskStatus::Completed;
        task.updated_at = Utc::now();
        self.memory.store_task(task).await?;
        tracing::info!(task_id = %task_id, "Task completed successfully");
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
    use std::path::PathBuf;
    use std::env;

    fn unique_db_path(name: &str) -> PathBuf {
        env::temp_dir().join(format!("merix_core_test_{}_{}.db", name, Uuid::new_v4()))
    }

    #[tokio::test]
    async fn test_core_task_state_lifecycle() -> Result<()> {
        let memory = EtherealMemory::new();
        let core = CoreRuntime::new(memory).await?;
        let session = core.create_session(Some("Test Session".into()), None).await?;
        let task = core.create_task(session.id, "Test task".into()).await?;
        core.execute_task(task.id, None).await?;
        let loaded = core.load_task(task.id).await?.unwrap();
        assert_eq!(loaded.status, TaskStatus::Completed);
        Ok(())
    }

    #[tokio::test]
    async fn test_core_with_persistent_memory() -> Result<()> {
        let db_path = unique_db_path("unit_persistent");
        let memory = PersistentMemory::new_at_path(&db_path).await?;
        let core = CoreRuntime::new(memory).await?;
        let session = core.create_session(Some("Persistent Test".into()), None).await?;
        let task = core.create_task(session.id, "Persistent task".into()).await?;
        core.execute_task(task.id, None).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_core_llm_configuration() -> Result<()> {
        let memory = EtherealMemory::new();
        let core = CoreRuntime::new(memory).await?;
        let config = InferenceConfig {
            use_gpu: true,
            n_gpu_layers: 42,
            ..Default::default()
        };
        core.configure_llm(config).await?;
        Ok(())
    }
}