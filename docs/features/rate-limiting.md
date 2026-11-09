# Rate limiting

Endpoint: `POST /check` with body `{"ip": "<IPv4 or IPv6>"}`.

## Behavior

For every IP the service keeps the timestamps of its recent allowed requests.

1. Timestamps older than `HTTP_LIMITER_RATE_LIMIT_WINDOW_SECS` are discarded (sliding window).
2. If the remaining count is below `HTTP_LIMITER_REQUESTS_PER_WINDOW`, the request is recorded and `200 OK` is returned.
3. Otherwise the IP is [blocked](ip-blocking.md) and `429 Too Many Requests` is returned.

With the defaults (20 requests / 60 s), requests 1-20 succeed and request 21 within the window triggers a block.

## Responses

| Status | Condition |
|---|---|
| `200 OK` | Request allowed |
| `429 Too Many Requests` | Limit exceeded, IP is blocked, or [unique IP cap](unique-ip-cap.md) reached. Carries `Retry-After` (seconds, at least 1) |
| `422` / `400` / `415` | Body is not valid JSON of the expected shape (rejected by axum) |
| `503 Service Unavailable` | Internal state lock poisoned |

## Per-IP storage

Each IP stores at most `requests_per_window + 10` timestamps. If this bucket ever fills, the oldest timestamp is dropped and the `http_limiter_ip_bucket_utiilization_spill_over` metric increases. This should not happen in normal operation and needs investigation if it does.
