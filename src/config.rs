use std::{net::SocketAddr, str::FromStr, time::Duration};

#[derive(Debug, Clone)]
pub struct Config {
    pub bind_addr: SocketAddr,
    pub timestamp_bucket_size: usize,
    pub requests_per_window: usize,
    pub rate_limit_window: Duration,
    /// How long an IP stays rejected after exceeding the limit; zero disables blocking.
    pub block_time: Duration,
    pub max_unique_ips: usize,
    pub idle_time: Duration,
    pub cleanup_interval: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bind_addr: SocketAddr::from(([0, 0, 0, 0], 8082)),
            max_unique_ips: 200_000,
            requests_per_window: 20,
            timestamp_bucket_size: 20 + 10,
            rate_limit_window: Duration::from_secs(60),
            block_time: Duration::from_secs(120),
            idle_time: Duration::from_secs(300),
            cleanup_interval: Duration::from_secs(60),
        }
    }
}

impl Config {
    /// Defaults overridden by `HTTP_LIMITER_*` environment variables; unset or unparsable values are ignored.
    pub fn from_env() -> Self {
        let d = Self::default();
        let requests_per_window =
            env_parse("HTTP_LIMITER_REQUESTS_PER_WINDOW").unwrap_or(d.requests_per_window);
        Self {
            bind_addr: env_parse("HTTP_LIMITER_BIND_ADDR").unwrap_or(d.bind_addr),
            requests_per_window,
            timestamp_bucket_size: requests_per_window + 10,
            rate_limit_window: env_parse("HTTP_LIMITER_RATE_LIMIT_WINDOW_SECS")
                .map(Duration::from_secs)
                .unwrap_or(d.rate_limit_window),
            block_time: env_parse("HTTP_LIMITER_BLOCK_TIME_SECS")
                .map(Duration::from_secs)
                .unwrap_or(d.block_time),
            max_unique_ips: env_parse("HTTP_LIMITER_MAX_UNIQUE_IPS").unwrap_or(d.max_unique_ips),
            idle_time: env_parse("HTTP_LIMITER_IDLE_TIME_SECS")
                .map(Duration::from_secs)
                .unwrap_or(d.idle_time),
            cleanup_interval: env_parse("HTTP_LIMITER_CLEANUP_INTERVAL_SECS")
                .map(Duration::from_secs)
                .unwrap_or(d.cleanup_interval),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.timestamp_bucket_size <= self.requests_per_window {
            return Err("requests_per_window must be less than timestamp_bucket_size".into());
        }
        if self.block_time == Duration::from_secs(0) {
            return Err("block_time must be greater than zero".into());
        }
        if self.block_time < self.rate_limit_window {
            return Err("block_time must be greater than rate_limit_window".into());
        }
        Ok(())
    }
}

fn env_parse<T: FromStr>(key: &str) -> Option<T> {
    std::env::var(key).ok()?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        assert_eq!(Config::default().validate(), Ok(()));
    }

    #[test]
    fn rejects_requests_per_window_equal_to_timestamp_bucket_size() {
        let cfg = Config {
            requests_per_window: 50,
            timestamp_bucket_size: 50,
            ..Config::default()
        };
        assert!(
            cfg.validate().is_err(),
            "Requests_per_window must be less than timestamp_bucket_size"
        );
    }

    #[test]
    fn rejects_requests_per_window_above_timestamp_bucket_size() {
        let cfg = Config {
            requests_per_window: 51,
            timestamp_bucket_size: 50,
            ..Config::default()
        };
        assert!(
            cfg.validate().is_err(),
            "Requests_per_window must be less than timestamp_bucket_size"
        );
    }

    #[test]
    fn accepts_requests_per_window_just_below_timestamp_bucket_size() {
        let cfg = Config {
            requests_per_window: 49,
            timestamp_bucket_size: 50,
            ..Config::default()
        };
        assert_eq!(cfg.validate(), Ok(()));
    }

    #[test]
    fn accepts_max_unique_ips_just_above_lower_bound() {
        let cfg = Config {
            max_unique_ips: 100_001,
            ..Config::default()
        };
        assert_eq!(cfg.validate(), Ok(()));
    }

    #[test]
    fn rejects_zero_block_time() {
        let cfg = Config {
            block_time: Duration::ZERO,
            ..Config::default()
        };
        assert_eq!(
            cfg.validate(),
            Err("block_time must be greater than zero".to_string())
        );
    }

    #[test]
    fn accepts_minimal_block_time() {
        let cfg = Config {
            rate_limit_window: Duration::from_secs(1),
            block_time: Duration::from_secs(1),
            ..Config::default()
        };
        assert_eq!(cfg.validate(), Ok(()));
    }

    #[test]
    fn reports_requests_per_window_error_first() {
        let cfg = Config {
            requests_per_window: 100,
            timestamp_bucket_size: 50,
            max_unique_ips: 1,
            block_time: Duration::ZERO,
            ..Config::default()
        };
        assert!(cfg.validate().unwrap_err().contains("requests_per_window"));
    }
}
