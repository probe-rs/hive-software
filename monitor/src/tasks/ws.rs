//! The websocket server manager which handles all ws connections during testing
use axum::extract::ws::{Message, WebSocket};
use axum::Error as AxumError;
use rand_chacha::{
    rand_core::{RngCore, SeedableRng},
    ChaChaRng,
};
use serde::Serialize;
use tokio::sync::mpsc::Receiver as MpscReceiver;

use super::TaskRunnerMessage;

/// A ticket which is used by the client to open a websocket connection for the corresponding [`TestTask`]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub(crate) struct WsTicket(String);

impl WsTicket {
    pub fn new() -> Self {
        let mut rng = ChaChaRng::from_entropy();

        let mut random_bytes = [0; 16];
        rng.fill_bytes(&mut random_bytes);

        let ticket = base64::encode(random_bytes);

        Self(ticket)
    }
}

impl From<String> for WsTicket {
    fn from(string: String) -> Self {
        Self(string)
    }
}

pub(crate) async fn socket_handler(
    mut socket: WebSocket,
    mut receiver: MpscReceiver<TaskRunnerMessage>,
) {
    tokio::spawn(async move {
        if send_json(
            &mut socket,
            TaskRunnerMessage::Status("Waiting in task queue for execution".to_owned()),
        )
        .await
        .is_err()
        {
            return;
        }

        while let Some(msg) = receiver.recv().await {
            if let TaskRunnerMessage::Results(_) = msg {
                // Close channel once results were received so the senders can be dropped and perform a graceful shutdown of the channel
                receiver.close();
            }

            if send_json(&mut socket, msg).await.is_err() {
                // Connection closed
                break;
            }
        }

        let _ = socket.close().await;
    });
}

async fn send_json(socket: &mut WebSocket, message: TaskRunnerMessage) -> Result<(), AxumError> {
    let bytes = serde_json::to_vec(&message).expect("Failed to serialize provided type to JSON");
    socket.send(Message::Binary(bytes)).await
}
