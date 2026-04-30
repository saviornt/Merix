use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use merix_mcp::{ToolCall, ToolRegistry};
use merix_memory::MemoryLayer;
use merix_schemas::SessionId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub capabilities: Vec<String>, // e.g. "memory_read", "tool_call"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillManifest {
    pub metadata: SkillMetadata,
    pub entrypoint: String, // for future dynamic loading
}

#[async_trait]
pub trait Skill: Send + Sync {
    fn metadata(&self) -> SkillMetadata;
    async fn execute(
        &self,
        _input: serde_json::Value,          // prefixed with _ to silence unused-variable warning
        memory: &MemoryLayer,
        tools: &ToolRegistry,
        session_id: Option<SessionId>,
    ) -> Result<serde_json::Value>;
}

pub struct SkillRegistry {
    skills: RwLock<HashMap<String, Arc<dyn Skill>>>,
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self {
            skills: RwLock::new(HashMap::new()),
        }
    }

    pub async fn register_skill<S: Skill + 'static>(&self, skill: S) -> Result<()> {
        let metadata = skill.metadata();
        let name = metadata.name.clone();
        let mut skills = self.skills.write().await;
        skills.insert(name.clone(), Arc::new(skill));
        info!("Skill registered: {} v{} - {}", name, metadata.version, metadata.description);
        Ok(())
    }

    pub async fn list_skills(&self) -> Vec<SkillMetadata> {
        let skills = self.skills.read().await;
        skills.values().map(|s| s.metadata()).collect()
    }

    pub async fn execute_skill(
        &self,
        skill_name: &str,
        input: serde_json::Value,
        memory: &MemoryLayer,
        tools: &ToolRegistry,
        session_id: Option<SessionId>,
    ) -> Result<serde_json::Value> {
        let skills = self.skills.read().await;
        let skill = skills.get(skill_name)
            .ok_or_else(|| anyhow!("Skill not found: {}", skill_name))?;

        info!("Executing skill: {}", skill_name);
        skill.execute(input, memory, tools, session_id).await
    }

    // Runtime unload (for self-extension loop)
    pub async fn unload_skill(&self, skill_name: &str) -> Result<()> {
        let mut skills = self.skills.write().await;
        if skills.remove(skill_name).is_some() {
            info!("Skill unloaded: {}", skill_name);
            Ok(())
        } else {
            Err(anyhow!("Skill not found: {}", skill_name))
        }
    }
}

// Example built-in skill: "context_analyzer" – demonstrates chaining MCP tools + memory
pub struct ContextAnalyzerSkill;

#[async_trait]
impl Skill for ContextAnalyzerSkill {
    fn metadata(&self) -> SkillMetadata {
        SkillMetadata {
            name: "context_analyzer".to_string(),
            version: "0.1.0".to_string(),
            description: "Analyzes session context and suggests next steps using memory + tools".to_string(),
            author: "Merix Core".to_string(),
            capabilities: vec!["memory_read".to_string(), "tool_call".to_string()],
        }
    }

    async fn execute(
        &self,
        _input: serde_json::Value,
        memory: &MemoryLayer,
        tools: &ToolRegistry,
        session_id: Option<SessionId>,
    ) -> Result<serde_json::Value> {
        let session_id = session_id.ok_or_else(|| anyhow!("session_id required"))?;

        // Use memory to reconstruct context
        let context = memory.reconstruct_context(session_id).await?;

        // Call MCP tool for additional processing (example)
        let tool_call = ToolCall {
            tool_name: "echo".to_string(),
            input: serde_json::json!({ "message": format!("Context summary: {}", context.len()) }),
        };
        let tool_result = tools.execute_tool(tool_call, memory).await?;

        // Return structured output
        Ok(serde_json::json!({
            "context_length": context.len(),
            "tool_result": tool_result.output,
            "suggestion": "Task decomposition complete – ready for next step"
        }))
    }
}