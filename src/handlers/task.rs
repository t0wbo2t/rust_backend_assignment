use axum::{
    extract::State,
    http::StatusCode,
    Extension,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::state::AppState;
use crate::models::{User, Status, Priority};
use crate::services::task::{TaskService, TaskError};

#[derive(Deserialize)]
pub struct CreateTaskRequest {
    pub title: String,
    pub description: String,
    pub status: Status,
    pub priority: Priority,
    pub assigned_to_id: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct AssignTasksRequest {
    pub task_ids: Vec<Uuid>,
    pub user_id: Uuid,
}

fn map_task_error(err: TaskError) -> (StatusCode, String) {
    match err {
        TaskError::PermissionDenied => (StatusCode::FORBIDDEN, "Permission denied: Admin role required".to_string()),
        TaskError::UserNotFound => (StatusCode::NOT_FOUND, "Assigned user not found".to_string()),
        TaskError::Database(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {e}")),
    }
}

pub async fn create_task(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, String)> {
    let task = TaskService::create_task(
        &state.database,
        &user,
        payload.title,
        payload.description,
        payload.status,
        payload.priority,
        payload.assigned_to_id,
    )
    .await
    .map_err(map_task_error)?;

    Ok((StatusCode::CREATED, Json(json!(task))))
}

pub async fn assign_tasks(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(payload): Json<AssignTasksRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, String)> {
    TaskService::assign_tasks(
        &state.database,
        &user,
        payload.task_ids,
        payload.user_id,
        &state.task_response_cache,
    )
    .await
    .map_err(map_task_error)?;

    Ok((
        StatusCode::OK,
        Json(json!({ "message": "Tasks assigned successfully" })),
    ))
}

pub async fn view_my_tasks(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<(StatusCode, Json<crate::models::ViewTasksResponse>), (StatusCode, String)> {
    let response = TaskService::get_my_tasks(
        &state.database,
        &user,
        &state.task_response_cache,
    )
    .await
    .map_err(map_task_error)?;

    Ok((StatusCode::OK, Json(response)))
}
