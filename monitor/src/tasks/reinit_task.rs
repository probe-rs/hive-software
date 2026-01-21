use tokio::sync::oneshot::{self, Receiver as OneshotReceiver, Sender as OneshotSender};

use crate::HARDWARE;

use super::{Task, TaskManagerError, scheduler::TaskScheduler};

/// A hardware reinitialization task which can be sent to a [`TaskManager`]
pub struct ReinitializationTask {
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

impl Task for ReinitializationTask {
    fn run(self, scheduler: &mut TaskScheduler) {
        let mut hardware = HARDWARE.lock().unwrap();

        scheduler.reinitialize_hardware(&mut hardware);

        self.task_complete_sender.send(Ok(())).expect("Failed to send reinit task complete to task creator. Please ensure that the oneshot channel is not dropped on the task creator for the duration of the reinitialization.")
    }
}
