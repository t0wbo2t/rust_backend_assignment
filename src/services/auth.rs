use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};
use chrono::Utc;
use jsonwebtoken::{encode, Header, EncodingKey};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::models::{User, Role, LoginChallenge, EmailLog, Task};
use crate::repositories::user::UserRepository;
use crate::repositories::challenge::ChallengeRepository;
use crate::repositories::task::TaskRepository;

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Challenge not found")]
    ChallengeNotFound,
    #[error("Challenge expired")]
    ChallengeExpired,
    #[error("Challenge already used")]
    ChallengeUsed,
    #[error("Invalid verification code")]
    InvalidCode,
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),
    #[error("Argon2 hashing error: {0}")]
    Argon2(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub role: String,
    pub exp: usize,
}

pub struct AuthService;

impl AuthService {
    pub fn hash_password(password: &str) -> Result<String, AuthError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|h| h.to_string())
            .map_err(|e| AuthError::Argon2(e.to_string()))
    }

    pub fn verify_password(password: &str, hashed_password: &str) -> bool {
        if let Ok(parsed_hash) = PasswordHash::new(hashed_password) {
            Argon2::default()
                .verify_password(password.as_bytes(), &parsed_hash)
                .is_ok()
        } else {
            false
        }
    }

    pub async fn seed_users(pool: &SqlitePool) -> Result<(), AuthError> {
        // Seed Admin user if not exists
        if UserRepository::find_by_email(pool, "admin@example.com").await?.is_none() {
            let hashed_password = Self::hash_password("admin123")?;
            let now = Utc::now();
            let admin = User {
                id: Uuid::new_v4(),
                full_name: "Admin User".to_string(),
                email: "admin@example.com".to_string(),
                hashed_password,
                role: Role::Admin,
                created_at: now,
                updated_at: now,
            };
            UserRepository::insert(pool, &admin).await?;
        }

        // Seed James Bond user if not exists
        if UserRepository::find_by_email(pool, "jamesbond@example.com").await?.is_none() {
            let hashed_password = Self::hash_password("james123")?;
            let now = Utc::now();
            let staff = User {
                id: Uuid::new_v4(),
                full_name: "James Bond".to_string(),
                email: "jamesbond@example.com".to_string(),
                hashed_password,
                role: Role::Staff,
                created_at: now,
                updated_at: now,
            };
            UserRepository::insert(pool, &staff).await?;
        }

        Ok(())
    }

    pub async fn login(pool: &SqlitePool, email: &str, password: &str) -> Result<Uuid, AuthError> {
        let user = UserRepository::find_by_email(pool, email)
            .await?
            .ok_or(AuthError::InvalidCredentials)?;

        if !Self::verify_password(password, &user.hashed_password) {
            return Err(AuthError::InvalidCredentials);
        }

        // Generate 6-digit code inside a local block to avoid holding RNG across .await
        let raw_code = {
            use rand::Rng;
            let mut rng = rand::rng();
            let num: u32 = rng.random_range(100000..1000000);
            format!("{:06}", num)
        };

        let hashed_code = Self::hash_password(&raw_code)?;
        let challenge_id = Uuid::new_v4();
        let now = Utc::now();
        let expires_at = now + chrono::Duration::minutes(5);

        let challenge = LoginChallenge {
            id: challenge_id,
            user_id: user.id,
            hashed_code,
            expires_at,
            used: false,
            created_at: now,
        };

        ChallengeRepository::insert_challenge(pool, &challenge).await?;

        // Log the sent email with code
        let email_log = EmailLog {
            id: Uuid::new_v4(),
            recipient: user.email.clone(),
            code: raw_code.clone(),
            created_at: now,
        };
        ChallengeRepository::log_email(pool, &email_log).await?;

        Ok(challenge_id)
    }

    pub async fn verify_2fa(
        pool: &SqlitePool,
        challenge_id: Uuid,
        code: &str,
        jwt_secret: &str,
    ) -> Result<String, AuthError> {
        let challenge = ChallengeRepository::find_challenge_by_id(pool, challenge_id)
            .await?
            .ok_or(AuthError::ChallengeNotFound)?;

        if challenge.used {
            return Err(AuthError::ChallengeUsed);
        }

        if challenge.expires_at < Utc::now() {
            return Err(AuthError::ChallengeExpired);
        }

        if !Self::verify_password(code, &challenge.hashed_code) {
            return Err(AuthError::InvalidCode);
        }

        // Mark challenge as used
        ChallengeRepository::mark_challenge_as_used(pool, challenge_id).await?;

        // Fetch user info
        let user = UserRepository::find_by_id(pool, challenge.user_id)
            .await?
            .ok_or(AuthError::InvalidCredentials)?;

        // Generate JWT token
        let expiration = Utc::now()
            .checked_add_signed(chrono::Duration::hours(24))
            .expect("valid timestamp")
            .timestamp() as usize;

        let claims = Claims {
            sub: user.id.to_string(),
            email: user.email.clone(),
            role: match user.role {
                Role::Admin => "admin".to_string(),
                Role::Staff => "staff".to_string(),
            },
            exp: expiration,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(jwt_secret.as_bytes()),
        )?;

        Ok(token)
    }

    pub async fn seed_full(
        pool: &SqlitePool,
        jwt_secret: &str,
        cache: &crate::state::TaskResponseCache,
    ) -> Result<(String, String), AuthError> {
        // Seed users
        Self::seed_users(pool).await?;

        // Clear existing tasks/challenges/logs
        sqlx::query("DELETE FROM tasks").execute(pool).await?;
        sqlx::query("DELETE FROM login_challenges").execute(pool).await?;
        sqlx::query("DELETE FROM email_logs").execute(pool).await?;

        // Get users
        let admin = UserRepository::find_by_email(pool, "admin@example.com")
            .await?
            .ok_or(AuthError::InvalidCredentials)?;

        let jb = UserRepository::find_by_email(pool, "jamesbond@example.com")
            .await?
            .ok_or(AuthError::InvalidCredentials)?;

        // Create exactly 5 tasks
        let mut task_ids = Vec::new();
        let now = Utc::now();
        for i in 1..=5 {
            let task = Task {
                id: Uuid::new_v4(),
                title: format!("Task {i}"),
                description: format!("Description for task {i}"),
                status: crate::models::Status::Todo,
                priority: if i % 2 == 0 { crate::models::Priority::High } else { crate::models::Priority::Low },
                created_by_id: admin.id,
                assigned_to_id: None,
                created_at: now,
                updated_at: now,
            };
            TaskRepository::insert(pool, &task).await?;
            task_ids.push(task.id);
        }

        // Assign exactly 3 tasks to James Bond
        TaskRepository::assign_tasks(pool, &task_ids[0..3], jb.id).await?;

        // Invalidate caches for both users
        cache.remove(&admin.id);
        cache.remove(&jb.id);

        // Generate JWT tokens for both users
        let expiration = Utc::now()
            .checked_add_signed(chrono::Duration::hours(24))
            .expect("valid timestamp")
            .timestamp() as usize;

        let admin_claims = Claims {
            sub: admin.id.to_string(),
            email: admin.email.clone(),
            role: "admin".to_string(),
            exp: expiration,
        };
        let admin_token = encode(
            &Header::default(),
            &admin_claims,
            &EncodingKey::from_secret(jwt_secret.as_bytes()),
        )?;

        let jb_claims = Claims {
            sub: jb.id.to_string(),
            email: jb.email.clone(),
            role: "staff".to_string(),
            exp: expiration,
        };
        let jb_token = encode(
            &Header::default(),
            &jb_claims,
            &EncodingKey::from_secret(jwt_secret.as_bytes()),
        )?;

        Ok((admin_token, jb_token))
    }
}
