use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Admin,
    Staff,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub full_name: String,
    pub email: String,
    pub hashed_password: String,
    pub role: Role,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LoginChallenge {
    pub id: Uuid,
    pub user_id: Uuid,
    pub hashed_code: String,
    pub expires_at: DateTime<Utc>,
    pub used: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EmailLog {
    pub id: Uuid,
    pub recipient: String,
    pub code: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Todo,
    InProgress,
    Done,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub status: Status,
    pub priority: Priority,
    pub created_by_id: Uuid,
    pub assigned_to_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ResponseTask {
    pub id: Uuid,
    pub title: String,
    pub status: Status,
    pub priority: Priority,
    pub assigned_to: Option<String>, // email of assigned user
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSummary {
    pub email: String,
    pub role: Role,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TasksSummary {
    pub total_assigned_tasks: usize,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CacheSummary {
    pub hit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewTasksResponse {
    pub user: UserSummary,
    pub tasks: Vec<ResponseTask>,
    pub summary: TasksSummary,
    pub cache: CacheSummary,
}
