use std::time::{Duration, SystemTime};

#[derive(Debug)]
pub struct JumpingWindow {
    capacity: u64,
    period: Duration,

    last_reset: SystemTime,
    tokens: u64,
}

impl JumpingWindow {
    pub fn new(capacity: u64, period: Duration) -> Self {
        Self {
            capacity,
            period,
            last_reset: SystemTime::now(),
            tokens: capacity,
        }
    }

    pub fn tokens(&mut self, now: Option<SystemTime>) -> u64 {
        let now = now.unwrap_or_else(SystemTime::now);

        if now
            .duration_since(self.last_reset)
            .expect("time went backwards")
            > self.period
        {
            self.reset(Some(now));
        }

        self.tokens
    }

    pub fn next_reset(&mut self, now: Option<SystemTime>) -> Duration {
        let now = now.unwrap_or_else(SystemTime::now);
        self.period - now.duration_since(self.last_reset).unwrap()
    }

    pub fn retry_after(&mut self, now: Option<SystemTime>) -> Option<Duration> {
        if self.tokens(now) == 0 {
            Some(self.next_reset(now))
        } else {
            None
        }
    }

    pub fn can_trigger(&mut self, now: Option<SystemTime>) -> bool {
        self.tokens(now) != 0
    }

    pub fn trigger(&mut self, now: Option<SystemTime>) -> Option<Duration> {
        let tokens = self.tokens(now);

        if tokens == 0 {
            Some(self.next_reset(now))
        } else {
            self.tokens -= 1;
            None
        }
    }

    pub fn reset(&mut self, now: Option<SystemTime>) {
        self.tokens = self.capacity;
        self.last_reset = now.unwrap_or_else(SystemTime::now);
    }
}
