use std::{
    hash::Hash,
    sync::Arc,
    thread::{self, sleep},
    time::Duration,
};

use dashmap::mapref::one::RefMut;

use crate::{mapping::Mapping, JumpingWindow};

/// A key-based mapping of `floodgate::JumpingWindow`'s.
///
/// For some method documentation, please see `floodgate::JumpingWindow`.
pub struct FixedMapping<K: Eq + Hash + Clone + Send + Sync + 'static> {
    mapping: Mapping<K>,
    capacity: u64,
    period: Duration,
}

impl<K: Eq + Hash + Clone + Send + Sync + 'static> FixedMapping<K> {
    /// Create a new FixedMapping.
    ///
    /// # Arguments
    /// * `capacity` - The capacity of the `JumpingWindow`s.
    /// * `period` - The duration of the `JumpingWindow`s.
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

    /// Cycles the mapping. Returns `true` if it cycled, or `false` if not.
    pub fn cycle(&self) -> bool {
        self.mapping.cycle(None)
    }

    /// Start the background cycler.
    ///
    /// If, for some reason, you don't want to use the default cycler, you must manually call
    /// the `.cycle` method on the mapping periodically.
    ///
    /// # Arguments
    /// * `mapping` - The FixedMapping, wrapped in an Arc.
    /// * `cycle_period` - How often to cycle the mapping. If specified, must be greater than the
    /// mapping's period.
    pub fn start(mapping: Arc<Self>, cycle_period: Option<Duration>) {
        let period = cycle_period.unwrap_or(mapping.period);
        assert!(period >= mapping.period);
        thread::spawn(move || loop {
            sleep(period);
            if !mapping.mapping.cycle(None) {
                eprintln!("Cycler attempted to call the mapping too soon.");
            }
        });
    }
}
