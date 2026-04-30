use anyhow::{anyhow, Result};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::path::Path;
use surrealdb::Surreal;
use surrealdb::engine::local::{Db, RocksDb};
use chrono::{DateTime, Utc};
use merix_models::{Session, SessionId, TaskId, Checkpoint};
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

        info!("MemoryLayer initialized");
        Ok(Self {
            db,
            short_term: DashMap::new(),
        })
    }

    /// Robustly cleans SurrealDB internal types into standard JSON domain models.
    fn clean_record<T: serde::de::DeserializeOwned>(val: surrealdb::Value) -> Result<T> {
        // Laundry step: Serialize to string and back to Value to strip internal SurrealDB enums
        let json_str = serde_json::to_string(&val)?;
        let mut json_val: serde_json::Value = serde_json::from_str(&json_str)?;

        if let Some(obj) = json_val.as_object_mut() {
            if let Some(id_field) = obj.get("id") {
                // Extracts the UUID part from "table:uuid" or complex ID objects
                let id_raw = match id_field {
                    serde_json::Value::String(s) => s.clone(),
                    other => other.to_string().replace('\"', ""),
                };
                
                let clean_id = id_raw.split(':').last().unwrap_or(&id_raw).to_string();
                obj.insert("id".to_string(), serde_json::Value::String(clean_id));
            }
        }

        Ok(serde_json::from_value(json_val)?)
    }

    // === Persistent (SurrealDB) APIs ===

    pub async fn save_session(&self, session: &Session) -> Result<()> {
        let mut payload = serde_json::to_value(session)?;
        if let Some(obj) = payload.as_object_mut() { obj.remove("id"); }

        // FIX: Using type::thing() to cast parameters to Record IDs
        self.db.query("UPDATE type::thing('sessions', $id) CONTENT $data")
            .bind(("id", session.id.0.to_string()))
            .bind(("data", payload))
            .await?;
            
        info!("Session {} persisted", session.id.0);
        Ok(())
    }

    pub async fn load_session(&self, session_id: SessionId) -> Result<Session> {
        // FIX: Using type::thing() for direct Record ID lookups
        let mut response = self.db.query("SELECT * FROM type::thing('sessions', $id)")
            .bind(("id", session_id.0.to_string()))
            .await?;
            
        let val: Option<surrealdb::Value> = response.take(0)?;
        let val = val.ok_or_else(|| anyhow!("Session {} not found", session_id.0))?;

        Self::clean_record(val)
    }

    pub async fn save_checkpoint(&self, checkpoint: &Checkpoint) -> Result<()> {
        let mut payload = serde_json::to_value(checkpoint)?;
        if let Some(obj) = payload.as_object_mut() { obj.remove("id"); }

        self.db.query("UPDATE type::thing('checkpoints', $id) CONTENT $data")
            .bind(("id", checkpoint.id.0.to_string()))
            .bind(("data", payload))
            .await?;
            
        Ok(())
    }

    pub async fn load_latest_checkpoint(&self, task_id: TaskId) -> Result<Option<Checkpoint>> {
        // This query uses a WHERE clause on a field, so type::thing is NOT needed for $task_id
        let mut response = self.db
            .query("SELECT * FROM checkpoints WHERE task_id = $task_id ORDER BY timestamp DESC LIMIT 1")
            .bind(("task_id", task_id.0.to_string()))
            .await?;
            
        let val: Option<surrealdb::Value> = response.take(0)?;
        match val {
            Some(v) => Ok(Some(Self::clean_record(v)?)),
            None => Ok(None)
        }
    }

    pub async fn store_project_memory(&self, key: &str, data: serde_json::Value) -> Result<()> {
        let mut payload = data;
        if let Some(obj) = payload.as_object_mut() { obj.remove("id"); }

        self.db.query("UPDATE type::thing('project', $key) CONTENT $data")
            .bind(("key", key.to_string()))
            .bind(("data", payload))
            .await?;
        Ok(())
    }

    // === Short-term / Ethereal (Dashmap) APIs ===

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
}