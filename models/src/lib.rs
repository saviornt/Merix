use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Paused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionId(pub Uuid);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskId(pub Uuid);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckpointId(pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: SessionId,
    pub created_at: DateTime<Utc>,
    pub tasks: Vec<Task>,
    pub current_task: Option<TaskId>,
}

impl Session {
    pub fn new() -> Self {
        Self {
            id: SessionId(Uuid::new_v4()),
            created_at: Utc::now(),
            tasks: Vec::new(),
            current_task: None,
        }
    }

    pub fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub description: String,
    pub status: TaskStatus,
    pub steps: Vec<Step>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Task {
    pub fn new(description: String) -> Self {
        Self {
            id: TaskId(Uuid::new_v4()),
            description,
            status: TaskStatus::Pending,
            steps: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn add_step(&mut self, description: String) {
        self.steps.push(Step {
            description,
            status: StepStatus::Pending,
            output: None,
            checkpoint_id: None,
        });
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    pub description: String,
    pub status: StepStatus,
    pub output: Option<String>,
    pub checkpoint_id: Option<CheckpointId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: CheckpointId,
    pub task_id: TaskId,
    pub session_id: SessionId,
    pub timestamp: DateTime<Utc>,
    pub state_snapshot: serde_json::Value,
}

impl Checkpoint {
    pub fn new(task_id: TaskId, session_id: SessionId, state_snapshot: serde_json::Value) -> Self {
        Self {
            id: CheckpointId(Uuid::new_v4()),
            task_id,
            session_id,
            timestamp: Utc::now(),
            state_snapshot,
        }
    }
}