# Development guide

## Build and run

```bash
cargo build
cargo run
```

## Checks (run before every commit)

```bash
cargo fmt
cargo clippy --all-targets
cargo test
```

## Testing

- Unit tests live next to the code (`config.rs`, `cleanup.rs`).
- Integration tests in `tests/api.rs` build the router with `build_router` and drive it with `tower::ServiceExt::oneshot`; no socket is bound.
- Create state with `AppState::new(Config { field: value, ..Config::default() })`.

## Conventions

- Keep `src/bin/main.rs` minimal.
- Add new settings to `src/config.rs`, parsed from `HTTP_LIMITER_*` variables.
- Hold the `ip_records` lock as briefly as possible and never across an `.await`.
- Propagate `std::io::Result`; avoid `unwrap()` on I/O.

## Documentation

Every new feature or bug fix must update the docs in the same change:

1. Add an entry under `Unreleased` in [CHANGELOG.md](CHANGELOG.md).
2. Update or add the matching page in [features](features/README.md).
3. Update the configuration table in [README.md](README.md) and [features/configuration.md](features/configuration.md) if settings change.
4. Update [architecture.md](architecture.md) if modules or flow change.
