use crate::{cleanup, config::Config, router::build_router, state::AppState};

/// Binds, starts the cleanup task and serves until the server stops.
pub async fn run(config: Config) -> std::io::Result<()> {
    let listener = tokio::net::TcpListener::bind(config.bind_addr).await?;
    println!(
        "IP Tracker server running on http://{}",
        listener.local_addr()?
    );

    let state = AppState::new(config);
    let cleanup = cleanup::spawn_cleanup_task(&state);
    let result = axum::serve(listener, build_router(state)).await;
    cleanup.abort();
    result
}
