# Changelog

All notable changes are documented here. Format based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Changed

- Renamed the misnamed `http_limiter_blocked_ip_current` gauge to `http_limiter_tracked_ip_current`; it counts all tracked IPs, not only blocked ones. Grafana dashboard updated.
- Logging uses `tracing` instead of `println!`/`eprintln!`; verbosity is controlled by `RUST_LOG` (default `info`). Log line format changed to structured fields.

### Fixed

- Corrected the memory estimates for the unique IP cap (previously overstated about 2x); see [Unique IP cap](features/unique-ip-cap.md).

### Added

- `GET /blocked` returns the currently blocked IPs with their remaining block time.
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
