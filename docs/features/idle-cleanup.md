# Idle cleanup

A background task runs every `HTTP_LIMITER_CLEANUP_INTERVAL_SECS` (default 60 s) and removes IPs that are no longer relevant.

## Eviction rules

An IP is kept if either:

- it is currently blocked, or
- its most recent request is not older than `HTTP_LIMITER_IDLE_TIME_SECS` (default 300 s).

Everything else, including entries with no timestamps, is removed.

## Notes

- The task starts with the server and is stopped when the server stops.
- If the internal lock is poisoned the pass is skipped and an error is logged via `tracing`.
- The cleanup holds the write lock only for the duration of one pass.
