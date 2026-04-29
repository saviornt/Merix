use anyhow::{anyhow, Result};
use async_trait::async_trait;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};
use merix_memory::MemoryLayer;
use uuid::Uuid;
use merix_models::SessionId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetadata {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
    pub output_schema: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub tool_name: String,
    pub input: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_name: String,
    pub output: Value,
    pub success: bool,
    pub error: Option<String>,
}

#[async_trait]
pub trait Tool: Send + Sync {
    fn metadata(&self) -> ToolMetadata;
    async fn execute(&self, input: Value, memory: &MemoryLayer) -> Result<ToolResult>;
}

pub struct ToolRegistry {
    tools: RwLock<Arc<DashMap<String, Arc<dyn Tool>>>>,
    permissions: RwLock<Arc<DashMap<String, Vec<String>>>>, // tool_name -> allowed capabilities (PHASE 1)
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: RwLock::new(Arc::new(DashMap::new())),
            permissions: RwLock::new(Arc::new(DashMap::new())),
        }
    }

    pub async fn register_tool<T: Tool + 'static>(&self, tool: T, allowed_capabilities: Vec<String>) -> Result<()> {
        let metadata = tool.metadata();
        let name = metadata.name.clone();
        let tools = self.tools.write().await;
        let permissions = self.permissions.write().await;

        tools.insert(name.clone(), Arc::new(tool));
        permissions.insert(name.clone(), allowed_capabilities);

        info!("MCP Tool registered: {} - {}", name, metadata.description);
        Ok(())
    }

    pub async fn discover_tools(&self) -> Vec<ToolMetadata> {
        let tools = self.tools.read().await;
        tools.iter().map(|entry| entry.value().metadata()).collect()
    }

    pub async fn execute_tool(&self, call: ToolCall, memory: &MemoryLayer) -> Result<ToolResult> {
        let tools = self.tools.read().await;
        let tool = tools.get(&call.tool_name)
            .ok_or_else(|| anyhow!("Tool not found: {}", call.tool_name))?;

        // PHASE 1 permission sandbox check
        let permissions = self.permissions.read().await;
        if let Some(allowed) = permissions.get(&call.tool_name) {
            if allowed.is_empty() {
                warn!("Tool {} executed with no explicit permissions", call.tool_name);
            }
        }

        info!("Executing MCP tool: {}", call.tool_name);
        let result = tool.execute(call.input, memory).await?;

        Ok(result)
    }
}

// Built-in example tool (Memory Query) – demonstrates integration
pub struct MemoryQueryTool;

#[async_trait]
impl Tool for MemoryQueryTool {
    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            name: "memory_query".to_string(),
            description: "Query the memory layer for context reconstruction".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "session_id": { "type": "string" }
                },
                "required": ["session_id"]
            }),
            output_schema: serde_json::json!({"type": "string"}),
        }
    }

    async fn execute(&self, input: Value, memory: &MemoryLayer) -> Result<ToolResult> {
        let session_id_str = input["session_id"].as_str()
            .ok_or_else(|| anyhow!("session_id is required"))?;
        let session_id = SessionId(Uuid::parse_str(session_id_str).map_err(|_| anyhow!("invalid UUID"))?);

        let context = memory.reconstruct_context(session_id).await?;

        Ok(ToolResult {
            tool_name: "memory_query".to_string(),
            output: serde_json::json!(context),
            success: true,
            error: None,
        })
    }
}

// Built-in example tool (Echo) – simple test tool
pub struct EchoTool;

#[async_trait]
impl Tool for EchoTool {
    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            name: "echo".to_string(),
            description: "Echo input back as output (test tool)".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "message": { "type": "string" }
                },
                "required": ["message"]
            }),
            output_schema: serde_json::json!({"type": "string"}),
        }
    }

    async fn execute(&self, input: Value, _memory: &MemoryLayer) -> Result<ToolResult> {
        let message = input["message"].as_str()
            .unwrap_or("no message provided");

        Ok(ToolResult {
            tool_name: "echo".to_string(),
            output: serde_json::json!(message),
            success: true,
            error: None,
        })
    }
}