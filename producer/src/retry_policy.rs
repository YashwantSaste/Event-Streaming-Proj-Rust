use std::time::Duration;

#[derive(Debug, Clone)]
pub struct RetryPolicy {
    max_attempts: u32,
    delay: Duration,
}

impl RetryPolicy {
    pub fn new(max_attempts: u32, delay: Duration) -> Self {
        Self {
            max_attempts: max_attempts.max(1),
            delay,
        }
    }

    pub fn max_attempts(&self) -> u32 {
        self.max_attempts
    }

    pub fn delay(&self) -> Duration {
        self.delay
    }
}
