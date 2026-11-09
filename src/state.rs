use std::{
    collections::{BTreeMap, VecDeque},
    net::IpAddr,
    sync::{Arc, RwLock, atomic::AtomicU64},
    time::Instant,
};

use crate::config::Config;

#[derive(Default)]
pub(crate) struct IpRecord {
    pub(crate) timestamps: VecDeque<Instant>,
    pub(crate) blocked_until: Option<Instant>,
}

pub struct AppState {
    pub(crate) config: Config,
    pub(crate) ip_records: RwLock<BTreeMap<IpAddr, IpRecord>>,
    pub(crate) total_requests: AtomicU64,
    pub(crate) ip_block_initiated: AtomicU64,
    pub(crate) requests_while_blocked: AtomicU64,
    pub(crate) unique_ip_limit_exceeded_total: AtomicU64,
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
}
