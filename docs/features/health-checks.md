# Health checks

| Endpoint | Returns `200` when | Returns `503` when |
|---|---|---|
| `GET /healthz/live` | The state lock can be acquired without blocking | The lock is held for writing or poisoned |
| `GET /healthz/ready` | Always | Never |

Kubernetes example:

```yaml
livenessProbe:
  httpGet: { path: /healthz/live, port: 8082 }
readinessProbe:
  httpGet: { path: /healthz/ready, port: 8082 }
```

Because the liveness probe uses a non-blocking read, it can briefly report `503` while a request or cleanup pass holds the write lock. Configure the probe with a failure threshold greater than 1.
