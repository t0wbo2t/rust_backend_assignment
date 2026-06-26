use axum::{
    extract::{Request, State},
    http::header,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use uuid::Uuid;

use crate::state::AppState;
use crate::services::auth::Claims;
use crate::repositories::user::UserRepository;

pub async fn jwt_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, Response> {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                axum::Json(serde_json::json!({ "error": "Missing authorization header" })),
            )
                .into_response()
        })?;

    if !auth_header.starts_with("Bearer ") {
        return Err((
            StatusCode::UNAUTHORIZED,
            axum::Json(serde_json::json!({ "error": "Invalid authorization header format" })),
        )
            .into_response());
    }

    let token = &auth_header[7..];

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            axum::Json(serde_json::json!({ "error": "Invalid or expired token" })),
        )
            .into_response()
    })?;

    let user_id = Uuid::parse_str(&token_data.claims.sub).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            axum::Json(serde_json::json!({ "error": "Invalid user ID in token" })),
        )
            .into_response()
    })?;

    let user = UserRepository::find_by_id(&state.database, user_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({ "error": format!("Database error: {e}") })),
            )
                .into_response()
        })?
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                axum::Json(serde_json::json!({ "error": "User not found" })),
            )
                .into_response()
        })?;

    // Insert user into request extensions
    req.extensions_mut().insert(user);

    Ok(next.run(req).await)
}
