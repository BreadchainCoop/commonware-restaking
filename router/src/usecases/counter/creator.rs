use anyhow::Result;
use async_trait::async_trait;
use bytes::{Buf, BufMut};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{error, warn};

use super::provider::CounterProvider;

use crate::creator::core::Creator;
use crate::ingress::types::TaskRequest;
use commonware_codec::{EncodeSize, Read, ReadExt, Write};

/// Task data specific to the counter use case
#[derive(Debug, Clone, PartialEq)]
pub struct CounterTaskData {
    pub var1: String,
    pub var2: String,
    pub var3: String,
}

impl Default for CounterTaskData {
    fn default() -> Self {
        Self {
            var1: "default_var1".to_string(),
            var2: "default_var2".to_string(),
            var3: "default_var3".to_string(),
        }
    }
}

impl Write for CounterTaskData {
    fn write(&self, buf: &mut impl BufMut) {
        // Write each field as length-prefixed string
        (self.var1.len() as u32).write(buf);
        buf.put_slice(self.var1.as_bytes());
        (self.var2.len() as u32).write(buf);
        buf.put_slice(self.var2.as_bytes());
        (self.var3.len() as u32).write(buf);
        buf.put_slice(self.var3.as_bytes());
    }
}

impl Read for CounterTaskData {
    type Cfg = ();

    fn read_cfg(buf: &mut impl Buf, _: &()) -> Result<Self, commonware_codec::Error> {
        // Read each field as length-prefixed string
        let var1_len = u32::read(buf)? as usize;
        if buf.remaining() < var1_len {
            return Err(commonware_codec::Error::EndOfBuffer);
        }
        let mut var1_bytes = vec![0u8; var1_len];
        buf.copy_to_slice(&mut var1_bytes);
        let var1 = String::from_utf8(var1_bytes)
            .map_err(|_| commonware_codec::Error::Invalid("var1", "decoding from utf8 failed"))?;

        let var2_len = u32::read(buf)? as usize;
        if buf.remaining() < var2_len {
            return Err(commonware_codec::Error::EndOfBuffer);
        }
        let mut var2_bytes = vec![0u8; var2_len];
        buf.copy_to_slice(&mut var2_bytes);
        let var2 = String::from_utf8(var2_bytes)
            .map_err(|_| commonware_codec::Error::Invalid("var2", "decoding from utf8 failed"))?;

        let var3_len = u32::read(buf)? as usize;
        if buf.remaining() < var3_len {
            return Err(commonware_codec::Error::EndOfBuffer);
        }
        let mut var3_bytes = vec![0u8; var3_len];
        buf.copy_to_slice(&mut var3_bytes);
        let var3 = String::from_utf8(var3_bytes)
            .map_err(|_| commonware_codec::Error::Invalid("var3", "decoding from utf8 failed"))?;

        Ok(Self { var1, var2, var3 })
    }
}

impl EncodeSize for CounterTaskData {
    fn encode_size(&self) -> usize {
        const LENGTH_PREFIX_SIZE: usize = std::mem::size_of::<u32>();
        LENGTH_PREFIX_SIZE
            + self.var1.len()
            + LENGTH_PREFIX_SIZE
            + self.var2.len()
            + LENGTH_PREFIX_SIZE
            + self.var3.len()
    }
}

/// A queue that can hold and provide task requests
pub trait TaskQueue: Send + Sync {
    /// Add a task to the queue
    #[allow(dead_code)]
    fn push(&self, task: TaskRequest);

    /// Remove and return the next task from the queue
    fn pop(&self) -> Option<TaskRequest>;
}

/// Simple in-memory task queue using Arc<Mutex> with proper error handling
#[derive(Clone)]
pub struct SimpleTaskQueue {
    queue: Arc<Mutex<Vec<TaskRequest>>>,
    timeout_ms: u64,
    max_retries: u32,
}

impl SimpleTaskQueue {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(Vec::new())),
            timeout_ms: 1000, // 1 second default timeout
            max_retries: 3,   // 3 retries by default
        }
    }

    #[allow(dead_code)]
    pub fn with_timeout(timeout_ms: u64) -> Self {
        Self {
            queue: Arc::new(Mutex::new(Vec::new())),
            timeout_ms,
            max_retries: 3,
        }
    }

    #[allow(dead_code)]
    pub fn with_config(timeout_ms: u64, max_retries: u32) -> Self {
        Self {
            queue: Arc::new(Mutex::new(Vec::new())),
            timeout_ms,
            max_retries,
        }
    }

    /// Try to acquire the lock with timeout and retries
    fn try_lock_with_timeout(&self) -> Result<std::sync::MutexGuard<'_, Vec<TaskRequest>>, String> {
        let start_time = Instant::now();
        let timeout_duration = Duration::from_millis(self.timeout_ms);

        for attempt in 0..self.max_retries {
            // Try to acquire the lock
            match self.queue.try_lock() {
                Ok(guard) => return Ok(guard),
                Err(_) => {
                    // Check if we've exceeded the timeout
                    if start_time.elapsed() >= timeout_duration {
                        return Err(format!(
                            "Failed to acquire lock after {}ms timeout ({} attempts)",
                            self.timeout_ms,
                            attempt + 1
                        ));
                    }

                    // Small delay before retry to avoid busy waiting
                    std::thread::sleep(Duration::from_millis(1));
                }
            }
        }

        Err(format!(
            "Failed to acquire lock after {} retries",
            self.max_retries
        ))
    }
}

impl Default for SimpleTaskQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskQueue for SimpleTaskQueue {
    fn push(&self, task: TaskRequest) {
        match self.try_lock_with_timeout() {
            Ok(mut queue) => {
                queue.push(task);
            }
            Err(e) => {
                error!("Failed to push task to queue: {}", e);
                warn!("Task dropped due to lock timeout: {:?}", task);
            }
        }
    }

    fn pop(&self) -> Option<TaskRequest> {
        match self.try_lock_with_timeout() {
            Ok(mut queue) => queue.pop(),
            Err(e) => {
                error!("Failed to pop task from queue: {}", e);
                None
            }
        }
    }
}

/// Configuration for listening creators
#[derive(Debug, Clone)]
pub struct CreatorConfig {
    pub polling_interval_ms: u64,
    pub timeout_ms: u64,
}

impl Default for CreatorConfig {
    fn default() -> Self {
        Self {
            polling_interval_ms: 100,
            timeout_ms: 5000,
        }
    }
}

/// Creator for the counter usecase without ingress.
pub struct CounterCreator {
    provider: Arc<CounterProvider>,
}

impl CounterCreator {
    pub fn new(provider: CounterProvider) -> Self {
        Self {
            provider: Arc::new(provider),
        }
    }
}

#[async_trait]
impl Creator for CounterCreator {
    type TaskData = CounterTaskData;

    async fn get_payload_and_round(&self) -> Result<(Vec<u8>, u64)> {
        let round = self.provider.get_current_round().await?;
        // Domain decision: payload is ABI-encoded round
        let payload = self.provider.encode_round(round);
        Ok((payload, round))
    }

    fn get_task_metadata(&self) -> Self::TaskData {
        CounterTaskData::default()
    }
}

/// Creator for the counter usecase that listens for external requests.
pub struct ListeningCounterCreator<Q: TaskQueue + Send + Sync + 'static> {
    provider: Arc<CounterProvider>,
    queue: Arc<Q>,
    config: CreatorConfig,
    current_task: std::sync::Mutex<Option<TaskRequest>>,
}

impl<Q: TaskQueue + Send + Sync + 'static> ListeningCounterCreator<Q> {
    pub fn new(provider: CounterProvider, queue: Q, config: CreatorConfig) -> Self {
        Self {
            provider: Arc::new(provider),
            queue: Arc::new(queue),
            config,
            current_task: std::sync::Mutex::new(None),
        }
    }

    async fn wait_for_task(&self) -> Result<TaskRequest> {
        use tokio::time::{Duration, sleep};
        let mut attempts = 0;
        let max_attempts = self.config.timeout_ms / self.config.polling_interval_ms;
        loop {
            if let Some(task) = self.queue.pop() {
                // Store the task for metadata access
                if let Ok(mut current_task) = self.current_task.lock() {
                    *current_task = Some(task.clone());
                } else {
                    error!(
                        "Failed to acquire lock on current_task mutex when storing task metadata"
                    );
                }
                return Ok(task);
            }
            attempts += 1;
            if attempts >= max_attempts {
                break;
            }
            sleep(Duration::from_millis(self.config.polling_interval_ms)).await;
        }
        Err(anyhow::anyhow!(
            "Timeout waiting for task after {}ms",
            self.config.timeout_ms
        ))
    }
}

#[async_trait]
impl<Q: TaskQueue + Send + Sync + 'static> Creator for ListeningCounterCreator<Q> {
    type TaskData = CounterTaskData;

    async fn get_payload_and_round(&self) -> Result<(Vec<u8>, u64)> {
        let _task = self.wait_for_task().await?;
        let round = self.provider.get_current_round().await?;
        let payload = self.provider.encode_round(round);
        Ok((payload, round))
    }

    fn get_task_metadata(&self) -> Self::TaskData {
        // Try to get metadata from the current task, fall back to defaults if not available
        if let Ok(current_task) = self.current_task.lock()
            && let Some(ref task) = *current_task
        {
            // Extract metadata from the task request body
            let var1 = task
                .body
                .metadata
                .get("var1")
                .cloned()
                .unwrap_or_else(|| "default_var1".to_string());
            let var2 = task
                .body
                .metadata
                .get("var2")
                .cloned()
                .unwrap_or_else(|| "default_var2".to_string());
            let var3 = task
                .body
                .metadata
                .get("var3")
                .cloned()
                .unwrap_or_else(|| "default_var3".to_string());

            return CounterTaskData { var1, var2, var3 };
        }

        // Fall back to default metadata if no task data is available
        CounterTaskData::default()
    }
}

/// This enum allows us to use concrete types at compile time while still
/// supporting different creator implementations. This enables the generic
/// orchestrator to work without runtime polymorphism.
pub enum CounterCreatorType {
    /// Basic counter creator without ingress
    Basic(CounterCreator),
    /// Listening counter creator with HTTP ingress
    Listening(ListeningCounterCreator<SimpleTaskQueue>),
}

#[async_trait]
impl Creator for CounterCreatorType {
    type TaskData = CounterTaskData;

    async fn get_payload_and_round(&self) -> Result<(Vec<u8>, u64)> {
        match self {
            CounterCreatorType::Basic(creator) => creator.get_payload_and_round().await,
            CounterCreatorType::Listening(creator) => creator.get_payload_and_round().await,
        }
    }

    fn get_task_metadata(&self) -> Self::TaskData {
        match self {
            CounterCreatorType::Basic(creator) => creator.get_task_metadata(),
            CounterCreatorType::Listening(creator) => creator.get_task_metadata(),
        }
    }
}
