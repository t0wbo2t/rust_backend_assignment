use sqlx::SqlitePool;
use uuid::Uuid;
use crate::models::{LoginChallenge, EmailLog};

pub struct ChallengeRepository;

impl ChallengeRepository {
    pub async fn insert_challenge(pool: &SqlitePool, challenge: &LoginChallenge) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO login_challenges (id, user_id, hashed_code, expires_at, used, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)"
        )
        .bind(challenge.id)
        .bind(challenge.user_id)
        .bind(&challenge.hashed_code)
        .bind(challenge.expires_at)
        .bind(challenge.used)
        .bind(challenge.created_at)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn find_challenge_by_id(pool: &SqlitePool, id: Uuid) -> Result<Option<LoginChallenge>, sqlx::Error> {
        sqlx::query_as::<_, LoginChallenge>(
            "SELECT id, user_id, hashed_code, expires_at, used, created_at \
             FROM login_challenges WHERE id = ?1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn mark_challenge_as_used(pool: &SqlitePool, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE login_challenges SET used = 1 WHERE id = ?1"
        )
        .bind(id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn log_email(pool: &SqlitePool, log: &EmailLog) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO email_logs (id, recipient, code, created_at) \
             VALUES (?1, ?2, ?3, ?4)"
        )
        .bind(log.id)
        .bind(&log.recipient)
        .bind(&log.code)
        .bind(log.created_at)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn get_latest_email_log(pool: &SqlitePool) -> Result<Option<EmailLog>, sqlx::Error> {
        sqlx::query_as::<_, EmailLog>(
            "SELECT id, recipient, code, created_at \
             FROM email_logs ORDER BY created_at DESC LIMIT 1"
        )
        .fetch_optional(pool)
        .await
    }
}
