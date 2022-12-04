use std::time::{Duration, SystemTime};

/// A simple ratelimit implementation.
#[derive(Debug)]
pub struct JumpingWindow {
    capacity: u64,
    period: Duration,

    last_reset: SystemTime,
    tokens: u64,
}

impl JumpingWindow {
    /// Create a new JumpingWindow.
    /// 
    /// # Arguments
    /// * `capacity` - How many triggers can occur per window.
    /// * `period` - How long the window is.
    /// 
    /// # Examples
    /// ```
    /// use floodgate::JumpingWindow;
    /// use std::time::Duration;
    /// 
    /// // create a new JumpingWindow that allows 2 triggers per 10 seconds.
    /// let mut cooldown = JumpingWindow::new(2, Duration::from_secs(10));
    /// 
    /// assert_eq!(cooldown.trigger(None), None);
    /// assert_eq!(cooldown.trigger(None), None);
    /// 
    /// // once the triggers are used up, calling .trigger() will return a "retry after" - that
    /// // is, how long before there will be more triggers available.
    /// assert!(matches!(cooldown.trigger(None), Some(_)));
    /// ```
    pub fn new(capacity: u64, period: Duration) -> Self {
        Self {
            capacity,
            period,
            last_reset: SystemTime::now(),
            tokens: capacity,
        }
    }

    /// How many triggers (tokens) are left in the current window.
    /// 
    /// # Arguments
    /// * `now` - Optionally specify the current time.
    /// 
    /// # Examples
    /// ```
    /// use floodgate::JumpingWindow;
    /// use std::time::Duration;
    /// 
    /// let mut cooldown = JumpingWindow::new(1, Duration::from_secs(10));
    /// 
    /// assert_eq!(cooldown.tokens(None), 1);
    /// cooldown.trigger(None);
    /// assert_eq!(cooldown.tokens(None), 0);
    /// ```
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

    /// Return the time until the next reset.
    /// 
    /// # Arguments
    /// * `now` - Optionally specify the current time.
    pub fn next_reset(&mut self, now: Option<SystemTime>) -> Duration {
        let now = now.unwrap_or_else(SystemTime::now);
        let since = now.duration_since(self.last_reset).unwrap();

        if since > self.period {
            Duration::from_secs(0)
        } else {
            self.period - since
        }
    }

    /// Similar to `next_reset`, except that it returns `None` if you still have triggers.
    /// 
    /// # Arguments
    /// * `now` - Optionally specify the current time.
    /// 
    /// # Examples
    /// ```
    /// use floodgate::JumpingWindow;
    /// use std::time::Duration;
    /// 
    /// let mut cooldown = JumpingWindow::new(1, Duration::from_secs(10));
    /// 
    /// assert_eq!(cooldown.retry_after(None), None);
    /// cooldown.trigger(None);
    /// assert!(matches!(cooldown.retry_after(None), Some(_)));
    /// ```
    pub fn retry_after(&mut self, now: Option<SystemTime>) -> Option<Duration> {
        if self.tokens(now) == 0 {
            Some(self.next_reset(now))
        } else {
            None
        }
    }

    /// Returns whether or not there are still available triggers.
    /// 
    /// # Arguments
    /// * `now` - Optionally specify the current time.
    /// 
    /// # Examples
    /// ```
    /// use floodgate::JumpingWindow;
    /// use std::time::Duration;
    /// 
    /// let mut cooldown = JumpingWindow::new(1, Duration::from_secs(10));
    /// 
    /// assert_eq!(cooldown.can_trigger(None), true);
    /// cooldown.trigger(None);
    /// assert_eq!(cooldown.can_trigger(None), false);
    /// ```
    pub fn can_trigger(&mut self, now: Option<SystemTime>) -> bool {
        self.tokens(now) != 0
    }

    /// Trigger the cooldown.
    /// 
    /// # Arguments
    /// * `now` - Optionally specify the current time.
    /// 
    /// # Examples
    /// ```
    /// use floodgate::JumpingWindow;
    /// use std::time::Duration;
    /// 
    /// let mut cooldown = JumpingWindow::new(1, Duration::from_secs(10));
    /// 
    /// assert_eq!(cooldown.trigger(None), None);
    /// assert!(matches!(cooldown.trigger(None), Some(_)));
    /// ```
    pub fn trigger(&mut self, now: Option<SystemTime>) -> Option<Duration> {
        let tokens = self.tokens(now);

        if tokens == 0 {
            Some(self.next_reset(now))
        } else {
            self.tokens -= 1;
            None
        }
    }

    /// Reset the cooldown.
    /// 
    /// # Arguments
    /// * `now` - Optionally specify the current time.
    /// 
    /// # Examples
    /// ```
    /// use floodgate::JumpingWindow;
    /// use std::time::Duration;
    /// 
    /// let mut cooldown = JumpingWindow::new(1, Duration::from_secs(10));
    /// cooldown.trigger(None);
    /// 
    /// assert!(!cooldown.can_trigger(None));
    /// cooldown.reset(None);
    /// assert!(cooldown.can_trigger(None));
    /// ```
    pub fn reset(&mut self, now: Option<SystemTime>) {
        self.tokens = self.capacity;
        self.last_reset = now.unwrap_or_else(SystemTime::now);
    }
}
