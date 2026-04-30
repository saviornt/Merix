use anyhow::{anyhow, Result};
use std::path::Path;
use tokio::fs;
use merix_schemas::{Session, Task, Checkpoint, TaskStatus, SessionId, StepStatus};
use tracing::{info, warn};
use merix_utilities::debug_val;

pub struct TaskExecutor {
    storage_path: String,
}

impl TaskExecutor {
    pub fn new(storage_path: &str) -> Self {
        Self {
            storage_path: storage_path.to_string(),
        }
    }

    pub async fn save_session(&self, session: &Session) -> Result<()> {
        debug_val("core::save_session - session", session);
        
        let path = Path::new(&self.storage_path).join(format!("session_{}.json", session.id.0));
        //debug_val("core::save_session - path", path);
        
        let json = serde_json::to_string_pretty(session)?;
        //debug_val("core::save_session - json", json);
        
        fs::write(&path, json).await?;
        info!("Session {} saved", session.id.0);
        Ok(())
    }

    pub async fn load_session(&self, session_id: SessionId) -> Result<Session> {
        let path = Path::new(&self.storage_path).join(format!("session_{}.json", session_id.0));
        if !path.exists() {
            return Err(anyhow!("Session not found"));
        }
        let data = fs::read_to_string(&path).await?;
        let session: Session = serde_json::from_str(&data)?;
        info!("Session {} loaded (resumable)", session_id.0);
        Ok(session)
    }

    pub async fn create_checkpoint(&self, session: &Session, task: &Task, step_index: usize) -> Result<Checkpoint> {
        // Explicit manual JSON for state_snapshot (no enum serialization path)
        debug_val("core::create_checkpoint - session", session);
        debug_val("core::create_checkpoint - task", task);

        let state_snapshot = serde_json::json!({
            "id": task.id.0.to_string(),
            "description": task.description,
            "status": match task.status {
                TaskStatus::Pending => "pending",
                TaskStatus::Running => "running",
                TaskStatus::Completed => "completed",
                TaskStatus::Failed => "failed",
                TaskStatus::Paused => "paused",
            },
            "steps": task.steps.iter().map(|step| {
                serde_json::json!({
                    "description": step.description,
                    "status": match step.status {
                        StepStatus::Pending => "pending",
                        StepStatus::Running => "running",
                        StepStatus::Completed => "completed",
                        StepStatus::Failed => "failed",
                    },
                    "output": step.output,
                    "checkpoint_id": step.checkpoint_id.as_ref().map(|id| id.0.to_string())
                })
            }).collect::<Vec<_>>(),
            "created_at": task.created_at,
            "updated_at": task.updated_at
        });
        debug_val("core::create_checkpoint - state_snapshot", &state_snapshot);

        let checkpoint = Checkpoint::new(
            task.id.clone(),
            session.id.clone(),
            state_snapshot,
        );
        debug_val("core::create_checkpoint - checkpoint", &checkpoint);

        info!("Checkpoint created at step {} for task {}", step_index, task.id.0);
        Ok(checkpoint)
    }

    pub async fn execute_task(&self, session: &mut Session, task_description: String) -> Result<()> {
        let mut task = Task::new(task_description);
        task.add_step("Initialize execution context".to_string());
        task.add_step("Perform core computation".to_string());
        task.add_step("Persist outputs and finalize".to_string());

        session.add_task(task.clone());

        for i in 0..task.steps.len() {
            // Checkpoint BEFORE mutable borrow of step (fixes borrow checker)
            let _cp = self.create_checkpoint(session, &task, i).await?;

            let step = &mut task.steps[i];
            info!("Executing step {}: {}", i, step.description);
            step.status = StepStatus::Running;

            // Simulate work
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            step.status = StepStatus::Completed;
            step.output = Some(format!("Step {} completed successfully", i));
        }

        task.status = TaskStatus::Completed;
        if let Some(pos) = session.tasks.iter().position(|t| t.id == task.id) {
            session.tasks[pos] = task;
        }

        self.save_session(session).await?;
        info!("Task completed and session persisted");
        Ok(())
    }

    pub async fn resume_task(&self, session: &mut Session) -> Result<()> {
        if let Some(task_id) = &session.current_task {
            warn!("Resuming task {} from last checkpoint", task_id.0);
        }
        Ok(())
    }
}