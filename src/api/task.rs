use std::result;

use crate::model::task::Task;
use crate::model::task::TaskState;
use crate::repository::ddb::DDBRepository;

use actix_web::{
    get,
    post,
    put,
    error::ResponseError,
    web::{ Data, Json, Path },
    HttpResponse,
    http::{ header::ContentType, StatusCode }
};

use serde::{ Serialize, Deserialize };
use derive_more::{ Display };

pub struct TaskIdentifier {
    task_global_id: String
}

#[derive(Deserialize)]
pub struct TaskCompletionRequest {
    result_file: String
}

#[derive(Deserialize)]
pub struct SubmitTaskRequest {
    user_id: String,
    task_type: String,
    source_file: String
}

#[derive(Debug, Display)]
pub enum TaskError {
    TaskNotFound,
    TaskUpdateFailure,
    TaskCreationFailure,
    BadTaskRequest
}

impl ResponseError for TaskError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match self {
            TaskError::TaskNotFound => StatusCode::NOT_FOUND,
            TaskError::TaskUpdateFailure => StatusCode::INTERNAL_SERVER_ERROR,
            TaskError::TaskCreationFailure => StatusCode::INTERNAL_SERVER_ERROR,
            TaskError::BadTaskRequest => StatusCode::BAD_REQUEST
        }
    }
}

#[get("/task/{task_global_id}")]
pub async fn get_task(
    task_identifier: Path<TaskIdentifier>, 
    ddb_repo: Data<DDBRepository>
) -> Result<Json<Task>, TaskError> {
    let task = ddb_repo.get_task(
        task_identifier.into_inner().task_global_id       
    ).await;

    match task {
        Some(task) => Ok(Json(task)),
        None => Err(TaskError::TaskNotFound)
    }
}

async fn state_transition(
    ddb_repo: Data<DDBRepository>,
    task_global_id: String,
    new_state: TaskState,
    result_file: Option<String>
) -> Result<Json<TaskIdentifier>, TaskError> {
    let mut task = match ddb_repo.get_task(
        task_global_id
    ).await {
        Some(task) => task,
        None => return Err(TaskError::TaskNotFound)
    };

    if !task.can_transition_to(&new_state) {
        return Err(TaskError::BadTaskRequest);
    }

    task.state = new_state;
    task.result = result_file;

    let task_identifier = task.get_global_id();
    match ddb_repo.put_task(task).await {
        Ok(()) => Ok(Json(TaskIdentifier { task_global_id: task_identifier })),
        Err(_) => Err(TaskError::TaskUpdateFailure)
    }
}