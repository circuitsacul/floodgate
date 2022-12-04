use std::{
    hash::Hash,
    sync::{
        atomic::{AtomicBool, Ordering},
        RwLock,
    },
    time::{Duration, Instant},
};

use dashmap::{mapref::one::RefMut, DashMap};

use crate::jumping_window::JumpingWindow;

pub(crate) struct Mapping<K: Eq + Hash + Clone + Send + Sync> {
    right: DashMap<K, JumpingWindow>,
    left: DashMap<K, JumpingWindow>,
    is_right_current: AtomicBool,
    last_cycle: RwLock<Instant>,
    cycle_period: Duration,
}

impl<K: Eq + Hash + Clone + Send + Sync> Mapping<K> {
    pub(crate) fn new(cycle_period: Duration) -> Self {
        Self {
            left: DashMap::new(),
            right: DashMap::new(),
            is_right_current: AtomicBool::new(true),
            last_cycle: RwLock::new(Instant::now()),
            cycle_period: cycle_period.mul_f32(0.95),
        }
    }

    pub(crate) fn get_bucket(
        &self,
        key: &K,
        capacity: u64,
        period: Duration,
    ) -> RefMut<K, JumpingWindow> {
        let (current, previous) = match self.is_right_current.load(Ordering::Relaxed) {
            true => (&self.right, &self.left),
            false => (&self.left, &self.right),
        };

        if let Some(bucket) = current.get_mut(key) {
            return bucket;
        }

        if let Some((key2, bucket)) = previous.remove(key) {
            current.insert(key2, bucket);
        } else {
            let bucket = JumpingWindow::new(capacity, period);
            current.insert(key.clone(), bucket);
        }

        self.get_bucket(key, capacity, period)
    }

    pub(crate) fn cycle(&self, now: Option<Instant>) -> bool {
        let now = now.unwrap_or_else(Instant::now);
        if now.duration_since(*self.last_cycle.read().unwrap()) < self.cycle_period {
            return false;
        }

        let is_right_current = !self.is_right_current.load(Ordering::Relaxed);
        self.is_right_current
            .store(is_right_current, Ordering::Relaxed);

        {
            match is_right_current {
                true => &self.left,
                false => &self.right,
            }
        }
        .clear();

        *self.last_cycle.write().unwrap() = now;

        true
    }
}
