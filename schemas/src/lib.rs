use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SessionId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TaskId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CheckpointId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Paused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StepStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

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

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckpointRecord {
    pub task_id: String,
    pub session_id: String,
    pub timestamp: DateTime<Utc>,
    pub state_snapshot: serde_json::Value,
}

impl From<&Checkpoint> for CheckpointRecord {
    fn from(c: &Checkpoint) -> Self {
        Self {
            // We convert IDs to strings here to keep SurrealDB happy
            task_id: c.task_id.0.to_string(),
            session_id: c.session_id.0.to_string(),
            timestamp: c.timestamp,
            state_snapshot: c.state_snapshot.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionRecord {
    pub created_at: DateTime<Utc>,
    pub tasks: Vec<Task>,
    pub current_task: Option<String>,
}

impl From<&Session> for SessionRecord {
    fn from(s: &Session) -> Self {
        Self {
            created_at: s.created_at,
            tasks: s.tasks.clone(),
            current_task: s.current_task.map(|id| id.0.to_string()),
        }
    }
}

impl CheckpointRecord {
    pub fn into_checkpoint(self, id: Uuid) -> Checkpoint {
        Checkpoint {
            id: CheckpointId(id),
            task_id: TaskId(Uuid::parse_str(&self.task_id).unwrap_or_default()),
            session_id: SessionId(Uuid::parse_str(&self.session_id).unwrap_or_default()),
            timestamp: self.timestamp,
            state_snapshot: self.state_snapshot,
        }
    }
}