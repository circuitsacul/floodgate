use std::{
    hash::Hash,
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, SystemTime},
};

use dashmap::{mapref::one::RefMut, DashMap};

use crate::jumping_window::JumpingWindow;

pub(crate) struct Mapping<K: Eq + Hash + Clone + Send + Sync> {
    right: DashMap<K, JumpingWindow>,
    left: DashMap<K, JumpingWindow>,
    is_right_current: AtomicBool,
}

impl<K: Eq + Hash + Clone + Send + Sync> Default for Mapping<K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Eq + Hash + Clone + Send + Sync> Mapping<K> {
    pub(crate) fn new() -> Self {
        Self {
            left: DashMap::new(),
            right: DashMap::new(),
            is_right_current: AtomicBool::new(true),
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

    pub(crate) fn cycle(&self) {
        let start = SystemTime::now();
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
        println!("Cycled: {:?}", start.elapsed());
    }
}
