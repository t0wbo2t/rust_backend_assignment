use axum::{
    routing::{get, post},
    Router,
    middleware,
};
use tower_http::trace::TraceLayer;

use crate::state::AppState;
use crate::handlers::{auth, task};
use crate::middleware::jwt_auth;

pub fn build_router(state: AppState) -> Router {
    let task_routes = Router::new()
        .route("/tasks", post(task::create_task))
        .route("/tasks/assign", post(task::assign_tasks))
        .route("/tasks/view-my-tasks", get(task::view_my_tasks))
        .route_layer(middleware::from_fn_with_state(state.clone(), jwt_auth));

    Router::new()
        .route("/seed/users", post(auth::seed_users))
        .route("/auth/login", post(auth::login))
        .route("/auth/verify-2fa", post(auth::verify_2fa))
        .route("/dev/email-logs/latest", get(auth::get_latest_email))
        .route("/dev/seed-full", post(auth::seed_full))
        .merge(task_routes)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
