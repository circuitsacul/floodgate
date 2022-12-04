use std::{
    hash::Hash,
    sync::Arc,
    thread::{self, sleep},
    time::Duration,
};

use dashmap::mapref::one::RefMut;

use crate::{mapping::Mapping, JumpingWindow};

/// Similar to `floodgate::FixedMapping`, except that each cooldown can have
/// a different capacity and/or period.
///
/// For some method documentation, please see `floodgate::JumpingWindow`.
pub struct DynamicMapping<K: Eq + Hash + Clone + Send + Sync + 'static> {
    mapping: Mapping<K>,
    cycle_period: Duration,
}

impl<K: Eq + Hash + Clone + Send + Sync + 'static> DynamicMapping<K> {
    /// Create a new DynamicMapping.
    ///
    /// # Arguments
    /// * `cycle_period` - How often to cycle the mapping. Must be greater than
    /// the period of any cooldown this mapping contains.
    pub fn new(cycle_period: Duration) -> Self {
        Self {
            mapping: Mapping::new(cycle_period),
            cycle_period,
        }
    }

    fn get_bucket(&self, key: &K, capacity: u64, period: Duration) -> RefMut<K, JumpingWindow> {
        debug_assert!(period <= self.cycle_period);
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

    /// Cycles the mapping. Returns `true` if it cycled, or `false` if not.
    pub fn cycle(&self) -> bool {
        self.mapping.cycle(None)
    }

    /// Start the background cycler. Failing to do this will result in a memory leak.
    ///
    /// If, for some reason, you don't want to use the default cycler, you must manually call
    /// the `.cycle` method on the mapping periodically.
    ///
    /// # Arguments
    /// * `mapping` - The DynamicMapping, wrapped in an Arc.
    pub fn start(mapping: Arc<Self>) {
        thread::spawn(move || loop {
            sleep(mapping.cycle_period);
            mapping.mapping.cycle(None);
        });
    }
}
