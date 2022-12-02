use std::{hash::Hash, time::Duration};

use dashmap::mapref::one::RefMut;

use crate::{mapping::Mapping, JumpingWindow};

#[cfg(feature = "tokio")]
use {
    std::{sync::Arc, time::SystemTime},
    tokio::sync::RwLock,
};

pub struct FixedMapping<K: Eq + Hash + Clone + Send + Sync + 'static> {
    pub(crate) mapping: Mapping<K>,
    capacity: u64,
    period: Duration,
}

impl<K: Eq + Hash + Clone + Send + Sync + 'static> FixedMapping<K> {
    pub fn new(capacity: u64, period: Duration) -> Self {
        Self {
            capacity,
            period,
            mapping: Mapping::new(period),
        }
    }

    fn get_bucket(&self, key: &K) -> RefMut<K, JumpingWindow> {
        self.mapping.get_bucket(key, self.capacity, self.period)
    }

    pub fn tokens(&self, key: &K) -> u64 {
        self.get_bucket(key).tokens(None)
    }

    pub fn next_reset(&self, key: &K) -> Duration {
        self.get_bucket(key).next_reset(None)
    }

    pub fn retry_after(&self, key: &K) -> Option<Duration> {
        self.get_bucket(key).retry_after(None)
    }

    pub fn can_trigger(&self, key: &K) -> bool {
        self.get_bucket(key).can_trigger(None)
    }

    pub fn trigger(&self, key: &K) -> Option<Duration> {
        self.get_bucket(key).trigger(None)
    }

    pub fn reset(&self, key: &K) {
        self.get_bucket(key).reset(None)
    }

    #[cfg(feature = "tokio")]
    pub fn start(mapping: Arc<RwLock<Self>>) {
        tokio::spawn(async move {
            loop {
                let now = SystemTime::now();

                let should_cycle = {
                    let mapping = mapping.read().await;
                    mapping.mapping.should_cycle(Some(now))
                };

                if should_cycle {
                    mapping.write().await.mapping.cycle(Some(now));
                }
            }
        });
    }
}
