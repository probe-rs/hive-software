//! Handles all ipc communications
use tokio::sync::mpsc::Receiver;

/// Messages which are passed between the [`std::thread`] used for testing, and the tokio runtime
#[derive(Debug)]
pub(crate) enum Message {
    //Error(String),
    Message(String),
    TestResult(String),
}

/// This function is the async entrypoint of tokio. All ipc from and to the monitor application are done here
pub(crate) async fn ipc(mut receiver: Receiver<Message>) {
    while let Some(msg) = receiver.recv().await {
        println!("{:?}", msg);
    }
}
