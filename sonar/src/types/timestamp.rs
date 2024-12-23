/// A UNIX timestamp in UTC with nanosecond precision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Timestamp {
    seconds: u64,
    nanos: u32,
}

impl Timestamp {
    /// Creates a new `Timestamp` from the given seconds and nanoseconds.
    pub fn new(mut seconds: u64, mut nanos: u32) -> Self {
        seconds += nanos as u64 / 1_000_000_000;
        nanos %= 1_000_000_000;
        Self { seconds, nanos }
    }

    pub fn from_millis(millis: u64) -> Self {
        Self::new(millis / 1000, (millis % 1000) as u32 * 1_000_000)
    }

    pub fn from_seconds(seconds: u64) -> Self {
        Self::new(seconds, 0)
    }

    pub fn seconds(&self) -> u64 {
        self.seconds
    }

    pub fn nanos(&self) -> u32 {
        self.nanos
    }

    pub fn seconds_f64(&self) -> f64 {
        self.seconds as f64 + self.nanos as f64 / 1_000_000_000.0
    }

    pub fn from_duration(duration: std::time::Duration) -> Self {
        Self::new(duration.as_secs(), duration.subsec_nanos())
    }

    /// Returns the timestamp as a duration since the UNIX epoch.
    pub fn as_duration(&self) -> std::time::Duration {
        std::time::Duration::new(self.seconds, self.nanos)
    }

    pub fn elapsed(&self) -> std::time::Duration {
        let self_dur = self.as_duration();
        let now_dur = Self::now().as_duration();
        now_dur - self_dur
    }

    pub fn now() -> Self {
        let now = std::time::SystemTime::now();
        let duration = now.duration_since(std::time::UNIX_EPOCH).unwrap();
        Self::new(duration.as_secs(), duration.subsec_nanos())
    }
}
