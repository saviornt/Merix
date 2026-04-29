use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::task::{Task, TaskId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionId(pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: SessionId,
    pub current_task: Option<TaskId>,
    pub tasks: Vec<Task>,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
}

impl Session {
    pub fn new() -> Self {
        Self {
            id: SessionId(Uuid::new_v4()),
            current_task: None,
            tasks: vec![],
            created_at: Utc::now(),
            last_active: Utc::now(),
        }
    }

    pub fn add_task(&mut self, task: Task) {
        self.current_task = Some(task.id);
        self.tasks.push(task);
        self.last_active = Utc::now();
    }
}
