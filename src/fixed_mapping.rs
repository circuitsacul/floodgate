use std::{
    hash::Hash,
    sync::Arc,
    thread::{self, sleep},
    time::Duration,
};

use dashmap::mapref::one::RefMut;

use crate::{mapping::Mapping, JumpingWindow};

pub struct FixedMapping<K: Eq + Hash + Clone + Send + Sync + 'static> {
    mapping: Mapping<K>,
    capacity: u64,
    period: Duration,
}

impl<K: Eq + Hash + Clone + Send + Sync + 'static> FixedMapping<K> {
    pub fn new(capacity: u64, period: Duration) -> Self {
        Self {
            capacity,
            period,
            mapping: Mapping::new(),
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

    pub fn start(mapping: Arc<Self>, cycle_period: Option<Duration>) {
        let period = cycle_period.unwrap_or(mapping.period);
        thread::spawn(move || loop {
            sleep(period);
            mapping.mapping.cycle();
        });
    }
}
