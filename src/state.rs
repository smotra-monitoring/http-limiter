use std::{
    collections::{BTreeMap, VecDeque},
    fmt,
    net::IpAddr,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard, atomic::AtomicU64},
    time::Instant,
};

use crate::config::Config;

#[derive(Default)]
pub(crate) struct IpRecord {
    pub(crate) timestamps: VecDeque<Instant>,
    pub(crate) blocked_until: Option<Instant>,
}

pub(crate) type RecordsMap = BTreeMap<IpAddr, IpRecord>;

pub struct AppState {
    pub(crate) config: Config,
    pub(crate) ip_records: RwLock<RecordsMap>,
    pub(crate) total_requests: AtomicU64,
    pub(crate) ip_block_initiated: AtomicU64,
    pub(crate) requests_while_blocked: AtomicU64,
    pub(crate) unique_ip_limit_exceeded_total: AtomicU64,
}

/// `ip_records` lock was poisoned by a panic in another thread.
#[derive(Debug)]
pub(crate) struct LockPoisoned;

impl fmt::Display for LockPoisoned {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ip_records lock poisoned by a panic in another thread")
    }
}

pub type SharedState = Arc<AppState>;

impl AppState {
    pub fn new(config: Config) -> SharedState {
        Arc::new(Self {
            config,
            ip_records: RwLock::new(BTreeMap::new()),
            total_requests: AtomicU64::new(0),
            ip_block_initiated: AtomicU64::new(0),
            requests_while_blocked: AtomicU64::new(0),
            unique_ip_limit_exceeded_total: AtomicU64::new(0),
        })
    }

    pub(crate) fn records_read(&self) -> Result<RwLockReadGuard<'_, RecordsMap>, LockPoisoned> {
        self.ip_records.read().map_err(|_| LockPoisoned)
    }

    pub(crate) fn records_write(&self) -> Result<RwLockWriteGuard<'_, RecordsMap>, LockPoisoned> {
        self.ip_records.write().map_err(|_| LockPoisoned)
    }
}
