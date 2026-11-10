# Graceful shutdown

The server stops on `SIGINT` (Ctrl-C) or, on Unix, `SIGTERM` (the signal sent by `docker stop` and Kubernetes).

## Behavior

1. The listener stops accepting new connections.
2. In-flight requests are allowed to finish.
3. The [idle cleanup](idle-cleanup.md) task is aborted.
4. The process exits with status 0.

## Notes

- There is no shutdown timeout: a connection that never completes delays exit until the orchestrator sends `SIGKILL`.
- Rate-limit state is in memory only and is lost on exit.
