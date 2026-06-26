use sqlx::SqlitePool;
use uuid::Uuid;
use crate::models::User;

pub struct UserRepository;

impl UserRepository {
    pub async fn insert(pool: &SqlitePool, user: &User) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO users (id, full_name, email, hashed_password, role, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"
        )
        .bind(user.id)
        .bind(&user.full_name)
        .bind(&user.email)
        .bind(&user.hashed_password)
        .bind(user.role)
        .bind(user.created_at)
        .bind(user.updated_at)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn find_by_email(pool: &SqlitePool, email: &str) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "SELECT id, full_name, email, hashed_password, role, created_at, updated_at \
             FROM users WHERE email = ?1"
        )
        .bind(email)
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_id(pool: &SqlitePool, id: Uuid) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "SELECT id, full_name, email, hashed_password, role, created_at, updated_at \
             FROM users WHERE id = ?1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }
}
