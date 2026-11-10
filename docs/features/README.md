# Features

| Feature | Description |
|---|---|
| [Rate limiting](rate-limiting.md) | Sliding-window request limit per IP |
| [IP blocking](ip-blocking.md) | Temporary block after exceeding the limit |
| [Unique IP cap](unique-ip-cap.md) | Upper bound on tracked IPs |
| [Idle cleanup](idle-cleanup.md) | Background eviction of inactive IPs |
| [Metrics](metrics.md) | Prometheus metrics endpoint |
| [Health checks](health-checks.md) | Liveness and readiness probes |
| [Graceful shutdown](graceful-shutdown.md) | Drains in-flight requests on SIGINT/SIGTERM |
| [Configuration](configuration.md) | Environment variables and validation rules |

Back to the [project README](../README.md).
