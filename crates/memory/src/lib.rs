use anyhow::{Result};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::path::Path;
use surrealdb::Surreal;
use surrealdb::engine::local::{Db, RocksDb};
use chrono::{DateTime, Utc};
use merix_schemas::{Session, SessionId, TaskId, Checkpoint};
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

    // === Persistent (SurrealDB) APIs – SIMPLE + RELIABLE ENUM-TO-JSON (schemas now correct) ===
    pub async fn save_session(&self, session: &Session) -> Result<()> {
        let mut payload = serde_json::to_value(session)
            .map_err(|e| anyhow::anyhow!("Failed to serialize Session to JSON value: {}", e))?;

        if let Some(obj) = payload.as_object_mut() {
            obj.remove("id");
        }

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
        let mut payload = serde_json::to_value(checkpoint)
            .map_err(|e| anyhow::anyhow!("Failed to serialize Checkpoint to JSON value: {}", e))?;

        if let Some(obj) = payload.as_object_mut() {
            obj.remove("id");
        }

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