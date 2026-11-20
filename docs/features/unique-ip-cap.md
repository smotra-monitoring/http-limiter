# Unique IP cap

Memory use is bounded by `HTTP_LIMITER_MAX_UNIQUE_IPS` (default 200000).

## Memory estimate

Estimated from the data structures with the default limits (20 requests per window, bucket of 30), 64-bit Linux. Add about 5-10 MB for the runtime and 10-20% for allocator fragmentation.

| Tracked IPs | Light (1-4 requests, about 190 B/IP) | Heavy (9-16 requests, about 380 B/IP) | Worst case (17+ requests, about 640 B/IP) |
|---|---|---|---|
| 100,000 | about 19 MB | about 38 MB | about 64 MB |
| 200,000 | about 38 MB | about 76 MB | about 128 MB |
| 500,000 | about 95 MB | about 191 MB | about 320 MB |

Per-IP timestamp buffers only grow and are released when [idle cleanup](idle-cleanup.md) removes the IP.

## Behavior

- Once that many IPs are tracked, requests from **new** IPs receive `429` with `Retry-After` equal to the cleanup interval.
- Already tracked IPs are unaffected.
- Slots are freed by [idle cleanup](idle-cleanup.md), so a retry after one cleanup interval may succeed.

## Metrics

- `http_limiter_tracked_ip_current`: currently tracked IPs
- `http_limiter_unique_ip_limit_exceeded_total`: requests rejected because of the cap

A growing `unique_ip_limit_exceeded_total` means the cap is too low or the service is under a many-source flood.
