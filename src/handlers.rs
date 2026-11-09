use std::{
    net::IpAddr,
    sync::atomic::Ordering,
    time::{Duration, Instant},
};

use axum::{
    Json,
    extract::State,
    http::{StatusCode, header::RETRY_AFTER},
    response::{IntoResponse, Response},
};
use serde::Deserialize;

use crate::error::Error;
use crate::state::SharedState;

#[derive(Deserialize)]
pub(crate) struct IpPayload {
    pub(crate) ip: IpAddr,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::Http429 { retry_after } => http_429(retry_after),
            Error::HttpStatus { status } => status.into_response(),
            _ => StatusCode::SERVICE_UNAVAILABLE.into_response(),
        }
    }
}

fn http_429(retry_after: Duration) -> Response {
    let secs = retry_after.as_secs() + u64::from(retry_after.subsec_nanos() > 0);
    (
        StatusCode::TOO_MANY_REQUESTS,
        [(RETRY_AFTER, secs.max(1).to_string())],
    )
        .into_response()
}

pub(crate) async fn check_ip_handler(
    State(state): State<SharedState>,
    Json(payload): Json<IpPayload>,
) -> Result<Response, Error> {
    state.total_requests.fetch_add(1, Ordering::Relaxed);

    let cfg = &state.config;
    let now = Instant::now();
    let window_start = now.checked_sub(cfg.rate_limit_window);

    let mut records = state.records_write()?;

    // New IPs are rejected once the unique IP cap is reached
    if !records.contains_key(&payload.ip) && records.len() >= cfg.max_unique_ips {
        println!("Unique IP limit exceeded for IP: {}", payload.ip);
        state
            .unique_ip_limit_exceeded_total
            .fetch_add(1, Ordering::Relaxed);
        // Slots free up via the idle cleanup, so retry after one cleanup interval
        return Err(Error::Http429 {
            retry_after: cfg.cleanup_interval,
        }); // this is application issue due to unique IP cap, return Err
    }

    let record = records.entry(payload.ip).or_default();

    match record.blocked_until {
        Some(until) if until > now => {
            state.requests_while_blocked.fetch_add(1, Ordering::Relaxed);
            return Ok(http_429(until - now));
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
        record.blocked_until = Some(now + cfg.block_time);

        return Ok(http_429(cfg.block_time)); // this is expected behavior for rate-limited IPs, return Ok
    }

    // IMO this will never happen under normal circumstances
    if timestamps.len() >= cfg.timestamp_bucket_size {
        println!(
            "(Require investigation) Timestamps exceeded max for IP: {}",
            payload.ip
        );
        state
            .ip_bucket_utiilization_spill_over
            .fetch_add(1, Ordering::Relaxed);
        timestamps.pop_front();
    }
    timestamps.push_back(now);

    Ok(StatusCode::OK.into_response())
}

pub(crate) async fn metrics_handler(State(state): State<SharedState>) -> Result<String, Error> {
    let unique_ips_tracked = state.records_read()?.len();
    let total_reqs = state.total_requests.load(Ordering::Relaxed);
    let ip_block_initiated = state.ip_block_initiated.load(Ordering::Relaxed);
    let unique_cap_exceeded = state.unique_ip_limit_exceeded_total.load(Ordering::Relaxed);
    let ip_bucket_utiilization_spill_over = state
        .ip_bucket_utiilization_spill_over
        .load(Ordering::Relaxed);
    let blocked = state.requests_while_blocked.load(Ordering::Relaxed);

    Ok(format!(
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
         # HELP ip_bucket_utiilization_spill_over Number of times the IP bucket utilization spilled over (if above 0, needed to be increased).\n\
         # TYPE ip_bucket_utiilization_spill_over counter\n\
         ip_bucket_utiilization_spill_over {}\n\
         \n\
         # HELP ip_tracker_requests_while_blocked_total Total requests rejected because the IP is in its block period.\n\
         # TYPE ip_tracker_requests_while_blocked_total counter\n\
         ip_tracker_requests_while_blocked_total {}\n",
        unique_ips_tracked,
        total_reqs,
        ip_block_initiated,
        unique_cap_exceeded,
        ip_bucket_utiilization_spill_over,
        blocked
    ))
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
