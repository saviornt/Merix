use anyhow::{Result};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::path::Path;
use surrealdb::Surreal;
use surrealdb::engine::local::{Db, RocksDb};
use chrono::{DateTime, Utc};
use merix_models::{Session, SessionId, TaskId, Checkpoint, TaskStatus, StepStatus};
use tracing::info;

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

    // === Persistent (SurrealDB) APIs – EVERY ENUM EXPLICITLY TURNED INTO VALID JSON STRING ===
    pub async fn save_session(&self, session: &Session) -> Result<()> {
        let mut payload = serde_json::Map::new();

        payload.insert("created_at".to_string(), serde_json::to_value(session.created_at)?);

        let mut tasks_json: Vec<serde_json::Value> = vec![];
        for task in &session.tasks {
            let mut task_map = serde_json::Map::new();
            task_map.insert("id".to_string(), serde_json::Value::String(task.id.0.to_string()));
            task_map.insert("description".to_string(), serde_json::Value::String(task.description.clone()));

            // EXPLICIT ENUM → JSON STRING (TaskStatus)
            let task_status_str = match task.status {
                TaskStatus::Pending => "pending",
                TaskStatus::Running => "running",
                TaskStatus::Completed => "completed",
                TaskStatus::Failed => "failed",
                TaskStatus::Paused => "paused",
            };
            task_map.insert("status".to_string(), serde_json::Value::String(task_status_str.to_string()));

            let mut steps_json: Vec<serde_json::Value> = vec![];
            for step in &task.steps {
                let mut step_map = serde_json::Map::new();
                step_map.insert("description".to_string(), serde_json::Value::String(step.description.clone()));

                // EXPLICIT ENUM → JSON STRING (StepStatus)
                let step_status_str = match step.status {
                    StepStatus::Pending => "pending",
                    StepStatus::Running => "running",
                    StepStatus::Completed => "completed",
                    StepStatus::Failed => "failed",
                };
                step_map.insert("status".to_string(), serde_json::Value::String(step_status_str.to_string()));

                step_map.insert("output".to_string(), step.output.clone().map_or(serde_json::Value::Null, serde_json::Value::String));
                if let Some(cp_id) = &step.checkpoint_id {
                    step_map.insert("checkpoint_id".to_string(), serde_json::Value::String(cp_id.0.to_string()));
                } else {
                    step_map.insert("checkpoint_id".to_string(), serde_json::Value::Null);
                }
                steps_json.push(serde_json::Value::Object(step_map));
            }
            task_map.insert("steps".to_string(), serde_json::Value::Array(steps_json));

            task_map.insert("created_at".to_string(), serde_json::to_value(task.created_at)?);
            task_map.insert("updated_at".to_string(), serde_json::to_value(task.updated_at)?);

            tasks_json.push(serde_json::Value::Object(task_map));
        }
        payload.insert("tasks".to_string(), serde_json::Value::Array(tasks_json));

        // current_task (Option<TaskId>) → plain string or null
        let current_task_val = match &session.current_task {
            Some(id) => serde_json::Value::String(id.0.to_string()),
            None => serde_json::Value::Null,
        };
        payload.insert("current_task".to_string(), current_task_val);

        let _: Option<serde_json::Value> = self.db.upsert(("sessions", session.id.0.to_string()))
            .content(serde_json::Value::Object(payload))
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
                Ok(merix_models::Session::new())
            }
        }
    }

    pub async fn save_checkpoint(&self, checkpoint: &Checkpoint) -> Result<()> {
        let mut payload = serde_json::Map::new();

        payload.insert("task_id".to_string(), serde_json::Value::String(checkpoint.task_id.0.to_string()));
        payload.insert("session_id".to_string(), serde_json::Value::String(checkpoint.session_id.0.to_string()));
        payload.insert("timestamp".to_string(), serde_json::to_value(checkpoint.timestamp)?);
        payload.insert("state_snapshot".to_string(), checkpoint.state_snapshot.clone());

        let _: Option<serde_json::Value> = self.db.upsert(("checkpoints", checkpoint.id.0.to_string()))
            .content(serde_json::Value::Object(payload))
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