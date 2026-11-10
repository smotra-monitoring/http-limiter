# Changelog

All notable changes are documented here. Format based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Changed

- Logging uses `tracing` instead of `println!`/`eprintln!`; verbosity is controlled by `RUST_LOG` (default `info`). Log line format changed to structured fields.

### Added

- Graceful shutdown on `SIGINT` / `SIGTERM`: in-flight requests are drained before exit.
- Documentation: README, feature pages, architecture and development guides.

## [0.0.0]

### Added

- `POST /check` per-IP sliding-window rate limiting.
- Temporary IP blocking with `Retry-After`; blocked IPs are protected from idle cleanup.
- Cap on tracked unique IPs.
- Background cleanup of idle IPs.
- `GET /metrics` Prometheus metrics.
- `GET /healthz/live` and `GET /healthz/ready` probes.
- Configuration via `HTTP_LIMITER_*` environment variables with startup validation.
