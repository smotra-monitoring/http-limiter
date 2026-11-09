# IP blocking

When an IP exceeds the [rate limit](rate-limiting.md), it is blocked for `HTTP_LIMITER_BLOCK_TIME_SECS` (default 120 s).

## Behavior

- The request that exceeds the limit starts the block and receives `429` with `Retry-After` equal to the block time.
- While blocked, every request from that IP receives `429` with `Retry-After` set to the remaining block time.
- Requests made while blocked are not recorded, so they do not extend the block.
- When the block expires, the IP is unblocked on its next request and continues to use its existing window.
- Blocked IPs are never evicted by [idle cleanup](idle-cleanup.md) until the block expires.

## Constraint

`block_time` must be at least `rate_limit_window`; see [Configuration](configuration.md).

## Metrics

- `http_limiter_ip_block_events_initiated_total`: blocks started
- `http_limiter_requests_while_blocked_total`: requests rejected during a block
