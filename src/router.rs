use axum::{
    Router,
    routing::{get, post},
};

use crate::{handlers, state::SharedState};

pub fn build_router(state: SharedState) -> Router {
    Router::new()
        .route("/track", post(handlers::track_ip_handler))
        .route("/metrics", get(handlers::metrics_handler))
        .route("/healthz/live", get(handlers::liveness_handler))
        .route("/healthz/ready", get(handlers::readiness_handler))
        .with_state(state)
}
