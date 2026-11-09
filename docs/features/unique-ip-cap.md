# Unique IP cap

Memory use is bounded by `HTTP_LIMITER_MAX_UNIQUE_IPS` (default 200000).

## Behavior

- Once that many IPs are tracked, requests from **new** IPs receive `429` with `Retry-After` equal to the cleanup interval.
- Already tracked IPs are unaffected.
- Slots are freed by [idle cleanup](idle-cleanup.md), so a retry after one cleanup interval may succeed.

## Metrics

- `http_limiter_unique_ips_total`: currently tracked IPs
- `http_limiter_unique_ip_limit_exceeded_total`: requests rejected because of the cap

A growing `unique_ip_limit_exceeded_total` means the cap is too low or the service is under a many-source flood.
