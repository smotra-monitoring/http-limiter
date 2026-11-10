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
use crate::state::{AppState, IpRecord, RecordsMap, SharedState};

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

    let ip = payload.ip;
    let cfg = &state.config;
    let now = Instant::now();

    let mut records = state.records_write()?;

    if is_unique_ip_cap_reached(&state, &records, ip) {
        return Err(unique_ip_cap_exceeded(&state, ip));
    }

    let record = records.entry(ip).or_default();

    if let Some(remaining) = still_blocked(record, now) {
        state.requests_while_blocked.fetch_add(1, Ordering::Relaxed);
        return Ok(http_429(remaining));
    }

    evict_expired_timestamps(record, now, cfg.rate_limit_window);

    if record.timestamps.len() >= cfg.requests_per_window {
        block_ip(&state, record, ip, now);
        // Expected behavior for rate-limited IPs, so Ok rather than Err
        return Ok(http_429(cfg.block_time));
    }

    record_request(&state, record, ip, now);

    Ok(StatusCode::OK.into_response())
}

/// New IPs are rejected once the unique IP cap is reached.
fn is_unique_ip_cap_reached(state: &AppState, records: &RecordsMap, ip: IpAddr) -> bool {
    !records.contains_key(&ip) && records.len() >= state.config.max_unique_ips
}

fn unique_ip_cap_exceeded(state: &AppState, ip: IpAddr) -> Error {
    tracing::warn!(%ip, "Unique IP limit exceeded");
    state
        .unique_ip_limit_exceeded_total
        .fetch_add(1, Ordering::Relaxed);
    // Slots free up via the idle cleanup, so retry after one cleanup interval
    Error::Http429 {
        retry_after: state.config.cleanup_interval,
    }
}

/// Returns the remaining block time, clearing an expired block.
fn still_blocked(record: &mut IpRecord, now: Instant) -> Option<Duration> {
    match record.blocked_until {
        Some(until) if until > now => Some(until - now),
        Some(_) => {
            record.blocked_until = None;
            None
        }
        None => None,
    }
}

fn evict_expired_timestamps(record: &mut IpRecord, now: Instant, window: Duration) {
    let Some(window_start) = now.checked_sub(window) else {
        return;
    };
    while record.timestamps.front().is_some_and(|&t| t < window_start) {
        record.timestamps.pop_front();
    }
}

fn block_ip(state: &AppState, record: &mut IpRecord, ip: IpAddr, now: Instant) {
    tracing::info!(%ip, "blocking ");
    state.ip_block_events.fetch_add(1, Ordering::Relaxed);
    record.blocked_until = Some(now + state.config.block_time);
}

fn record_request(state: &AppState, record: &mut IpRecord, ip: IpAddr, now: Instant) {
    if record.timestamps.len() >= state.config.timestamp_bucket_size {
        // IMO this will never happen under normal circumstances
        tracing::warn!(%ip, "(Require investigation) Timestamps exceeded max");
        state
            .ip_bucket_utiilization_spill_over
            .fetch_add(1, Ordering::Relaxed);
        record.timestamps.pop_front();
    }
    record.timestamps.push_back(now);
}

pub(crate) async fn metrics_handler(State(state): State<SharedState>) -> Result<String, Error> {
    let unique_ips_tracked = state.records_read()?.len();
    let total_reqs = state.total_requests.load(Ordering::Relaxed);
    let ip_block_events = state.ip_block_events.load(Ordering::Relaxed);
    let unique_cap_exceeded = state.unique_ip_limit_exceeded_total.load(Ordering::Relaxed);
    let ip_bucket_utiilization_spill_over = state
        .ip_bucket_utiilization_spill_over
        .load(Ordering::Relaxed);
    let requests_while_blocked = state.requests_while_blocked.load(Ordering::Relaxed);

    Ok(format!(
        "# HELP http_limiter_blocked_ip_current Total active unique IP addresses tracked in memory.\n\
         # TYPE http_limiter_blocked_ip_current gauge\n\
         http_limiter_blocked_ip_current {}\n\
         \n\
         # HELP http_limiter_http_requests_total Total number of HTTP requests processed.\n\
         # TYPE http_limiter_http_requests_total counter\n\
         http_limiter_http_requests_total {}\n\
         \n\
         # HELP http_limiter_ip_block_events_total Total IP block-events initiated due to rate limiting.\n\
         # TYPE http_limiter_ip_block_events_total counter\n\
         http_limiter_ip_block_events_total {}\n\
         \n\
         # HELP http_limiter_requests_while_blocked_total Total requests rejected because the IP is in its block period.\n\
         # TYPE http_limiter_requests_while_blocked_total counter\n\
         http_limiter_requests_while_blocked_total {}\n\
         \n\
         # HELP http_limiter_unique_ip_limit_exceeded_total Total requests rejected due to max unique IP capacity cap.\n\
         # TYPE http_limiter_unique_ip_limit_exceeded_total counter\n\
         http_limiter_unique_ip_limit_exceeded_total {}\n\
         \n\
         # HELP http_limiter_ip_bucket_utiilization_spill_over Number of times the IP bucket utilization spilled over (if above 0, needed to be investigated).\n\
         # TYPE http_limiter_ip_bucket_utiilization_spill_over counter\n\
         http_limiter_ip_bucket_utiilization_spill_over {}\n\
         ",
        unique_ips_tracked,
        total_reqs,
        ip_block_events,
        requests_while_blocked,
        unique_cap_exceeded,
        ip_bucket_utiilization_spill_over,
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
