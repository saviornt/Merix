use anyhow::Result;
use merix_schemas::{Checkpoint, CheckpointId, Session, SessionId, Skill, SkillId, Task, TaskId};
use serde_json::Value;
use std::path::Path;
use surrealdb::engine::local::{Db, RocksDb};
use surrealdb::Surreal;
use async_trait::async_trait;

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
}

/// Persistent memory (SurrealDB + RocksDB — disk-backed, resumable)
pub struct PersistentMemory {
    db: Surreal<Db>,
}

impl PersistentMemory {
    /// Creates a new PersistentMemory using the configured data directory by default
    pub async fn new() -> Result<Self> {
        let data_dir = merix_utilities::config::MerixConfig::get_data_directory();
        let db_path = data_dir.join("merix.db");
        let db = Surreal::new::<RocksDb>(db_path).await?;
        Ok(Self { db })
    }

    /// Creates a new PersistentMemory at a custom path (for tests)
    pub async fn new_at_path(db_path: impl AsRef<Path>) -> Result<Self> {
        let db = Surreal::new::<RocksDb>(db_path.as_ref()).await?;
        Ok(Self { db })
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
        let _: Option<Value> = self.db
            .upsert(("session", session.id.to_string()))
            .content(value)
            .await?;
        Ok(())
    }

    async fn load_session(&self, id: SessionId) -> Result<Option<Session>> {
        let opt: Option<Value> = self.db.select(("session", id.to_string())).await?;
        match opt {
            Some(mut v) => {
                // SurrealDB returns RecordId strings like "session:xxx". Restore the original Uuid.
                if let Some(id_val) = v.get_mut("id") {
                    *id_val = serde_json::Value::String(id.to_string());
                }
                Ok(serde_json::from_value(v)?)
            }
            None => Ok(None),
        }
    }

    async fn store_task(&self, task: Task) -> Result<()> {
        let value = serde_json::to_value(&task)?;
        let _: Option<Value> = self.db
            .upsert(("task", task.id.to_string()))
            .content(value)
            .await?;
        Ok(())
    }

    async fn load_task(&self, id: TaskId) -> Result<Option<Task>> {
        let opt: Option<Value> = self.db.select(("task", id.to_string())).await?;
        match opt {
            Some(mut v) => {
                if let Some(id_val) = v.get_mut("id") {
                    *id_val = serde_json::Value::String(id.to_string());
                }
                Ok(serde_json::from_value(v)?)
            }
            None => Ok(None),
        }
    }

    async fn store_checkpoint(&self, checkpoint: Checkpoint) -> Result<()> {
        let value = serde_json::to_value(&checkpoint)?;
        let _: Option<Value> = self.db
            .upsert(("checkpoint", checkpoint.id.to_string()))
            .content(value)
            .await?;
        Ok(())
    }

    async fn load_checkpoint(&self, id: CheckpointId) -> Result<Option<Checkpoint>> {
        let opt: Option<Value> = self.db.select(("checkpoint", id.to_string())).await?;
        match opt {
            Some(mut v) => {
                if let Some(id_val) = v.get_mut("id") {
                    *id_val = serde_json::Value::String(id.to_string());
                }
                Ok(serde_json::from_value(v)?)
            }
            None => Ok(None),
        }
    }

    async fn store_skill(&self, skill: Skill) -> Result<()> {
        let value = serde_json::to_value(&skill)?;
        let _: Option<Value> = self.db
            .upsert(("skill", skill.id.to_string()))
            .content(value)
            .await?;
        Ok(())
    }

    async fn load_skill(&self, id: SkillId) -> Result<Option<Skill>> {
        let opt: Option<Value> = self.db.select(("skill", id.to_string())).await?;
        match opt {
            Some(mut v) => {
                if let Some(id_val) = v.get_mut("id") {
                    *id_val = serde_json::Value::String(id.to_string());
                }
                Ok(serde_json::from_value(v)?)
            }
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
}

impl Default for EtherealMemory {
    fn default() -> Self {
        Self {
            sessions: dashmap::DashMap::new(),
            tasks: dashmap::DashMap::new(),
            checkpoints: dashmap::DashMap::new(),
            skills: dashmap::DashMap::new(),
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use merix_schemas::Uuid;
    use chrono::Utc;

    #[tokio::test]
    async fn test_persistent_memory_basic() {
        let db_path = std::env::temp_dir().join("merix_integration_test.db");
        let persistent = PersistentMemory::new_at_path(db_path.as_path()).await.unwrap();
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
        let loaded = mem.load_session(session_id).await.unwrap();
        assert_eq!(loaded, Some(session));
    }
}




