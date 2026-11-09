use std::{net::SocketAddr, str::FromStr, time::Duration};

#[derive(Debug, Clone)]
pub struct Config {
    pub bind_addr: SocketAddr,
    pub max_timestamps: usize,
    pub requests_per_window: usize,
    pub rate_limit_window: Duration,
    pub max_unique_ips: usize,
    pub idle_time: Duration,
    pub cleanup_interval: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bind_addr: SocketAddr::from(([0, 0, 0, 0], 8080)),
            max_unique_ips: 200_000,
            max_timestamps: 50,
            requests_per_window: 10,
            rate_limit_window: Duration::from_secs(60),
            idle_time: Duration::from_secs(300),
            cleanup_interval: Duration::from_secs(60),
        }
    }
}

impl Config {
    /// Defaults overridden by `HTTP_LIMITER_*` environment variables; unset or unparsable values are ignored.
    pub fn from_env() -> Self {
        let d = Self::default();
        Self {
            bind_addr: env_parse("HTTP_LIMITER_BIND_ADDR").unwrap_or(d.bind_addr),
            max_timestamps: env_parse("HTTP_LIMITER_MAX_TIMESTAMPS").unwrap_or(d.max_timestamps),
            requests_per_window: env_parse("HTTP_LIMITER_REQUESTS_PER_WINDOW")
                .unwrap_or(d.requests_per_window),
            rate_limit_window: env_parse("HTTP_LIMITER_RATE_LIMIT_WINDOW_SECS")
                .map(Duration::from_secs)
                .unwrap_or(d.rate_limit_window),
            max_unique_ips: env_parse("HTTP_LIMITER_MAX_UNIQUE_IPS").unwrap_or(d.max_unique_ips),
            idle_time: env_parse("HTTP_LIMITER_IDLE_TIME_SECS")
                .map(Duration::from_secs)
                .unwrap_or(d.idle_time),
            cleanup_interval: env_parse("HTTP_LIMITER_CLEANUP_INTERVAL_SECS")
                .map(Duration::from_secs)
                .unwrap_or(d.cleanup_interval),
        }
    }
}

fn env_parse<T: FromStr>(key: &str) -> Option<T> {
    std::env::var(key).ok()?.parse().ok()
}
