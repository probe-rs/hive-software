//! The websocket server manager which handles all ws connections during testing
use super::TaskRunnerMessage;
use axum::extract::ws::{Message, WebSocket};
use axum::Error as AxumError;
use rand_chacha::{
    rand_core::{RngCore, SeedableRng},
    ChaChaRng,
};
use serde::Serialize;
use tokio::sync::mpsc::Receiver as MpscReceiver;

#[cfg(doc)]
use super::scheduler::TaskScheduler;
#[cfg(doc)]
use super::test_task::TestTask;
#[cfg(doc)]
use tokio::sync::mpsc;

use crate::SHUTDOWN_SIGNAL;

/// A ticket which is used by the client to open a websocket connection for the corresponding [`TestTask`]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct WsTicket(String);

impl WsTicket {
    pub fn new() -> Self {
        let mut rng = ChaChaRng::from_entropy();

        let mut random_bytes = [0; 16];
        rng.fill_bytes(&mut random_bytes);

        // As the ticket will later be used in a url query string it should be url safe
        let base64_config = base64::Config::new(base64::CharacterSet::UrlSafe, true);
        let ticket = base64::encode_config(random_bytes, base64_config);

        Self(ticket)
    }
}

impl From<String> for WsTicket {
    fn from(string: String) -> Self {
        Self(string)
    }
}

/// Handles a single websocket connection.
///
/// It only forwards the received [`TaskRunnerMessage`]'s to the websocket.
/// In case the websocket closes but no test results have been received this function closes the [`mpsc`] channel to the [`TaskScheduler`] which indicates
/// the task runner to fail the current task as the test results can never be received by the requesting user.
pub async fn socket_handler(mut socket: WebSocket, mut receiver: MpscReceiver<TaskRunnerMessage>) {
    tokio::spawn(async move {
        let mut shutdown_signal = SHUTDOWN_SIGNAL.subscribe();

        if send_json(
            &mut socket,
            TaskRunnerMessage::Status("Waiting in task queue for execution".to_owned()),
        )
        .await
        .is_err()
        {
            return;
        }

        loop {
            tokio::select! {
                msg = receiver.recv() => {
                    if let Some(msg) = msg {
                        if let TaskRunnerMessage::Results(_) = msg {
                            // Close channel once results were received so the senders can be dropped and perform a graceful shutdown of the channel
                            receiver.close();
                        }

                        if send_json(&mut socket, msg).await.is_err() {
                            // Connection closed
                            // Close the receiver to force the running task to fail once it tries to send any message to the websocket
                            receiver.close();
                            break;
                        }
                    } else {
                        break;
                    }
                }
                result = shutdown_signal.recv() => {
                    result.expect("Failed to receive global shutdown signal");
                    break;
                }
            }
        }

        let _ = socket.close().await;
    });
}

/// Send data as JSON over the websocket
async fn send_json(socket: &mut WebSocket, message: TaskRunnerMessage) -> Result<(), AxumError> {
    let bytes = serde_json::to_vec(&message).expect("Failed to serialize provided type to JSON");
    socket.send(Message::Binary(bytes)).await
}
