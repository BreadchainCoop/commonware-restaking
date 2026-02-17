use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tracing::error;

use crate::provider::CounterProvider;
use commonware_avs_router::creator::{Creator, CreatorConfig, SimpleTaskQueue, TaskQueue};
use commonware_avs_router::ingress::types::TaskRequest;
use counter_common::types::CounterTaskData;

#[allow(unused_imports)]
use serde::{Deserialize, Serialize};

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
        let payload = self.provider.encode_round(round);
        Ok((payload, round))
    }

    fn get_task_metadata(&self) -> Self::TaskData {
        CounterTaskData::default()
    }
}

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
        if let Ok(current_task) = self.current_task.lock()
            && let Some(ref task) = *current_task
        {
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

        CounterTaskData::default()
    }
}

pub enum CounterCreatorType {
    Basic(CounterCreator),
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
