use anyhow::Result;
use dashmap::DashMap;
use merix_schemas::{Session, SessionId, Checkpoint, TaskId, Skill};
use serde_json::Value;
use std::path::Path;
use surrealdb::engine::local::RocksDb;
use surrealdb::Surreal;
use tokio::fs;
use tracing::info;
use chrono::{DateTime, Utc};

// ===================================================================
// SEPARATED PERSISTENT MEMORY (SurrealDB only)
// ===================================================================
type Db = RocksDb; // Alias for clean trait bounds in SurrealDB 3.0

#[derive(Debug)]
pub struct PersistentMemory {
    db: Surreal<Db>,
}

impl PersistentMemory {
    pub async fn new(storage_path: &str) -> Result<Self> {
        let db_path = Path::new(storage_path).join("memory").join("merix.db");
        fs::create_dir_all(db_path.parent().unwrap()).await?;

        let db = Surreal::new::<Db>(db_path.to_str().unwrap()).await?;
        db.use_ns("merix").use_db("main").await?;

        // Define all PHASE 1 tables + indexes (Tooling DB, MCP registry, Skills, vector index)
        db.query(
            r#"
            DEFINE TABLE session SCHEMAFULL;
            DEFINE TABLE checkpoint SCHEMAFULL;
            DEFINE TABLE project SCHEMAFULL;
            DEFINE TABLE tooling SCHEMAFULL;
            DEFINE TABLE mcp_tools SCHEMAFULL;
            DEFINE TABLE skill SCHEMAFULL;

            DEFINE INDEX session_idx ON session FIELDS id UNIQUE;
            DEFINE INDEX checkpoint_task_idx ON checkpoint FIELDS task_id;
            DEFINE INDEX project_idx ON project FIELDS id;
            DEFINE INDEX tooling_idx ON tooling FIELDS name;
            DEFINE INDEX mcp_idx ON mcp_tools FIELDS name;
            DEFINE INDEX skill_idx ON skill FIELDS name;

            DEFINE INDEX vec_idx ON project FIELDS embedding TYPE vector DIMENSION 384 DIST euclidean;
            "#,
        )
        .await?;

        info!("PersistentMemory (SurrealDB 3.0 RocksDB) initialized with Tooling/MCP/Skill tables + vector index");
        Ok(Self { db })
    }

    // Session persistence
    pub async fn save_session(&self, session: &Session) -> Result<()> {
        let value = serde_json::to_value(session)?;
        let _: Option<Value> = self.db.create(("session", session.id.0.to_string()))
            .content(value)
            .await?;
        info!("Session {} persisted", session.id.0);
        Ok(())
    }

    pub async fn load_session(&self, id: SessionId) -> Result<Option<Session>> {
        let res: Option<Session> = self.db.select(("session", id.0.to_string())).await?;
        Ok(res)
    }

    // Checkpoint (resumable)
    pub async fn save_checkpoint(&self, checkpoint: &Checkpoint) -> Result<()> {
        let value = serde_json::to_value(checkpoint)?;
        let _: Option<Value> = self.db.create(("checkpoint", checkpoint.id.0.to_string()))
            .content(value)
            .await?;
        Ok(())
    }

    pub async fn load_latest_checkpoint(&self, task_id: TaskId) -> Result<Option<Checkpoint>> {
        let mut res = self.db.query("SELECT * FROM checkpoint WHERE task_id = $task_id ORDER BY timestamp DESC LIMIT 1")
            .bind(("task_id", task_id))
            .await?
            .take::<Vec<Checkpoint>>(0)?;
        Ok(res.pop())
    }

    // Project memory + vector
    pub async fn store_project_memory(&self, id: String, description: String, data: Value) -> Result<()> {
        let embedding: Vec<f32> = vec![0.1; 384]; // mock for PHASE 1
        let _: Option<Value> = self.db.create(("project", id))
            .content(serde_json::json!({
                "description": description,
                "data": data,
                "embedding": embedding,
                "timestamp": Utc::now()
            }))
            .await?;
        info!("Project memory stored: {}", description);
        Ok(())
    }

    // MCP registry
    pub async fn install_mcp_tool(
        &self,
        name: String,
        description: String,
        mcp_json: Value,
        permission: String,
    ) -> Result<()> {
        let _: Option<Value> = self.db.upsert(("mcp_tools", &name))
            .content(serde_json::json!({
                "name": name,
                "description": description,
                "mcp_json": mcp_json,
                "permission": permission,
                "installed_at": Utc::now()
            }))
            .await?;
        info!("MCP tool '{}' installed (permission: {})", name, permission);
        Ok(())
    }

    pub async fn register_hello_world_mcp(&self) -> Result<()> {
        let mcp_json = serde_json::json!({
            "name": "hello_world",
            "description": "Simple greeting from Merix MCP",
            "input_schema": { "type": "object", "properties": { "name": { "type": "string", "default": "World" } } }
        });
        self.install_mcp_tool("hello_world".into(), "Hello World MCP tool example".into(), mcp_json, "always".into()).await
    }

    // Skills
    pub async fn save_skill(&self, skill: &Skill) -> Result<()> {
        let value = serde_json::to_value(skill)?;
        let _: Option<Value> = self.db.create(("skill", skill.id.to_string()))
            .content(value)
            .await?;
        info!("Skill '{}' (v{}) saved", skill.name, skill.version);
        Ok(())
    }

    pub async fn list_skills(&self) -> Result<Vec<Skill>> {
        let skills: Vec<Skill> = self.db.select("skill").await?;
        Ok(skills)
    }

    // Context reconstruction (uses persistent + ephemeral later)
    pub async fn reconstruct_context(&self, session_id: Option<SessionId>) -> Result<String> {
        let mut context = String::new();
        if let Some(sid) = session_id {
            if let Some(session) = self.load_session(sid).await? {
                context.push_str(&format!("Session {}: {} tasks\n", session.id.0, session.tasks.len()));
            }
        }
        Ok(context)
    }
}

// ===================================================================
// SEPARATED ETHEREAL MEMORY (Dashmap only)
// ===================================================================
#[derive(Debug)]
pub struct EtherealMemory {
    short_term: DashMap<String, Value>,
}

impl EtherealMemory {
    pub fn new() -> Self {
        Self { short_term: DashMap::new() }
    }

    pub fn store(&self, key: String, value: Value) {
        self.short_term.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<Value> {
        self.short_term.get(key).map(|v| v.value().clone())
    }
}

// ===================================================================
// PUBLIC MEMORY LAYER (composes both)
// ===================================================================
#[derive(Debug)]
pub struct MemoryLayer {
    pub persistent: PersistentMemory,
    pub ethereal: EtherealMemory,
}

impl MemoryLayer {
    pub async fn new(storage_path: &str) -> Result<Self> {
        Ok(Self {
            persistent: PersistentMemory::new(storage_path).await?,
            ethereal: EtherealMemory::new(),
        })
    }

    // Convenience wrappers
    pub async fn save_session(&self, session: &Session) -> Result<()> {
        self.persistent.save_session(session).await
    }

    pub async fn load_session(&self, id: SessionId) -> Result<Option<Session>> {
        self.persistent.load_session(id).await
    }

    pub async fn save_checkpoint(&self, checkpoint: &Checkpoint) -> Result<()> {
        self.persistent.save_checkpoint(checkpoint).await
    }

    pub async fn load_latest_checkpoint(&self, task_id: TaskId) -> Result<Option<Checkpoint>> {
        self.persistent.load_latest_checkpoint(task_id).await
    }

    pub async fn store_project_memory(&self, id: String, description: String, data: Value) -> Result<()> {
        self.persistent.store_project_memory(id, description, data).await
    }

    pub async fn install_mcp_tool(&self, name: String, description: String, mcp_json: Value, permission: String) -> Result<()> {
        self.persistent.install_mcp_tool(name, description, mcp_json, permission).await
    }

    pub async fn register_hello_world_mcp(&self) -> Result<()> {
        self.persistent.register_hello_world_mcp().await
    }

    pub async fn save_skill(&self, skill: &Skill) -> Result<()> {
        self.persistent.save_skill(skill).await
    }

    pub async fn list_skills(&self) -> Result<Vec<Skill>> {
        self.persistent.list_skills().await
    }

    pub async fn reconstruct_context(&self, session_id: Option<SessionId>) -> Result<String> {
        let mut ctx = self.persistent.reconstruct_context(session_id).await?;
        // Add ephemeral data
        for entry in self.ethereal.short_term.iter() {
            ctx.push_str(&format!("Ephemeral[{}]: {}\n", entry.key(), entry.value()));
        }
        Ok(ctx)
    }

    // Ethereal convenience
    pub fn store_ephemeral(&self, key: String, value: Value) {
        self.ethereal.store(key, value);
    }

    pub fn get_ephemeral(&self, key: &str) -> Option<Value> {
        self.ethereal.get(key)
    }
}