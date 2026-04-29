use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct StepId(pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StepStatus {
    Pending,
    Running,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    pub id: StepId,
    pub description: String,
    pub status: StepStatus,
    pub output: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed(String),
    PausedAtCheckpoint,
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
        let id = TaskId(Uuid::new_v4());
        let now = Utc::now();
        Self {
            id,
            description,
            status: TaskStatus::Pending,
            steps: vec![],
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_step(&mut self, description: String) {
        let step = Step {
            id: StepId(Uuid::new_v4()),
            description,
            status: StepStatus::Pending,
            output: None,
        };
        self.steps.push(step);
        self.updated_at = Utc::now();
    }
}
