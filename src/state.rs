use std::{
    collections::{BTreeMap, VecDeque},
    net::IpAddr,
    sync::{Arc, RwLock, atomic::AtomicU64},
    time::Instant,
};

use crate::config::Config;

pub struct AppState {
    pub(crate) config: Config,
    pub(crate) ip_records: RwLock<BTreeMap<IpAddr, VecDeque<Instant>>>,
    pub(crate) total_requests: AtomicU64,
    pub(crate) rate_limit_exceeded_total: AtomicU64,
    pub(crate) unique_ip_limit_exceeded_total: AtomicU64,
}

pub type SharedState = Arc<AppState>;

impl AppState {
    pub fn new(config: Config) -> SharedState {
        Arc::new(Self {
            config,
            ip_records: RwLock::new(BTreeMap::new()),
            total_requests: AtomicU64::new(0),
            rate_limit_exceeded_total: AtomicU64::new(0),
            unique_ip_limit_exceeded_total: AtomicU64::new(0),
        })
    }
}
