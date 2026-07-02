use commonware_runtime::telemetry::metrics::{
    Metric, Registered, Registration, encoding, raw, registry::Registry, status,
};
use commonware_runtime::{Clock, Metrics, Name, Supervisor};
use futures::Future;
use std::any::Any;
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

// `commonware_runtime::Clock` requires the governor clock traits as supertraits.
impl governor::clock::Clock for MockClock {
    type Instant = SystemTime;

    fn now(&self) -> Self::Instant {
        *self.current_time.lock().unwrap()
    }
}

impl governor::clock::ReasonablyRealtime for MockClock {}

impl MockClock {
    /// Prefix `name` with this context's accumulated label scope.
    fn prefixed(&self, name: &str) -> String {
        if self.label.is_empty() {
            name.to_string()
        } else {
            format!("{}_{}", self.label, name)
        }
    }
}

impl Supervisor for MockClock {
    fn name(&self) -> Name {
        Name {
            label: self.label.clone(),
            attributes: Vec::new(),
        }
    }

    fn child(&self, label: &'static str) -> Self {
        Self {
            current_time: self.current_time.clone(),
            label: self.prefixed(label),
            registry: self.registry.clone(),
        }
    }

    fn with_attribute(self, _key: &'static str, _value: impl std::fmt::Display) -> Self {
        // Attributes are not asserted on in tests; ignore them.
        self
    }
}

impl Metrics for MockClock {
    fn register<N: Into<String>, H: Into<String>, M: Metric>(
        &self,
        name: N,
        help: H,
        metric: M,
    ) -> Registered<M> {
        let name = self.prefixed(&name.into());
        let help = help.into();

        // The runtime's own registry is `pub(crate)`, so we encode through a plain
        // prometheus `Registry` instead (re-exported via `telemetry::metrics`).
        // `register` consumes its metric by value, but the concrete metric types are
        // internally `Arc`-shared, so a clone registered for encoding stays in sync
        // with the returned handle.
        // `M: Metric` is `'static`, which lets us recover the concrete type.
        let any = &metric as &dyn Any;
        let mut registry = self.registry.lock().unwrap();
        if let Some(counter) = any.downcast_ref::<raw::Counter>() {
            registry.register(name, help, counter.clone());
        } else if let Some(histogram) = any.downcast_ref::<raw::Histogram>() {
            registry.register(name, help, histogram.clone());
        } else if let Some(status) = any.downcast_ref::<status::Raw>() {
            registry.register(name, help, status.clone());
        }
        drop(registry);

        Registered::with_registration(metric, Registration::from(()))
    }

    fn encode(&self) -> String {
        let mut buffer = String::new();
        encoding::text::encode(&mut buffer, &self.registry.lock().unwrap())
            .expect("metrics encoding failed");
        buffer
    }
}

impl Default for MockClock {
    fn default() -> Self {
        Self::new()
    }
}
