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
    db: Surreal<Db>,                              // Official concrete type for local engines
    short_term: DashMap<String, MemoryItem>,
}

impl MemoryLayer {
    pub async fn new(storage_path: &str) -> Result<Self> {
        let path = Path::new(storage_path).join("memory");
        std::fs::create_dir_all(&path)?;

        // Official stable embedded RocksDB pattern (kv-rocksdb feature)
        let db_path = path.to_string_lossy().into_owned();
        let db = Surreal::new::<RocksDb>(&db_path).await?;
        db.use_ns("merix").use_db("runtime").await?;

        info!("MemoryLayer initialized (SurrealDB RocksDB persistent + Dashmap short-term)");
        Ok(Self {
            db,
            short_term: DashMap::new(),
        })
    }

    // === Persistent (SurrealDB) APIs ===
    pub async fn save_session(&self, session: &Session) -> Result<()> {
        let _: Option<Session> = self.db.create(("sessions", session.id.0.to_string()))
            .content(session.clone())
            .await?;
        info!("Session {} persisted in long-term memory", session.id.0);
        Ok(())
    }

    pub async fn load_session(&self, session_id: SessionId) -> Result<Session> {
        let opt: Option<Session> = self.db.select(("sessions", session_id.0.to_string())).await?;
        opt.ok_or_else(|| anyhow!("Session not found in persistent memory"))
    }

    pub async fn save_checkpoint(&self, checkpoint: &Checkpoint) -> Result<()> {
        let _: Option<Checkpoint> = self.db.create(("checkpoints", checkpoint.id.0.to_string()))
            .content(checkpoint.clone())
            .await?;
        info!("Checkpoint {} persisted", checkpoint.id.0);
        Ok(())
    }

    pub async fn load_latest_checkpoint(&self, task_id: TaskId) -> Result<Option<Checkpoint>> {
        let mut result = self.db.query("SELECT * FROM checkpoints WHERE task_id = $task_id ORDER BY timestamp DESC LIMIT 1")
            .bind(("task_id", task_id.0))
            .await?;
        let cp: Option<Checkpoint> = result.take(0)?;
        Ok(cp)
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

    // === Retrieval-based Context Reconstruction ===
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

    // === Structured Project Memory ===
    pub async fn store_project_memory(&self, key: &str, data: serde_json::Value) -> Result<()> {
        let _: Option<serde_json::Value> = self.db.create(("project", key))
            .content(data)
            .await?;
        Ok(())
    }
}