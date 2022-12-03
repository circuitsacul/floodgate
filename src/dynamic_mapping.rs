use std::{
    hash::Hash,
    sync::Arc,
    thread::{self, sleep},
    time::Duration,
};

use dashmap::mapref::one::RefMut;

use crate::{mapping::Mapping, JumpingWindow};

pub struct DynamicMapping<K: Eq + Hash + Clone + Send + Sync + 'static> {
    pub(crate) mapping: Mapping<K>,
    pub(crate) max_period: Duration,
}

impl<K: Eq + Hash + Clone + Send + Sync + 'static> DynamicMapping<K> {
    pub fn new(max_period: Duration) -> Self {
        Self {
            max_period,
            mapping: Mapping::new(),
        }
    }

    fn get_bucket(&self, key: &K, capacity: u64, period: Duration) -> RefMut<K, JumpingWindow> {
        debug_assert!(period <= self.max_period);
        self.mapping.get_bucket(key, capacity, period)
    }

    pub fn tokens(&self, key: &K, capacity: u64, period: Duration) -> u64 {
        self.get_bucket(key, capacity, period).tokens(None)
    }

    pub fn next_reset(&self, key: &K, capacity: u64, period: Duration) -> Duration {
        self.get_bucket(key, capacity, period).next_reset(None)
    }

    pub fn retry_after(&self, key: &K, capacity: u64, period: Duration) -> Option<Duration> {
        self.get_bucket(key, capacity, period).retry_after(None)
    }

    pub fn can_trigger(&self, key: &K, capacity: u64, period: Duration) -> bool {
        self.get_bucket(key, capacity, period).can_trigger(None)
    }

    pub fn trigger(&self, key: &K, capacity: u64, period: Duration) -> Option<Duration> {
        self.get_bucket(key, capacity, period).trigger(None)
    }

    pub fn reset(&self, key: &K, capacity: u64, period: Duration) {
        self.get_bucket(key, capacity, period).reset(None)
    }

    pub fn start(mapping: Arc<Self>, cycle_period: Option<Duration>) {
        let period = cycle_period.unwrap_or(mapping.max_period);
        thread::spawn(move || loop {
            sleep(period);
            mapping.mapping.cycle();
        });
    }
}
