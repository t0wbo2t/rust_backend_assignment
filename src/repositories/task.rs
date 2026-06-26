use sqlx::SqlitePool;
use uuid::Uuid;
use crate::models::Task;

pub struct TaskRepository;

impl TaskRepository {
    pub async fn insert(pool: &SqlitePool, task: &Task) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO tasks (id, title, description, status, priority, created_by_id, assigned_to_id, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"
        )
        .bind(task.id)
        .bind(&task.title)
        .bind(&task.description)
        .bind(task.status)
        .bind(task.priority)
        .bind(task.created_by_id)
        .bind(task.assigned_to_id)
        .bind(task.created_at)
        .bind(task.updated_at)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn assign_tasks(pool: &SqlitePool, task_ids: &[Uuid], user_id: Uuid) -> Result<(), sqlx::Error> {
        let mut tx = pool.begin().await?;
        let now = chrono::Utc::now();
        for &task_id in task_ids {
            sqlx::query(
                "UPDATE tasks SET assigned_to_id = ?1, updated_at = ?2 WHERE id = ?3"
            )
            .bind(user_id)
            .bind(now)
            .bind(task_id)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn find_by_id(pool: &SqlitePool, id: Uuid) -> Result<Option<Task>, sqlx::Error> {
        sqlx::query_as::<_, Task>(
            "SELECT id, title, description, status, priority, created_by_id, assigned_to_id, created_at, updated_at \
             FROM tasks WHERE id = ?1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn find_assigned_to_by_ids(pool: &SqlitePool, task_ids: &[Uuid]) -> Result<Vec<Uuid>, sqlx::Error> {
        let mut assignees = Vec::new();
        for &task_id in task_ids {
            let row: Option<(Option<Uuid>,)> = sqlx::query_as(
                "SELECT assigned_to_id FROM tasks WHERE id = ?1"
            )
            .bind(task_id)
            .fetch_optional(pool)
            .await?;
            if let Some((Some(user_id),)) = row {
                assignees.push(user_id);
            }
        }
        assignees.sort();
        assignees.dedup();
        Ok(assignees)
    }

    pub async fn find_assigned_tasks_with_emails(pool: &SqlitePool, user_id: Uuid) -> Result<Vec<crate::models::ResponseTask>, sqlx::Error> {
        sqlx::query_as::<_, crate::models::ResponseTask>(
            "SELECT t.id, t.title, t.status, t.priority, u.email as assigned_to \
             FROM tasks t \
             LEFT JOIN users u ON t.assigned_to_id = u.id \
             WHERE t.assigned_to_id = ?1"
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
    }
}
