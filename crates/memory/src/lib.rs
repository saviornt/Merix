use anyhow::{Result};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::path::Path;
use surrealdb::Surreal;
use surrealdb::engine::local::{Db, RocksDb};
use chrono::{DateTime, Utc};
use merix_schemas::{Session, SessionId, TaskId, Checkpoint, CheckpointRecord, SessionRecord};
use serde_json;
use uuid::Uuid;
use tracing::info;
//use merix_utilities::debug_val;

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
        let record = SessionRecord::from(session);

        let _: Option<serde_json::Value> = self.db
            .upsert(("sessions", session.id.0.to_string()))
            .content(record)
            .await?;

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
        // Convert the domain struct into a DB-friendly record
        let record = CheckpointRecord::from(checkpoint);

        let _: Option<serde_json::Value> = self.db
            .upsert(("checkpoints", checkpoint.id.0.to_string()))
            .content(record) // Serializing a Struct vs an Enum Value
            .await?;

        Ok(())
    }

    pub async fn load_latest_checkpoint(&self, task_id: TaskId) -> Result<Option<Checkpoint>> {
        let mut result = self.db
            .query("SELECT * FROM checkpoints WHERE task_id = $task_id ORDER BY timestamp DESC LIMIT 1")
            .bind(("task_id", task_id.0.to_string()))
            .await?;

        // Tell Rust exactly what type we are taking from the response
        let opt_record: Option<CheckpointRecord> = result.take(0)?;

        if let Some(record) = opt_record {
            // We need to extract the ID from the SurrealDB result 
            // SurrealDB returns 'id' as a Thing (table:id), so we treat it as a String
            let db_id: Option<serde_json::Value> = result.take("id")?;
            
            let uuid = db_id
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .and_then(|s| s.split(':').last().map(|s| s.to_string()))
                .and_then(|s| Uuid::parse_str(&s).ok())
                .unwrap_or_else(Uuid::new_v4);

            Ok(Some(record.into_checkpoint(uuid)))
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