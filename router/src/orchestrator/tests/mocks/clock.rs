use commonware_runtime::{Clock, Metrics};
use futures::Future;
use prometheus_client::registry::{Metric, Registry};
use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

/// Mock clock implementation for testing purposes.
///
/// This implementation provides a configurable mock that can be used
/// for unit testing without requiring real time-based operations. It allows
/// for predictable behavior and easy test scenario setup.
///
/// It also implements `Metrics` with a registry shared across clones, so tests
/// can build an orchestrator from one clone and assert on `encode()` output
/// from another.
#[derive(Clone)]
pub struct MockClock {
    /// The current time for the mock clock
    current_time: Arc<Mutex<SystemTime>>,
    /// Label scope applied to registered metric names, mirroring the runtime contexts
    label: String,
    /// Metric registry shared across clones
    registry: Arc<Mutex<Registry>>,
}

impl fmt::Debug for MockClock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MockClock")
            .field("current_time", &self.current_time)
            .field("label", &self.label)
            .finish_non_exhaustive()
    }
}

impl MockClock {
    /// Creates a new MockClock with the current system time.
    ///
    /// This constructor creates a mock clock that starts with
    /// the current system time.
    ///
    /// # Returns
    /// * `Self` - The new MockClock instance
    pub fn new() -> Self {
        Self::with_time(SystemTime::now())
    }

    /// Creates a new MockClock with a specific start time.
    ///
    /// This constructor allows for precise control over the
    /// initial time of the mock clock.
    ///
    /// # Arguments
    /// * `start_time` - The initial time for the mock clock
    ///
    /// # Returns
    /// * `Self` - The new MockClock instance
    #[allow(dead_code)]
    pub fn with_time(start_time: SystemTime) -> Self {
        Self {
            current_time: Arc::new(Mutex::new(start_time)),
            label: String::new(),
            registry: Arc::new(Mutex::new(Registry::default())),
        }
    }

    /// Advances the mock clock by the specified duration.
    ///
    /// This method allows for time manipulation during testing.
    ///
    /// # Arguments
    /// * `duration` - The duration to advance the clock by
    #[allow(dead_code)]
    pub fn advance(&self, duration: Duration) {
        let mut time = self.current_time.lock().unwrap();
        *time += duration;
    }

    /// Sets the mock clock to a specific time.
    ///
    /// This method allows for precise time control during testing.
    ///
    /// # Arguments
    /// * `time` - The new time to set
    #[allow(dead_code)]
    pub fn set_time(&self, time: SystemTime) {
        let mut current = self.current_time.lock().unwrap();
        *current = time;
    }

    /// Gets the current time of the mock clock.
    ///
    /// This method is useful for testing to verify the current
    /// time state of the mock clock.
    ///
    /// # Returns
    /// * `SystemTime` - The current time of the mock clock
    #[allow(dead_code)]
    pub fn get_current_time(&self) -> SystemTime {
        *self.current_time.lock().unwrap()
    }
}

impl Clock for MockClock {
    fn current(&self) -> SystemTime {
        *self.current_time.lock().unwrap()
    }

    fn sleep_until(&self, target: SystemTime) -> impl Future<Output = ()> + Send + 'static {
        let current = self.current();
        async move {
            if current < target {
                let sleep_duration = target.duration_since(current).unwrap_or(Duration::ZERO);
                tokio::time::sleep(sleep_duration).await;
            }
        }
    }

    #[allow(clippy::manual_async_fn)]
    fn sleep(&self, duration: Duration) -> impl Future<Output = ()> + Send + 'static {
        async move {
            tokio::time::sleep(duration).await;
        }
    }
}

impl Metrics for MockClock {
    fn label(&self) -> String {
        self.label.clone()
    }

    fn with_label(&self, label: &str) -> Self {
        Self {
            current_time: self.current_time.clone(),
            label: self.scoped_label(label),
            registry: self.registry.clone(),
        }
    }

    fn register<N: Into<String>, H: Into<String>>(&self, name: N, help: H, metric: impl Metric) {
        let name = self.scoped_label(&name.into());
        self.registry.lock().unwrap().register(name, help, metric);
    }

    fn encode(&self) -> String {
        let mut buffer = String::new();
        prometheus_client::encoding::text::encode(&mut buffer, &self.registry.lock().unwrap())
            .expect("metrics encoding failed");
        buffer
    }
}

impl Default for MockClock {
    fn default() -> Self {
        Self::new()
    }
}
