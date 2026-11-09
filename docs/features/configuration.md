# Configuration

Settings are read once at startup from environment variables. Unset or unparsable values silently fall back to the default.

| Variable | Type | Default |
|---|---|---|
| `HTTP_LIMITER_BIND_ADDR` | socket address | `0.0.0.0:8082` |
| `HTTP_LIMITER_REQUESTS_PER_WINDOW` | integer | `20` |
| `HTTP_LIMITER_RATE_LIMIT_WINDOW_SECS` | seconds | `60` |
| `HTTP_LIMITER_BLOCK_TIME_SECS` | seconds | `120` |
| `HTTP_LIMITER_MAX_UNIQUE_IPS` | integer | `200000` |
| `HTTP_LIMITER_IDLE_TIME_SECS` | seconds | `300` |
| `HTTP_LIMITER_CLEANUP_INTERVAL_SECS` | seconds | `60` |

The per-IP timestamp bucket size is derived as `requests_per_window + 10` and is not configurable.

## Validation

The process prints `error: <reason>` and exits with a failure code if:

| Rule | Error |
|---|---|
| `block_time` is zero | `block_time must be greater than zero` |
| `block_time` is less than `rate_limit_window` | `block_time must be greater than rate_limit_window` |

Equal values for `block_time` and `rate_limit_window` are accepted.

Related: [Rate limiting](rate-limiting.md), [IP blocking](ip-blocking.md), [Unique IP cap](unique-ip-cap.md), [Idle cleanup](idle-cleanup.md).
