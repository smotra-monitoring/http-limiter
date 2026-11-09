use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use http_limiter::{AppState, Config, build_router};
use std::time::Duration;
use tower::ServiceExt;

fn make_request(ip: &str) -> Request<Body> {
    Request::post("/check")
        .header("content-type", "application/json")
        .body(Body::from(format!(r#"{{"ip":"{ip}"}}"#)))
        .unwrap()
}

async fn status(app: &Router, req: Request<Body>) -> StatusCode {
    app.clone().oneshot(req).await.unwrap().status()
}

async fn metric(app: &Router, name: &str) -> String {
    let req = Request::get("/metrics").body(Body::empty()).unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    String::from_utf8(body.to_vec())
        .unwrap()
        .lines()
        .find_map(|l| l.strip_prefix(name)?.strip_prefix(' '))
        .unwrap()
        .to_string()
}

#[tokio::test]
async fn rate_limits_per_ip() {
    let state = AppState::new(Config {
        requests_per_window: 2,
        ..Config::default()
    });
    let app = build_router(state);

    assert_eq!(status(&app, make_request("1.1.1.1")).await, StatusCode::OK);
    assert_eq!(status(&app, make_request("1.1.1.1")).await, StatusCode::OK);
    assert_eq!(
        status(&app, make_request("1.1.1.1")).await,
        StatusCode::TOO_MANY_REQUESTS
    );
    assert_eq!(status(&app, make_request("2.2.2.2")).await, StatusCode::OK);
    assert_eq!(
        metric(&app, "http_limiter_ip_block_events_initiated_total").await,
        "1"
    );
    assert_eq!(
        metric(&app, "http_limiter_requests_while_blocked_total").await,
        "0"
    );
}

#[tokio::test]
async fn block_time_outlasts_rate_limit_window() {
    let state = AppState::new(Config {
        requests_per_window: 1,
        rate_limit_window: Duration::from_millis(20),
        block_time: Duration::from_millis(300),
        ..Config::default()
    });
    let app = build_router(state);

    assert_eq!(status(&app, make_request("1.1.1.1")).await, StatusCode::OK);
    assert_eq!(
        status(&app, make_request("1.1.1.1")).await,
        StatusCode::TOO_MANY_REQUESTS
    );

    assert_eq!(
        metric(&app, "http_limiter_ip_block_events_initiated_total").await,
        "1"
    );
    assert_eq!(
        metric(&app, "http_limiter_requests_while_blocked_total").await,
        "0"
    );

    // The window has elapsed but the block has not.
    tokio::time::sleep(Duration::from_millis(60)).await;
    assert_eq!(
        status(&app, make_request("1.1.1.1")).await,
        StatusCode::TOO_MANY_REQUESTS
    );

    assert_eq!(
        metric(&app, "http_limiter_ip_block_events_initiated_total").await,
        "1"
    );
    assert_eq!(
        metric(&app, "http_limiter_requests_while_blocked_total").await,
        "1"
    );

    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!(status(&app, make_request("1.1.1.1")).await, StatusCode::OK);

    assert_eq!(
        metric(&app, "http_limiter_ip_block_events_initiated_total").await,
        "1"
    );
    assert_eq!(
        metric(&app, "http_limiter_requests_while_blocked_total").await,
        "1"
    );
}

#[tokio::test]
async fn zero_block_time_recovers_after_window() {
    let state = AppState::new(Config {
        requests_per_window: 1,
        rate_limit_window: Duration::from_millis(20),
        block_time: Duration::from_millis(40),
        ..Config::default()
    });
    let app = build_router(state);

    assert_eq!(status(&app, make_request("1.1.1.1")).await, StatusCode::OK);
    assert_eq!(
        status(&app, make_request("1.1.1.1")).await,
        StatusCode::TOO_MANY_REQUESTS
    );
    tokio::time::sleep(Duration::from_millis(60)).await;
    assert_eq!(status(&app, make_request("1.1.1.1")).await, StatusCode::OK);
}

#[tokio::test]
async fn caps_unique_ips() {
    let state = AppState::new(Config {
        max_unique_ips: 1,
        ..Config::default()
    });
    let app = build_router(state);

    assert_eq!(status(&app, make_request("1.1.1.1")).await, StatusCode::OK);
    assert_eq!(status(&app, make_request("1.1.1.1")).await, StatusCode::OK);
    assert_eq!(
        status(&app, make_request("2.2.2.2")).await,
        StatusCode::TOO_MANY_REQUESTS
    );
    assert_eq!(
        metric(&app, "http_limiter_unique_ip_limit_exceeded_total").await,
        "1"
    );
}

#[tokio::test]
async fn health_endpoints_return_ok() {
    let app = build_router(AppState::new(Config::default()));
    for path in ["/healthz/live", "/healthz/ready", "/metrics"] {
        let req = Request::get(path).body(Body::empty()).unwrap();
        assert_eq!(status(&app, req).await, StatusCode::OK, "{path}");
    }
}
