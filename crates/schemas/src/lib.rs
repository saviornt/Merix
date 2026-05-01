use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

// Re-export Uuid so other crates can use it directly
pub use uuid::Uuid;

/// Unique identifier for a session
pub type SessionId = Uuid;

/// Unique identifier for a task
pub type TaskId = Uuid;

/// Unique identifier for a checkpoint
pub type CheckpointId = Uuid;

/// Unique identifier for a skill
pub type SkillId = Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskStatus {
    /// Task is queued but not started
    Pending,
    /// Task is currently executing
    Running,
    /// Task completed successfully
    Completed,
    /// Task failed
    Failed,
    /// Task is paused
    Paused,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Session {
    pub id: SessionId,
    pub title: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Task {
    pub id: TaskId,
    pub session_id: SessionId,
    pub description: String,
    pub status: TaskStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// Optional parent task for sub-tasks
    pub parent_id: Option<TaskId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Checkpoint {
    pub id: CheckpointId,
    pub task_id: TaskId,
    pub sequence: u64,
    pub state: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Skill {
    pub id: SkillId,
    pub name: String,
    pub description: String,
    pub version: String,
    pub created_at: DateTime<Utc>,
}
