use std::{sync::Arc, time::Instant};

use tokio::task::JoinHandle;

use crate::state::{AppState, SharedState};

/// Drops IPs whose last request is older than the inactivity timeout.
pub(crate) fn cleanup_inactive(state: &AppState, now: Instant) {
    let timeout = state.config.idle_time;
    let mut records = state.ip_records.write().unwrap();
    records.retain(|_, record| {
        // Evicting a blocked IP would silently lift its block.
        record.blocked_until.is_some_and(|until| until > now)
            || record
                .timestamps
                .back()
                .is_some_and(|&last_seen| now.duration_since(last_seen) <= timeout)
    });
}

/// Starts the periodic cleanup; abort the returned handle to stop it.
pub fn spawn_cleanup_task(state: &SharedState) -> JoinHandle<()> {
    // let state = Arc::clone(state);
    let state = Arc::clone(state);
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(state.config.cleanup_interval);
        loop {
            interval.tick().await;
            cleanup_inactive(&state, Instant::now());
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::Config, state::IpRecord};
    use std::{collections::VecDeque, net::IpAddr, time::Duration};

    #[test]
    fn removes_only_inactive_and_empty_entries() {
        let state = AppState::new(Config {
            idle_time: Duration::from_secs(10),
            ..Config::default()
        });
        let now = Instant::now();
        let stale: IpAddr = "10.0.0.1".parse().unwrap();
        let block: IpAddr = "10.0.0.2".parse().unwrap();
        let empty: IpAddr = "10.0.0.3".parse().unwrap();
        {
            let mut records = state.ip_records.write().unwrap();
            let with = |timestamps: VecDeque<Instant>| IpRecord {
                timestamps,
                blocked_until: None,
            };
            records.insert(stale, with(VecDeque::from([now - Duration::from_secs(20)])));
            records.insert(block, with(VecDeque::from([now - Duration::from_secs(5)])));
            records.insert(empty, IpRecord::default());
        }

        cleanup_inactive(&state, now);

        let records = state.ip_records.read().unwrap();
        assert_eq!(records.keys().copied().collect::<Vec<_>>(), vec![block]);
    }

    #[test]
    fn keeps_blocked_ips_even_when_idle_expired() {
        let state = AppState::new(Config {
            idle_time: Duration::from_secs(10),
            ..Config::default()
        });
        let now = Instant::now();
        let blocked: IpAddr = "10.0.0.4".parse().unwrap();
        let expired: IpAddr = "10.0.0.5".parse().unwrap();
        {
            let mut records = state.ip_records.write().unwrap();
            records.insert(
                blocked,
                IpRecord {
                    timestamps: VecDeque::from([now - Duration::from_secs(20)]),
                    blocked_until: Some(now + Duration::from_secs(5)),
                },
            );
            records.insert(
                expired,
                IpRecord {
                    timestamps: VecDeque::from([now - Duration::from_secs(20)]),
                    blocked_until: Some(now - Duration::from_secs(1)),
                },
            );
        }

        cleanup_inactive(&state, now);

        let records = state.ip_records.read().unwrap();
        assert_eq!(records.keys().copied().collect::<Vec<_>>(), vec![blocked]);
    }
}
