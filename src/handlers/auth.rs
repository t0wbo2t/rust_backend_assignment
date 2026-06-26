use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::state::AppState;
use crate::services::auth::{AuthService, AuthError};
use crate::repositories::challenge::ChallengeRepository;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct Verify2FaRequest {
    pub login_challenge_id: Uuid,
    pub code: String,
}

fn map_auth_error(err: AuthError) -> (StatusCode, String) {
    match err {
        AuthError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()),
        AuthError::ChallengeNotFound => (StatusCode::NOT_FOUND, "Challenge not found".to_string()),
        AuthError::ChallengeExpired => (StatusCode::GONE, "Challenge expired".to_string()),
        AuthError::ChallengeUsed => (StatusCode::GONE, "Challenge already used".to_string()),
        AuthError::InvalidCode => (StatusCode::UNAUTHORIZED, "Invalid code".to_string()),
        AuthError::Database(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {e}")),
        AuthError::Jwt(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("JWT error: {e}")),
        AuthError::Argon2(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Argon2 error: {e}")),
    }
}

pub async fn seed_users(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, String)> {
    AuthService::seed_users(&state.database)
        .await
        .map_err(map_auth_error)?;

    Ok((
        StatusCode::OK,
        Json(json!({ "message": "Users seeded successfully" })),
    ))
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, String)> {
    let challenge_id = AuthService::login(&state.database, &payload.email, &payload.password)
        .await
        .map_err(map_auth_error)?;

    Ok((
        StatusCode::OK,
        Json(json!({ "login_challenge_id": challenge_id })),
    ))
}

pub async fn verify_2fa(
    State(state): State<AppState>,
    Json(payload): Json<Verify2FaRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, String)> {
    let token = AuthService::verify_2fa(
        &state.database,
        payload.login_challenge_id,
        &payload.code,
        &state.jwt_secret,
    )
    .await
    .map_err(map_auth_error)?;

    Ok((
        StatusCode::OK,
        Json(json!({ "token": token })),
    ))
}

pub async fn get_latest_email(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, String)> {
    let log = ChallengeRepository::get_latest_email_log(&state.database)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {e}")))?;

    match log {
        Some(log) => Ok((
            StatusCode::OK,
            Json(json!({
                "recipient": log.recipient,
                "code": log.code,
            })),
        )),
        None => Err((StatusCode::NOT_FOUND, "No email logs found".to_string())),
    }
}

pub async fn seed_full(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, String)> {
    let (admin_token, jb_token) = AuthService::seed_full(
        &state.database,
        &state.jwt_secret,
        &state.task_response_cache,
    )
    .await
    .map_err(map_auth_error)?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "admin_token": admin_token,
            "james_bond_token": jb_token,
        })),
    ))
}
