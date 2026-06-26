use sqlx::SqlitePool;
use uuid::Uuid;
use chrono::Utc;

use crate::models::{Task, User, Role, Status, Priority};
use crate::repositories::task::TaskRepository;
use crate::repositories::user::UserRepository;
use crate::state::TaskResponseCache;

#[derive(Debug, thiserror::Error)]
pub enum TaskError {
    #[error("Permission denied: Admin role required")]
    PermissionDenied,
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Assigned user not found")]
    UserNotFound,
}

pub struct TaskService;

impl TaskService {
    pub async fn create_task(
        pool: &SqlitePool,
        creator: &User,
        title: String,
        description: String,
        status: Status,
        priority: Priority,
        assigned_to_id: Option<Uuid>,
    ) -> Result<Task, TaskError> {
        if creator.role != Role::Admin {
            return Err(TaskError::PermissionDenied);
        }

        // Validate assigned user exists if provided
        if let Some(user_id) = assigned_to_id {
            if UserRepository::find_by_id(pool, user_id).await?.is_none() {
                return Err(TaskError::UserNotFound);
            }
        }

        let now = Utc::now();
        let task = Task {
            id: Uuid::new_v4(),
            title,
            description,
            status,
            priority,
            created_by_id: creator.id,
            assigned_to_id,
            created_at: now,
            updated_at: now,
        };

        TaskRepository::insert(pool, &task).await?;

        Ok(task)
    }

    pub async fn assign_tasks(
        pool: &SqlitePool,
        admin: &User,
        task_ids: Vec<Uuid>,
        user_id: Uuid,
        cache: &TaskResponseCache,
    ) -> Result<(), TaskError> {
        if admin.role != Role::Admin {
            return Err(TaskError::PermissionDenied);
        }

        // Validate user exists
        if UserRepository::find_by_id(pool, user_id).await?.is_none() {
            return Err(TaskError::UserNotFound);
        }

        // Fetch previous assignees for invalidating their cache
        let previous_assignees = TaskRepository::find_assigned_to_by_ids(pool, &task_ids).await?;

        // Bulk assign tasks
        TaskRepository::assign_tasks(pool, &task_ids, user_id).await?;

        // Invalidate cache for the new assignee
        cache.remove(&user_id);

        // Invalidate cache for all previous assignees
        for old_user_id in previous_assignees {
            cache.remove(&old_user_id);
        }

        Ok(())
    }

    pub async fn get_my_tasks(
        pool: &SqlitePool,
        user: &User,
        cache: &TaskResponseCache,
    ) -> Result<crate::models::ViewTasksResponse, TaskError> {
        if let Some(cached_val) = cache.get(&user.id) {
            if let Ok(mut response_data) = serde_json::from_value::<crate::models::ViewTasksResponse>(cached_val.clone()) {
                response_data.cache.hit = true;
                return Ok(response_data);
            }
        }

        // Fetch from DB
        let tasks = TaskRepository::find_assigned_tasks_with_emails(pool, user.id).await?;
        let user_summary = crate::models::UserSummary {
            email: user.email.clone(),
            role: user.role,
        };
        let summary = crate::models::TasksSummary {
            total_assigned_tasks: tasks.len(),
        };

        let response = crate::models::ViewTasksResponse {
            user: user_summary,
            tasks,
            summary,
            cache: crate::models::CacheSummary { hit: false },
        };

        // Cache the response
        if let Ok(val) = serde_json::to_value(&response) {
            cache.insert(user.id, val);
        }

        Ok(response)
    }
}
