# Copilot Instructions

## Project layout
- `src/bin/main.rs`: minimal entry point; only calls `http_limiter::run`. Keep it that way.
- `src/lib.rs`: public re-exports.
- `src/router.rs`: router construction (`build_router`).
- `src/server.rs`: server startup (`run`).
- `src/config.rs`: all tunables and env parsing (`HTTP_LIMITER_*`). Add new settings here, not as constants elsewhere.
- `src/state.rs`: shared `AppState`.
- `src/handlers.rs`: HTTP handlers.
- `src/cleanup.rs`: background cleanup task and its logic.
- `src/error.rs`: `Error` type.
- `docs/`: project documentation (see Documentation section).

## Conventions
- New logic goes in the library crate and must be testable without binding a socket (use `build_router` + `tower::ServiceExt::oneshot`).
- Build `AppState` through `AppState::new(Config)`; tests override fields with `Config { .., ..Config::default() }`.
- Hold the `ip_records` lock for as short a time as possible and never across an `.await`.
- Avoid `unwrap()` on I/O; propagate `std::io::Result`.
- Run `cargo fmt`, `cargo clippy --all-targets` and `cargo test` before finishing.

## Documentation (mandatory)
Always update documentation in the same change whenever a feature is implemented or a bug is fixed:
- `docs/CHANGELOG.md`: add an entry under `Unreleased` (Added / Changed / Fixed / Removed).
- `docs/features/`: update or add the page for the affected feature and keep the `docs/features/README.md` index in sync.
- `docs/README.md`: update the API table, configuration table or quick start if affected. It is the GitHub landing page; keep it concise and accurate.
- `docs/architecture.md`: update if modules, state or request flow change.
- New or changed `HTTP_LIMITER_*` settings must be reflected in `docs/README.md` and `docs/features/configuration.md`.
- Documentation must match the code's actual behavior; verify before writing.

## Improvement suggestions
- Replace `RwLock<BTreeMap>` with a sharded map (e.g. `dashmap`) to reduce write-lock contention on `/check`.
- Handle `RwLock` poisoning instead of `unwrap()`.
- Use a metrics crate (e.g. `prometheus`) instead of hand-formatted text.
