use backend_project::config::AppConfig;
use backend_project::state::AppState;
use backend_project::router;
use backend_project::error;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> error::AppResult<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let state = AppState::new(AppConfig::default());

    // Run database migrations on startup
    sqlx::migrate!()
        .run(&state.database)
        .await?;

    let address = state.config.address;
    let app = router::build_router(state.clone());

    let listener = tokio::net::TcpListener::bind(address).await?;
    tracing::info!("listening on {address}");

    axum::serve(listener, app).await?;

    Ok(())
}
