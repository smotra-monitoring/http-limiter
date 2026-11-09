use std::{
    collections::{BTreeMap, VecDeque},
    net::IpAddr,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard, atomic::AtomicU64},
    time::Instant,
};

use crate::config::Config;
use crate::error::Error;

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
    pub(crate) ip_bucket_utiilization_spill_over: AtomicU64,
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
            ip_bucket_utiilization_spill_over: AtomicU64::new(0),
        })
    }

    pub(crate) fn records_read(&self) -> Result<RwLockReadGuard<'_, RecordsMap>, Error> {
        self.ip_records.read().map_err(|_| Error::LockPoisoned)
    }

    pub(crate) fn records_write(&self) -> Result<RwLockWriteGuard<'_, RecordsMap>, Error> {
        self.ip_records.write().map_err(|_| Error::LockPoisoned)
    }
}
