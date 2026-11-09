# Copilot Instructions

## Project layout
- `src/bin/main.rs`: minimal entry point; only calls `http_limiter::run`. Keep it that way.
- `src/lib.rs`: router construction (`build_router`) and server startup (`run`).
- `src/config.rs`: all tunables and env parsing (`HTTP_LIMITER_*`). Add new settings here, not as constants elsewhere.
- `src/state.rs`: shared `AppState`.
- `src/handlers.rs`: HTTP handlers.
- `src/cleanup.rs`: background cleanup task and its logic.

## Conventions
- New logic goes in the library crate and must be testable without binding a socket (use `build_router` + `tower::ServiceExt::oneshot`).
- Build `AppState` through `AppState::new(Config)`; tests override fields with `Config { .., ..Config::default() }`.
- Hold the `ip_records` lock for as short a time as possible and never across an `.await`.
- Avoid `unwrap()` on I/O; propagate `std::io::Result`.
- Run `cargo fmt`, `cargo clippy --all-targets` and `cargo test` before finishing.

## Improvement suggestions
- Replace `RwLock<BTreeMap>` with a sharded map (e.g. `dashmap`) to reduce write-lock contention on `/check`.
- Add graceful shutdown (`with_graceful_shutdown` on ctrl-c/SIGTERM) and stop the cleanup task there.
- Replace `println!` with `tracing`.
- Handle `RwLock` poisoning instead of `unwrap()`.
- Use a metrics crate (e.g. `prometheus`) instead of hand-formatted text.
