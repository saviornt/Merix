//! merix-memory — Persistent (SurrealDB + RocksDB) + Ethereal (Dashmap) memory layer

use anyhow::Result;
use async_trait::async_trait;
use merix_schemas::{
    Checkpoint, CheckpointId, InferenceConfig, Session, SessionId, Skill, SkillId, Task, TaskId,
};
use serde_json::Value;
use std::path::Path;
use surrealdb::Surreal;
use surrealdb::engine::local::{Db, RocksDb};

/// Unified MemoryLayer trait (both persistent and ethereal implement it)
#[async_trait]
pub trait MemoryLayer: Send + Sync {
    async fn init(&self) -> Result<()>;
    async fn store_session(&self, session: Session) -> Result<()>;
    async fn load_session(&self, id: SessionId) -> Result<Option<Session>>;
    async fn store_task(&self, task: Task) -> Result<()>;
    async fn load_task(&self, id: TaskId) -> Result<Option<Task>>;
    async fn store_checkpoint(&self, checkpoint: Checkpoint) -> Result<()>;
    async fn load_checkpoint(&self, id: CheckpointId) -> Result<Option<Checkpoint>>;
    async fn store_skill(&self, skill: Skill) -> Result<()>;
    async fn load_skill(&self, id: SkillId) -> Result<Option<Skill>>;

    // Dynamic, persisted LLM configuration (Phase 1 requirement)
    async fn store_inference_config(&self, config: InferenceConfig) -> Result<()>;
    async fn load_inference_config(&self) -> Result<Option<InferenceConfig>>;
}

/// Persistent memory (SurrealDB + RocksDB — disk-backed, resumable)
pub struct PersistentMemory {
    db: Surreal<Db>,
}

impl PersistentMemory {
    pub async fn new() -> Result<Self> {
        let data_dir = merix_utilities::config::MerixConfig::get_data_directory();
        let db_path = data_dir.join("merix.db");
        let db = Surreal::new::<RocksDb>(db_path).await?;
        Ok(Self { db })
    }

    pub async fn new_at_path(db_path: impl AsRef<Path>) -> Result<Self> {
        let db = Surreal::new::<RocksDb>(db_path.as_ref()).await?;
        Ok(Self { db })
    }

    fn fix_id_in_value(mut value: Value, id: impl ToString) -> Value {
        if let Some(id_val) = value.get_mut("id") {
            *id_val = Value::String(id.to_string());
        }
        value
    }
}

#[async_trait]
impl MemoryLayer for PersistentMemory {
    async fn init(&self) -> Result<()> {
        self.db.use_ns("merix").use_db("main").await?;
        Ok(())
    }

    async fn store_session(&self, session: Session) -> Result<()> {
        let value = serde_json::to_value(&session)?;
        let _: Option<Value> = self
            .db
            .upsert(("session", session.id.to_string()))
            .content(value)
            .await?;
        Ok(())
    }

    async fn load_session(&self, id: SessionId) -> Result<Option<Session>> {
        let opt: Option<Value> = self.db.select(("session", id.to_string())).await?;
        match opt {
            Some(v) => Ok(serde_json::from_value(Self::fix_id_in_value(v, id))?),
            None => Ok(None),
        }
    }

    async fn store_task(&self, task: Task) -> Result<()> {
        let value = serde_json::to_value(&task)?;
        let _: Option<Value> = self
            .db
            .upsert(("task", task.id.to_string()))
            .content(value)
            .await?;
        Ok(())
    }

    async fn load_task(&self, id: TaskId) -> Result<Option<Task>> {
        let opt: Option<Value> = self.db.select(("task", id.to_string())).await?;
        match opt {
            Some(v) => Ok(serde_json::from_value(Self::fix_id_in_value(v, id))?),
            None => Ok(None),
        }
    }

    async fn store_checkpoint(&self, checkpoint: Checkpoint) -> Result<()> {
        let value = serde_json::to_value(&checkpoint)?;
        let _: Option<Value> = self
            .db
            .upsert(("checkpoint", checkpoint.id.to_string()))
            .content(value)
            .await?;
        Ok(())
    }

    async fn load_checkpoint(&self, id: CheckpointId) -> Result<Option<Checkpoint>> {
        let opt: Option<Value> = self.db.select(("checkpoint", id.to_string())).await?;
        match opt {
            Some(v) => Ok(serde_json::from_value(Self::fix_id_in_value(v, id))?),
            None => Ok(None),
        }
    }

    async fn store_skill(&self, skill: Skill) -> Result<()> {
        let value = serde_json::to_value(&skill)?;
        let _: Option<Value> = self
            .db
            .upsert(("skill", skill.id.to_string()))
            .content(value)
            .await?;
        Ok(())
    }

    async fn load_skill(&self, id: SkillId) -> Result<Option<Skill>> {
        let opt: Option<Value> = self.db.select(("skill", id.to_string())).await?;
        match opt {
            Some(v) => Ok(serde_json::from_value(Self::fix_id_in_value(v, id))?),
            None => Ok(None),
        }
    }

    async fn store_inference_config(&self, config: InferenceConfig) -> Result<()> {
        let value = serde_json::to_value(&config)?;
        let _: Option<Value> = self
            .db
            .upsert(("inference_config", "global"))
            .content(value)
            .await?;
        Ok(())
    }

    async fn load_inference_config(&self) -> Result<Option<InferenceConfig>> {
        let opt: Option<Value> = self.db.select(("inference_config", "global")).await?;
        match opt {
            Some(v) => Ok(serde_json::from_value(v)?),
            None => Ok(None),
        }
    }
}

/// Ethereal memory (in-memory, high-performance concurrent map)
pub struct EtherealMemory {
    sessions: dashmap::DashMap<SessionId, Session>,
    tasks: dashmap::DashMap<TaskId, Task>,
    checkpoints: dashmap::DashMap<CheckpointId, Checkpoint>,
    skills: dashmap::DashMap<SkillId, Skill>,
    inference_config: dashmap::DashMap<String, InferenceConfig>, // singleton config
}

impl Default for EtherealMemory {
    fn default() -> Self {
        Self {
            sessions: dashmap::DashMap::new(),
            tasks: dashmap::DashMap::new(),
            checkpoints: dashmap::DashMap::new(),
            skills: dashmap::DashMap::new(),
            inference_config: dashmap::DashMap::new(),
        }
    }
}

impl EtherealMemory {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl MemoryLayer for EtherealMemory {
    async fn init(&self) -> Result<()> {
        Ok(())
    }

    async fn store_session(&self, session: Session) -> Result<()> {
        self.sessions.insert(session.id, session);
        Ok(())
    }

    async fn load_session(&self, id: SessionId) -> Result<Option<Session>> {
        Ok(self.sessions.get(&id).map(|r| r.value().clone()))
    }

    async fn store_task(&self, task: Task) -> Result<()> {
        self.tasks.insert(task.id, task);
        Ok(())
    }

    async fn load_task(&self, id: TaskId) -> Result<Option<Task>> {
        Ok(self.tasks.get(&id).map(|r| r.value().clone()))
    }

    async fn store_checkpoint(&self, checkpoint: Checkpoint) -> Result<()> {
        self.checkpoints.insert(checkpoint.id, checkpoint);
        Ok(())
    }

    async fn load_checkpoint(&self, id: CheckpointId) -> Result<Option<Checkpoint>> {
        Ok(self.checkpoints.get(&id).map(|r| r.value().clone()))
    }

    async fn store_skill(&self, skill: Skill) -> Result<()> {
        self.skills.insert(skill.id, skill);
        Ok(())
    }

    async fn load_skill(&self, id: SkillId) -> Result<Option<Skill>> {
        Ok(self.skills.get(&id).map(|r| r.value().clone()))
    }

    async fn store_inference_config(&self, config: InferenceConfig) -> Result<()> {
        self.inference_config.insert("global".to_string(), config);
        Ok(())
    }

    async fn load_inference_config(&self) -> Result<Option<InferenceConfig>> {
        Ok(self
            .inference_config
            .get("global")
            .map(|r| r.value().clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use merix_schemas::Uuid;

    #[tokio::test]
    async fn test_persistent_memory_basic() {
        let db_path = std::env::temp_dir().join("merix_integration_test.db");
        let persistent = PersistentMemory::new_at_path(db_path.as_path())
            .await
            .unwrap();
        persistent.init().await.unwrap();
    }

    #[tokio::test]
    async fn test_ethereal_memory_basic() {
        let mem = EtherealMemory::new();
        let session_id = Uuid::new_v4();
        let session = Session {
            id: session_id,
            title: Some("Test Session".to_string()),
            description: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        mem.store_session(session.clone()).await.unwrap();
        let loaded = mem.load_session(session_id).await.unwrap().unwrap();
        assert_eq!(loaded.id, session_id);
    }
}
