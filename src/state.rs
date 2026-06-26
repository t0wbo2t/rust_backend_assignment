use crate::config::AppConfig;
use dashmap::DashMap;
use serde_json::Value;
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use std::sync::Arc;
use uuid::Uuid;

pub type TaskResponseCache = Arc<DashMap<Uuid, Value>>;

#[derive(Clone, Debug)]
pub struct AppState {
    pub config: AppConfig,
    pub database: SqlitePool,
    pub task_response_cache: TaskResponseCache,
    pub jwt_secret: Arc<str>,
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite://tasks.db?mode=rwc".to_string());
        Self {
            config,
            database: SqlitePoolOptions::new()
                .connect_lazy(&database_url)
                .expect("database connection URL must be valid"),
            task_response_cache: Arc::new(DashMap::new()),
            jwt_secret: Arc::from("change-me"),
        }
    }
}
