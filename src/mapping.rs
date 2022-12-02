use std::{
    hash::Hash,
    time::{Duration, SystemTime},
};

use dashmap::{mapref::one::RefMut, DashMap};

use crate::jumping_window::JumpingWindow;

pub(crate) struct Mapping<K: Eq + Hash + Clone + Send + Sync> {
    previous: DashMap<K, JumpingWindow>,
    current: DashMap<K, JumpingWindow>,
    cycle_period: Duration,
    last_cycle: SystemTime,
}

impl<K: Eq + Hash + Clone + Send + Sync> Mapping<K> {
    pub(crate) fn new(cycle_period: Duration) -> Self {
        Self {
            previous: DashMap::new(),
            current: DashMap::new(),
            cycle_period,
            last_cycle: SystemTime::now(),
        }
    }

    pub(crate) fn get_bucket(
        &self,
        key: &K,
        capacity: u64,
        period: Duration,
    ) -> RefMut<K, JumpingWindow> {
        if let Some((key, bucket)) = self.previous.remove(key) {
            self.current.insert(key, bucket);
        }

        let bucket = self.current.get_mut(key).unwrap_or_else(|| {
            let bucket = JumpingWindow::new(capacity, period);
            self.current.insert(key.clone(), bucket);
            self.current.get_mut(key).unwrap()
        });

        bucket
    }

    pub(crate) fn should_cycle(&self, now: Option<SystemTime>) -> bool {
        let now = now.unwrap_or_else(SystemTime::now);
        now.duration_since(self.last_cycle).unwrap() > self.cycle_period
    }

    pub(crate) fn cycle(&mut self, now: Option<SystemTime>) {
        std::mem::swap(&mut self.previous, &mut self.current);
        self.current = DashMap::new();

        self.last_cycle = now.unwrap_or_else(SystemTime::now);
    }
}
