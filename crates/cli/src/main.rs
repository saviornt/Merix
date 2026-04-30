use anyhow::Result;
use clap::{Parser, Subcommand};
use std::sync::Arc;
use tokio::runtime::Runtime;
use tracing::{info, Level};
use tracing_subscriber;
use merix_core::TaskExecutor;
use merix_memory::MemoryLayer;
use merix_mcp::{ToolRegistry, EchoTool, MemoryQueryTool};
use merix_skills::{SkillRegistry, ContextAnalyzerSkill};
use merix_self_extension::SelfExtensionCore;
use merix_models::SessionId;
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "merix")]
#[command(about = "Merix - Local-first self-extending AI runtime (PHASE 1 MVP)", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create and run a new task
    Task {
        #[arg(required = true)]
        description: String,
    },

    /// Resume the last task from checkpoint
    Resume,

    /// List all registered skills
    SkillList,

    /// List all available MCP tools
    ToolList,

    /// Trigger self-extension on a session
    SelfExtend {
        #[arg(long, required = true)]
        session_id: String,
    },

    /// Show system status (memory, registries)
    Status,
}

fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    let rt = Runtime::new()?;
    rt.block_on(async {
        // Initialize all PHASE 1 components
        let storage_path = "data";
        std::fs::create_dir_all(storage_path)?;

        let memory = Arc::new(MemoryLayer::new(storage_path).await?);
        let tool_registry = Arc::new(ToolRegistry::new());
        let skill_registry = Arc::new(SkillRegistry::new());
        let task_executor = TaskExecutor::new(storage_path);
        let self_extension = Arc::new(SelfExtensionCore::new(skill_registry.clone(), memory.clone()));

        // Register built-in tools
        tool_registry.register_tool(EchoTool, vec!["test".to_string()]).await?;
        tool_registry.register_tool(MemoryQueryTool, vec!["memory".to_string()]).await?;

        // Register built-in skills
        skill_registry.register_skill(ContextAnalyzerSkill).await?;

        // Parse CLI
        let cli = Cli::parse();

        match cli.command {
            Commands::Task { description } => {
                info!("Creating new task: {}", description);
                let mut session = merix_models::Session::new();
                task_executor.execute_task(&mut session, description).await?;
                memory.save_session(&session).await?;
                info!("Task completed. Session ID: {}", session.id.0);
            }

            Commands::Resume => {
                info!("Resuming last task...");
                let mut session = merix_models::Session::new(); // In full CLI this would load from disk
                task_executor.resume_task(&mut session).await?;
                info!("Resume complete");
            }

            Commands::SkillList => {
                let skills = skill_registry.list_skills().await;
                println!("Registered Skills ({}):", skills.len());
                for skill in skills {
                    println!("  • {} v{} - {}", skill.name, skill.version, skill.description);
                }
            }

            Commands::ToolList => {
                let tools = tool_registry.discover_tools().await;
                println!("Available MCP Tools ({}):", tools.len());
                for tool in tools {
                    println!("  • {} - {}", tool.name, tool.description);
                }
            }

            Commands::SelfExtend { session_id } => {
                let session_uuid = Uuid::parse_str(&session_id)?;
                let session_id = SessionId(session_uuid);
                info!("Running self-extension on session {}", session_id.0);
                self_extension.run_self_extension(session_id).await?;
                info!("Self-extension complete");
            }

            Commands::Status => {
                println!("Merix PHASE 1 MVP Status");
                println!("  Memory Layer: initialized");
                println!("  MCP Tools: registered");
                println!("  Skills: registered");
                println!("  Self-Extension: ready");
                println!("\nRun `merix --help` for full commands");
            }
        }

        Ok(())
    })
}