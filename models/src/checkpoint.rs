use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::task::TaskStatus;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckpointId(pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: CheckpointId,
    pub task_id: Uuid,
    pub session_id: Uuid,
    pub step_index: usize,
    pub task_status: TaskStatus,
    pub timestamp: DateTime<Utc>,
    pub serialized_state: String,
}

impl Checkpoint {
    pub fn new(task_id: Uuid, session_id: Uuid, step_index: usize, task_status: TaskStatus, serialized_state: String) -> Self {
        Self {
            id: CheckpointId(Uuid::new_v4()),
            task_id,
            session_id,
            step_index,
            task_status,
            timestamp: Utc::now(),
            serialized_state,
        }
    }
}
