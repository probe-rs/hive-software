//! Handles all tasks which can be triggered by external users. For example test and hardware reinit tasks
use std::collections::VecDeque;
use std::sync::Arc;

use axum::body::Bytes;
use axum::response::IntoResponse;
use cached::stores::TimedCache;
use cached::Cached;
use comm_types::test::{TaskRunnerMessage, TestOptions};
use hyper::StatusCode;
use thiserror::Error;
use tokio::sync::mpsc::{self, Receiver as MpscReceiver, Sender as MpscSender};
use tokio::sync::oneshot::{self, Receiver as OneshotReceiver, Sender as OneshotSender};
use tokio::sync::Mutex as AsyncMutex;

use self::ws::WsTicket;

pub mod runner;
pub mod ws;

const TASK_CACHE_LIMIT: usize = 10;
/// Duration until a cached test request is invalidated if no websocket for the corresponding [`TestTask`] has been created
pub(crate) const WS_CONNECT_TIMEOUT_SECS: u64 = 30;

/// The possible task types the testmanager can handle
pub(super) enum TaskType {
    TestTask(TestTask),
    ReinitTask(ReinitializationTask),
    Shutdown,
}

/// A test task which can be sent to a [`TestManager`]
#[derive(Debug)]
pub(crate) struct TestTask {
    status_and_result_sender: Option<MpscSender<TaskRunnerMessage>>,
    ws_ticket: Option<WsTicket>,
    pub probe_rs_project: Bytes,
    pub options: TestOptions,
}

impl TestTask {
    pub fn new(probe_rs_project: Bytes, options: TestOptions) -> Self {
        Self {
            status_and_result_sender: None,
            ws_ticket: None,
            probe_rs_project,
            options,
        }
    }

    /// Generates a random websocket ticket and appends it to the struct and returns its value
    pub fn generate_ws_ticket(&mut self) -> WsTicket {
        let ticket = WsTicket::new();

        self.ws_ticket = Some(ticket.clone());

        ticket
    }

    /// Insert the provided sender as status and result sender for the [`TaskRunner`]
    pub fn insert_status_and_result_sender(&mut self, sender: MpscSender<TaskRunnerMessage>) {
        self.status_and_result_sender = Some(sender);
    }
}

/// A hardware reinitialization task which can be sent to a [`TestManager`]
pub(crate) struct ReinitializationTask {
    pub task_complete_sender: OneshotSender<Result<(), TaskManagerError>>,
}

impl ReinitializationTask {
    pub fn new() -> (Self, OneshotReceiver<Result<(), TaskManagerError>>) {
        let (task_complete_sender, task_complete_receiver) = oneshot::channel();

        (
            Self {
                task_complete_sender,
            },
            task_complete_receiver,
        )
    }
}

#[derive(Debug, Error)]
pub(crate) enum TaskManagerError {
    #[error("The test queue is full. Please try again later.")]
    TestQueueFull,
    #[error(
        "Discarded this reinitialization task as it has been replaced by a newer reinit request"
    )]
    ReinitTaskDiscarded,
    #[error("The provided ticket is invalid or the client took too long to connect the websocket after the initial test request")]
    TestTaskTicketInvalid,
}

impl IntoResponse for TaskManagerError {
    fn into_response(self) -> axum::response::Response {
        match self {
            TaskManagerError::TestQueueFull => {
                (StatusCode::SERVICE_UNAVAILABLE, self.to_string()).into_response()
            }
            TaskManagerError::ReinitTaskDiscarded => {
                (StatusCode::CONFLICT, self.to_string()).into_response()
            }
            TaskManagerError::TestTaskTicketInvalid => {
                (StatusCode::UNAUTHORIZED, self.to_string()).into_response()
            }
        }
    }
}

/// Manages all incoming tasks
pub(crate) struct TaskManager {
    //TODO Arc is probably not needed if I can do all within this struct
    reinit_queue: Arc<AsyncMutex<Option<ReinitializationTask>>>,
    /// The initial cache which contains all valid test requests which do not yet have a websocket connection
    test_cache: Arc<AsyncMutex<TimedCache<WsTicket, TestTask>>>,
    /// Test queue which contains all test that do have a valid websocket connection and are ready for testing
    valid_test_queue: Arc<AsyncMutex<VecDeque<TestTask>>>,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            reinit_queue: Arc::new(AsyncMutex::new(None)),
            test_cache: Arc::new(AsyncMutex::new(TimedCache::with_lifespan_and_capacity(
                WS_CONNECT_TIMEOUT_SECS,
                TASK_CACHE_LIMIT,
            ))),
            valid_test_queue: Arc::new(AsyncMutex::new(VecDeque::new())),
        }
    }

    /// Attempts to register a new [`TestTask`] scheduled for execution. If successful, a [`WsTicket`] is returned, which should be sent back to the client
    /// so the client can reopen a websocket with said ticket to receive the test status and results.
    ///
    /// This function can fail in case the internal task queue has reached the [`TASK_QUEUE_LIMIT`].
    pub async fn register_test_task(
        &self,
        mut task: TestTask,
    ) -> Result<WsTicket, TaskManagerError> {
        let mut test_cache = self.test_cache.lock().await;

        if test_cache.get_store().len() >= TASK_CACHE_LIMIT {
            return Err(TaskManagerError::TestQueueFull);
        }

        let ticket = task.generate_ws_ticket();

        test_cache.cache_set(ticket.clone(), task);

        Ok(ticket)
    }

    /// Attempts to validate the provided [`WsTicket`]. If validation succeeds the corresponding [`TestTask`] is moved from the `test_cache` into the end of the `valid_test_queue` where it will be processed by the [`TestManager`].
    /// During processing the [`TestManager`] will send status messages and ultimately the test result to the Receiver which is returned by this function.
    ///
    /// This function can fail in case the client took longer than [`WS_CONNECT_TIMEOUT_SECS`] to connect the websocket after the test run request has been received.
    pub async fn validate_test_task_ticket(
        &self,
        ws_ticket: WsTicket,
    ) -> Result<MpscReceiver<TaskRunnerMessage>, TaskManagerError> {
        let mut test_cache = self.test_cache.lock().await;

        let mut test_task = test_cache
            .cache_remove(&ws_ticket)
            .ok_or(TaskManagerError::TestTaskTicketInvalid)?;

        drop(test_cache);

        let (sender, receiver) = mpsc::channel(5);

        test_task.insert_status_and_result_sender(sender);

        let mut valid_test_queue = self.valid_test_queue.lock().await;

        valid_test_queue.push_back(test_task);

        Ok(receiver)
    }

    /// Register a new reinitialization task
    ///
    /// In case a previous reinitialization task is still in queue it is discarded in favor of the newly registered one
    pub async fn register_reinit_task(&self, task: ReinitializationTask) {
        let mut reinit_queue = self.reinit_queue.lock().await;

        if reinit_queue.is_some() {
            let old_task = reinit_queue.take().unwrap();
            let _ = old_task
                .task_complete_sender
                .send(Err(TaskManagerError::ReinitTaskDiscarded));
        }

        *reinit_queue = Some(task);
    }

    /// Get the next pending test task for execution. Returns [`None`] if the queue is currently empty
    pub async fn get_next_test_task(&self) -> Option<TestTask> {
        let mut test_queue = self.valid_test_queue.lock().await;

        test_queue.pop_front()
    }

    /// Get the next pending reinitialization task for execution. Returns [`None`] if the queue is currently empty
    pub async fn get_next_reinit_task(&self) -> Option<ReinitializationTask> {
        let mut reinit_queue = self.reinit_queue.lock().await;

        reinit_queue.take()
    }
}
