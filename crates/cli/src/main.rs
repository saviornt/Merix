use anyhow::Result;
use clap::{Parser, Subcommand};
use std::sync::Arc;
use tokio::runtime::Runtime;
use tracing::{info, warn, Level};
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
#[command(about = "MerixAI - Local-first self-extending AI runtime (PHASE 1 MVP)", long_about = None)]
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

    /// Trigger self-extension on a session (use "auto" for a new session)
    SelfExtend {
        #[arg(long, required = true)]
        session_id: String,
    },

    /// Show system status
    Status,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    let rt = Runtime::new()?;
    rt.block_on(async {
        let storage_path = "data";
        std::fs::create_dir_all(storage_path)?;

        let memory = Arc::new(MemoryLayer::new(storage_path).await?);
        let tool_registry = Arc::new(ToolRegistry::new());
        let skill_registry = Arc::new(SkillRegistry::new());
        let task_executor = TaskExecutor::new(storage_path);
        let self_extension = Arc::new(SelfExtensionCore::new(skill_registry.clone(), memory.clone()));

        tool_registry.register_tool(EchoTool, vec!["test".to_string()]).await?;
        tool_registry.register_tool(MemoryQueryTool, vec!["memory".to_string()]).await?;
        skill_registry.register_skill(ContextAnalyzerSkill).await?;

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
                let mut session = merix_models::Session::new();
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

            Commands::SelfExtend { session_id: input_id } => {
                let session_id = if input_id.eq_ignore_ascii_case("auto") {
                    let new_session = merix_models::Session::new();
                    memory.save_session(&new_session).await?;
                    info!("Created new session for self-extension: {}", new_session.id.0);
                    new_session.id
                } else {
                    match Uuid::parse_str(&input_id) {
                        Ok(u) => SessionId(u),
                        Err(_) => {
                            warn!("Invalid session ID: {}. Creating new session instead.", input_id);
                            let new_session = merix_models::Session::new();
                            memory.save_session(&new_session).await?;
                            info!("Created new session: {}", new_session.id.0);
                            new_session.id
                        }
                    }
                };

                info!("Running self-extension on session {}", session_id.0);
                self_extension.run_self_extension(session_id).await?;
                info!("Self-extension complete");
            }

            Commands::Status => {
                println!("MerixAI PHASE 1 MVP Status");
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