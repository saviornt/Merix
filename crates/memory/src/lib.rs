use anyhow::{Result};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::path::Path;
use surrealdb::Surreal;
use surrealdb::engine::local::{Db, RocksDb};
use chrono::{DateTime, Utc};
use merix_schemas::{Session, SessionId, TaskId, Checkpoint, StepStatus, TaskStatus};
use serde_json;
use tracing::info;
use merix_utilities::debug_val;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryItem {
    pub key: String,
    pub value: String,
    pub timestamp: DateTime<Utc>,
}

pub struct MemoryLayer {
    db: Surreal<Db>,
    short_term: DashMap<String, MemoryItem>,
}

impl MemoryLayer {
    pub async fn new(storage_path: &str) -> Result<Self> {
        let path = Path::new(storage_path).join("memory");
        std::fs::create_dir_all(&path)?;

        let db_path = path.to_string_lossy().into_owned();
        let db = Surreal::new::<RocksDb>(&db_path).await?;
        db.use_ns("merix").use_db("runtime").await?;

        info!("MemoryLayer initialized (SurrealDB RocksDB persistent + DashMap short-term)");
        Ok(Self {
            db,
            short_term: DashMap::new(),
        })
    }

    // === Persistent (SurrealDB) APIs – SIMPLE + RELIABLE ENUM-TO-JSON (schemas now correct) ===
    pub async fn save_session(&self, session: &Session) -> Result<()> {
        debug_val("session", session);

        let mut tasks_json = vec![];
        debug_val("tasks_json (initial)", &tasks_json);

        for (i, task) in session.tasks.iter().enumerate() {
            debug_val(&format!("task[{}]", i), task);

            let mut steps_json = vec![];
            debug_val(&format!("steps_json for task[{}]", i), &steps_json);

            for (j, step) in task.steps.iter().enumerate() {
                debug_val(&format!("step[{}]", j), step);

                let step_status_str = match step.status {
                    StepStatus::Pending => "pending",
                    StepStatus::Running => "running",
                    StepStatus::Completed => "completed",
                    StepStatus::Failed => "failed",
                };
                debug_val(&format!("step[{}].status_str", j), &step_status_str);

                let step_json = serde_json::json!({
                    "description": step.description,
                    "status": step_status_str,
                    "output": step.output,
                    "checkpoint_id": step.checkpoint_id.as_ref().map(|id| id.0.to_string())
                });
                debug_val(&format!("step[{}].json", j), &step_json);

                steps_json.push(step_json);
            }

            let task_status_str = match task.status {
                TaskStatus::Pending => "pending",
                TaskStatus::Running => "running",
                TaskStatus::Completed => "completed",
                TaskStatus::Failed => "failed",
                TaskStatus::Paused => "paused",
            };
            debug_val(&format!("task[{}].status_str", i), &task_status_str);

            let task_json = serde_json::json!({
                "id": task.id.0.to_string(),
                "description": task.description,
                "status": task_status_str,
                "steps": steps_json,
                "created_at": task.created_at,
                "updated_at": task.updated_at
            });
            debug_val(&format!("task[{}].json", i), &task_json);

            tasks_json.push(task_json);
        }

        let current_task_val = match &session.current_task {
            Some(id) => serde_json::Value::String(id.0.to_string()),
            None => serde_json::Value::Null,
        };
        debug_val("current_task_val", &current_task_val);

        let payload = serde_json::json!({
            "created_at": session.created_at,
            "tasks": tasks_json,
            "current_task": current_task_val
        });
        debug_val("final payload (save_session)", &payload);

        let _: Option<serde_json::Value> = self.db.upsert(("sessions", session.id.0.to_string()))
            .content(payload)
            .await?;

        info!("Session {} persisted in long-term memory", session.id.0);
        Ok(())
    }

    pub async fn load_session(&self, session_id: SessionId) -> Result<Session> {
        let opt: Option<serde_json::Value> = self.db.select(("sessions", session_id.0.to_string())).await?;
        
        match opt {
            Some(mut value) => {
                if let Some(obj) = value.as_object_mut() {
                    obj.insert("id".to_string(), serde_json::Value::String(session_id.0.to_string()));
                }
                let session: Session = serde_json::from_value(value)
                    .map_err(|e| anyhow::anyhow!("Failed to deserialize Session from JSON value: {}", e))?;
                Ok(session)
            }
            None => {
                info!("Session {} not found — creating empty session on-the-fly", session_id.0);
                Ok(merix_schemas::Session::new())
            }
        }
    }

    pub async fn save_checkpoint(&self, checkpoint: &Checkpoint) -> Result<()> {
        debug_val("checkpoint", checkpoint);

        let payload = serde_json::json!({
            "task_id": checkpoint.task_id.0.to_string(),
            "session_id": checkpoint.session_id.0.to_string(),
            "timestamp": checkpoint.timestamp,
            "state_snapshot": checkpoint.state_snapshot
        });
        debug_val("final payload (save_checkpoint)", &payload);

        let _: Option<serde_json::Value> = self.db.upsert(("checkpoints", checkpoint.id.0.to_string()))
            .content(payload)
            .await?;

        info!("Checkpoint {} persisted", checkpoint.id.0);
        Ok(())
    }

    pub async fn load_latest_checkpoint(&self, task_id: TaskId) -> Result<Option<Checkpoint>> {
        let mut result = self.db.query("SELECT * FROM checkpoints WHERE task_id = $task_id ORDER BY timestamp DESC LIMIT 1")
            .bind(("task_id", task_id.0.to_string()))
            .await?;
            
        let opt_value: Option<serde_json::Value> = result.take(0)?;
        
        if let Some(mut val) = opt_value {
            if let Some(obj) = val.as_object_mut() {
                if let Some(id_val) = obj.get("id") {
                    if let Some(id_str) = id_val.as_str() {
                        let clean_id = id_str.split(':').last().unwrap_or(id_str);
                        obj.insert("id".to_string(), serde_json::Value::String(clean_id.to_string()));
                    }
                }
            }
            let cp: Checkpoint = serde_json::from_value(val)
                .map_err(|e| anyhow::anyhow!("Failed to deserialize Checkpoint from JSON value: {}", e))?;
            Ok(Some(cp))
        } else {
            Ok(None)
        }
    }

    pub fn store_ephemeral(&self, key: String, value: String) {
        let item = MemoryItem {
            key: key.clone(),
            value,
            timestamp: Utc::now(),
        };
        self.short_term.insert(key, item);
    }

    pub fn get_ephemeral(&self, key: &str) -> Option<String> {
        self.short_term.get(key).map(|r| r.value().value.clone())
    }

    pub async fn reconstruct_context(&self, session_id: SessionId) -> Result<String> {
        let session = self.load_session(session_id).await?;
        let mut context = format!("Session {} ({} tasks)\n", session.id.0, session.tasks.len());

        for task in &session.tasks {
            context.push_str(&format!("Task: {} [{:?}]\n", task.description, task.status));
            for step in &task.steps {
                if let Some(output) = &step.output {
                    context.push_str(&format!("  Step: {} → {}\n", step.description, output));
                }
            }
        }

        for entry in self.short_term.iter() {
            context.push_str(&format!("Ephemeral: {} = {}\n", entry.key(), entry.value().value));
        }
        Ok(context)
    }

    pub async fn store_project_memory(&self, key: &str, data: serde_json::Value) -> Result<()> {
        let _: Option<serde_json::Value> = self.db.upsert(("project", key))
            .content(data)
            .await?;
        Ok(())
    }
}