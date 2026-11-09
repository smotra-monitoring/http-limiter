use std::{net::IpAddr, sync::atomic::Ordering, time::Instant};

use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;

use crate::state::SharedState;

#[derive(Deserialize)]
pub(crate) struct IpPayload {
    pub(crate) ip: IpAddr,
}

pub(crate) async fn check_ip_handler(
    State(state): State<SharedState>,
    Json(payload): Json<IpPayload>,
) -> Result<StatusCode, StatusCode> {
    state.total_requests.fetch_add(1, Ordering::Relaxed);

    let cfg = &state.config;
    let now = Instant::now();
    let window_start = now.checked_sub(cfg.rate_limit_window);

    let mut records = state.ip_records.write().unwrap();

    // New IPs are rejected once the unique IP cap is reached
    if !records.contains_key(&payload.ip) && records.len() >= cfg.max_unique_ips {
        println!("Unique IP limit exceeded for IP: {}", payload.ip);
        state
            .unique_ip_limit_exceeded_total
            .fetch_add(1, Ordering::Relaxed);
        return Err(StatusCode::TOO_MANY_REQUESTS); // this is application issue due to unique IP cap, return Err
    }

    let record = records.entry(payload.ip).or_default();

    match record.blocked_until {
        Some(until) if until > now => {
            state.requests_while_blocked.fetch_add(1, Ordering::Relaxed);
            return Ok(StatusCode::TOO_MANY_REQUESTS);
        }
        Some(_) => record.blocked_until = None,
        None => {}
    }

    let timestamps = &mut record.timestamps;

    if let Some(window_start) = window_start {
        while timestamps.front().is_some_and(|&t| t < window_start) {
            timestamps.pop_front();
        }
    }

    if timestamps.len() >= cfg.requests_per_window {
        println!("Blocking IP: {}", payload.ip);
        state.ip_block_initiated.fetch_add(1, Ordering::Relaxed);
        if !cfg.block_time.is_zero() {
            record.blocked_until = Some(now + cfg.block_time);
        }
        return Ok(StatusCode::TOO_MANY_REQUESTS); // this is expected behavior for rate-limited IPs, return Ok
    }

    if timestamps.len() >= cfg.max_timestamps {
        timestamps.pop_front();
    }
    timestamps.push_back(now);

    Ok(StatusCode::OK)
}

pub(crate) async fn metrics_handler(State(state): State<SharedState>) -> String {
    let unique_ips_tracked = state.ip_records.read().unwrap().len();
    let total_reqs = state.total_requests.load(Ordering::Relaxed);
    let ip_block_initiated = state.ip_block_initiated.load(Ordering::Relaxed);
    let unique_cap_exceeded = state.unique_ip_limit_exceeded_total.load(Ordering::Relaxed);
    let blocked = state.requests_while_blocked.load(Ordering::Relaxed);

    format!(
        "# HELP ip_tracker_unique_ips_total Total active unique IP addresses tracked in memory.\n\
         # TYPE ip_tracker_unique_ips_total gauge\n\
         ip_tracker_unique_ips_total {}\n\
         \n\
         # HELP ip_tracker_http_requests_total Total number of HTTP requests processed.\n\
         # TYPE ip_tracker_http_requests_total counter\n\
         ip_tracker_http_requests_total {}\n\
         \n\
         # HELP ip_tracker_ip_block_initiated_total Total IP block-events initiated due to rate limiting.\n\
         # TYPE ip_tracker_ip_block_initiated_total counter\n\
         ip_tracker_ip_block_initiated_total {}\n\
         \n\
         # HELP ip_tracker_unique_ip_limit_exceeded_total Total requests rejected due to max unique IP capacity cap.\n\
         # TYPE ip_tracker_unique_ip_limit_exceeded_total counter\n\
         ip_tracker_unique_ip_limit_exceeded_total {}\n\
         \n\
         # HELP ip_tracker_requests_while_blocked_total Total requests rejected because the IP is in its block period.\n\
         # TYPE ip_tracker_requests_while_blocked_total counter\n\
         ip_tracker_requests_while_blocked_total {}\n",
        unique_ips_tracked, total_reqs, ip_block_initiated, unique_cap_exceeded, blocked
    )
}

pub(crate) async fn liveness_handler(State(state): State<SharedState>) -> StatusCode {
    match state.ip_records.try_read() {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::SERVICE_UNAVAILABLE,
    }
}

pub(crate) async fn readiness_handler() -> StatusCode {
    StatusCode::OK
}
