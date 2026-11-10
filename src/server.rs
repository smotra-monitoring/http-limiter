use crate::{Result, cleanup, config::Config, router::build_router, state::AppState};

/// Binds, starts the cleanup task and serves until Ctrl-C/SIGTERM, then drains in-flight requests.
pub async fn run(config: Config) -> Result<()> {
    // Ignore the error: a global subscriber may already be installed.
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .try_init();

    let listener = tokio::net::TcpListener::bind(config.bind_addr).await?;
    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        addr = %listener.local_addr()?,
        "server listening..."
    );

    let state = AppState::new(config);
    let cleanup = cleanup::spawn_cleanup_task(&state);
    let result = axum::serve(listener, build_router(state))
        .with_graceful_shutdown(shutdown_signal())
        .await;
    tracing::info!("Server stopped");
    tracing::info!("Stopping cleanup task...");
    cleanup.abort();
    tracing::info!("Cleanup task stopped");
    Ok(result?)
}

/// Resolves on Ctrl-C or, on Unix, SIGTERM.
async fn shutdown_signal() {
    let ctrl_c = async {
        if tokio::signal::ctrl_c().await.is_err() {
            std::future::pending::<()>().await;
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut sigterm) => {
                sigterm.recv().await;
            }
            Err(_) => std::future::pending::<()>().await,
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => { tracing::info!("Ctrl-C received"); },
        _ = terminate => { tracing::info!("SIGTERM received"); },
    }
    tracing::info!("Shutdown signal received, draining connections...");
}
