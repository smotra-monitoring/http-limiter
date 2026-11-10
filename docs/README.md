# http-limiter

A small, fast, in-memory per-IP rate limiter exposed as an HTTP service. Written in Rust with [axum](https://github.com/tokio-rs/axum) and [tokio](https://tokio.rs).

Your gateway, proxy, or application asks `http-limiter` whether a client IP may proceed. The service answers with `200 OK` (allow) or `429 Too Many Requests` (reject) and a `Retry-After` header.

## Features

- Sliding-window rate limit per IP address (IPv4 and IPv6)
- Temporary block of IPs that exceed the limit, with `Retry-After`
- Hard cap on tracked unique IPs to bound memory usage
- Background cleanup of idle IPs
- Prometheus-compatible `/metrics`
- Liveness and readiness probes
- Configuration through environment variables only

Details for each feature: [docs/features](features/README.md).

## Quick start

Requires a recent stable Rust toolchain (edition 2024).

```bash
cargo run --release
```

The server listens on `0.0.0.0:8082` by default.

Ask whether an IP is allowed:

```bash
curl -i -X POST http://localhost:8082/check \
  -H 'Content-Type: application/json' \
  -d '{"ip": "203.0.113.7"}'
```

| Response | Meaning |
|---|---|
| `200 OK` | Request allowed |
| `429 Too Many Requests` + `Retry-After: <seconds>` | IP is rate limited or blocked, or the unique IP cap is reached |
| `503 Service Unavailable` | Internal error (poisoned lock) |

## API

| Method | Path | Description |
|---|---|---|
| `POST` | `/check` | Body `{"ip": "<address>"}`; registers a request and returns the verdict |
| `GET` | `/metrics` | Prometheus text metrics |
| `GET` | `/healthz/live` | Liveness probe |
| `GET` | `/healthz/ready` | Readiness probe |

## Configuration

All settings are optional environment variables. Unset or unparsable values fall back to the default.

| Variable | Default | Description |
|---|---|---|
| `HTTP_LIMITER_BIND_ADDR` | `0.0.0.0:8082` | Address and port to listen on |
| `HTTP_LIMITER_REQUESTS_PER_WINDOW` | `20` | Allowed requests per IP within the window |
| `HTTP_LIMITER_RATE_LIMIT_WINDOW_SECS` | `60` | Sliding window length |
| `HTTP_LIMITER_BLOCK_TIME_SECS` | `120` | How long an IP stays blocked after exceeding the limit |
| `HTTP_LIMITER_MAX_UNIQUE_IPS` | `200000` | Maximum number of IPs tracked at once |
| `HTTP_LIMITER_IDLE_TIME_SECS` | `300` | Inactivity after which an IP is forgotten |
| `HTTP_LIMITER_CLEANUP_INTERVAL_SECS` | `60` | How often the cleanup task runs |
| `RUST_LOG` | `info` | Log filter (standard `tracing` env filter syntax) |

Example:

```bash
HTTP_LIMITER_BIND_ADDR=127.0.0.1:9000 \
HTTP_LIMITER_REQUESTS_PER_WINDOW=100 \
cargo run --release
```

The process exits with an error at startup if the configuration is invalid. See [Configuration](features/configuration.md) for the rules.

## Development

```bash
cargo fmt
cargo clippy --all-targets
cargo test
```

## Documentation

- [Features](features/README.md)
- [Architecture](architecture.md)
- [Development guide](development.md)
- [Changelog](CHANGELOG.md)

## Limitations

- State is kept in process memory: it is lost on restart and not shared between instances.
- The service trusts the IP supplied in the request body; callers are responsible for extracting the real client IP.
