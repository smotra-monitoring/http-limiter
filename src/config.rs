use std::{net::SocketAddr, str::FromStr, time::Duration};

#[derive(Debug, Clone)]
pub struct Config {
    pub bind_addr: SocketAddr,
    pub max_timestamps: usize,
    pub rate_limit_per_window: usize,
    pub rate_limit_window: Duration,
    pub max_unique_ips: usize,
    pub inactivity_timeout: Duration,
    pub cleanup_interval: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bind_addr: SocketAddr::from(([0, 0, 0, 0], 8080)),
            max_timestamps: 50,
            rate_limit_per_window: 30,
            rate_limit_window: Duration::from_secs(60),
            max_unique_ips: 50_000,
            inactivity_timeout: Duration::from_secs(300),
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
            rate_limit_per_window: env_parse("HTTP_LIMITER_RATE_LIMIT_PER_MIN")
                .unwrap_or(d.rate_limit_per_window),
            rate_limit_window: d.rate_limit_window,
            max_unique_ips: env_parse("HTTP_LIMITER_MAX_UNIQUE_IPS").unwrap_or(d.max_unique_ips),
            inactivity_timeout: env_parse("HTTP_LIMITER_INACTIVITY_TIMEOUT_SECS")
                .map(Duration::from_secs)
                .unwrap_or(d.inactivity_timeout),
            cleanup_interval: env_parse("HTTP_LIMITER_CLEANUP_INTERVAL_SECS")
                .map(Duration::from_secs)
                .unwrap_or(d.cleanup_interval),
        }
    }
}

fn env_parse<T: FromStr>(key: &str) -> Option<T> {
    std::env::var(key).ok()?.parse().ok()
}
