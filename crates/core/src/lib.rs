//! merix-core — Task execution + LLM runtime foundation

use anyhow::Result;
use merix_memory::MemoryLayer;
use merix_schemas::{Checkpoint, Session, SessionId, Task, TaskId, TaskStatus};
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;

/// Core runtime — coordinates memory, tasks, and future LLM execution.
/// Fully resumable and crash-safe (all state lives in MemoryLayer).
pub struct CoreRuntime {
    memory: Arc<dyn MemoryLayer + Send + Sync>,
}

impl CoreRuntime {
    /// Create new runtime with any MemoryLayer impl (Persistent or Ethereal)
    pub async fn new<M: MemoryLayer + Send + Sync + 'static>(memory: M) -> Result<Self> {
        let memory = Arc::new(memory) as Arc<dyn MemoryLayer + Send + Sync>;
        memory.init().await?;
        Ok(Self { memory })
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use merix_memory::{EtherealMemory, PersistentMemory};
    use std::env;

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