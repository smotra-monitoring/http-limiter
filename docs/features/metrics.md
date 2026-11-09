# Metrics

Endpoint: `GET /metrics` returns metrics in the Prometheus text exposition format.

| Metric | Type | Description |
|---|---|---|
| `http_limiter_unique_ips_total` | gauge | IPs currently tracked in memory |
| `http_limiter_http_requests_total` | counter | Requests handled by `/check` |
| `http_limiter_ip_block_events_initiated_total` | counter | Blocks started because of rate limiting |
| `http_limiter_unique_ip_limit_exceeded_total` | counter | Requests rejected by the [unique IP cap](unique-ip-cap.md) |
| `http_limiter_ip_bucket_utiilization_spill_over` | counter | Per-IP timestamp bucket overflows; anything above 0 needs investigation |
| `http_limiter_requests_while_blocked_total` | counter | Requests rejected because the IP was already blocked |

Counters reset on restart. The `utiilization` spelling is the current metric name.

## Prometheus scrape example

```yaml
scrape_configs:
  - job_name: http-limiter
    static_configs:
      - targets: ["localhost:8082"]
```
