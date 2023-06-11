use serde::Serialize;
use uuid::Uuid;
use strum_macros::{ Display, EnumString };

pub enum TaskState {
    NotStarted,
    InProgress,
    Completed,
    Paused,
    Failed
}

#[derive(Serialize)]
pub struct Task {
    pub user_uuid: String,
    pub task_uuid: String,
    pub task_type: String,
    pub state: TaskState,
    pub source: String,
    pub result: Option<String>
}

impl Task {
    pub fn new(user_uuid: String, task_type: String, soure_file: String) -> Task {
        Task {
            user_uuid,
            task_uuid: Uuid::new_v4().to_string(),
            task_type,
            state: TaskState::NotStarted,
            soure_file,
            result_file: None
        }
    }
    
    pub fn get_global_id(&self) -> String {
        format!("{}_{}", self.user_uuid, self.task_uuid)
    }

    pub fn can_transition_to(&self, state: &TaskState) -> bool {
        self.state != *state
    }
} 