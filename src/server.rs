use crate::{Result, cleanup, config::Config, router::build_router, state::AppState};

/// Binds, starts the cleanup task and serves until Ctrl-C/SIGTERM, then drains in-flight requests.
pub async fn run(config: Config) -> Result<()> {
    let listener = tokio::net::TcpListener::bind(config.bind_addr).await?;
    println!(
        "IP Tracker server v{} running on http://{}",
        env!("CARGO_PKG_VERSION"),
        listener.local_addr()?
    );

    let state = AppState::new(config);
    let cleanup = cleanup::spawn_cleanup_task(&state);
    let result = axum::serve(listener, build_router(state))
        .with_graceful_shutdown(shutdown_signal())
        .await;
    println!("Server stopped");
    println!("Stopping cleanup task...");
    cleanup.abort();
    println!("Cleanup task stopped");
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
        _ = ctrl_c => { println!("Ctrl-C received"); },
        _ = terminate => { println!("SIGTERM received"); },
    }
    println!("Shutdown signal received, draining connections...");
}
