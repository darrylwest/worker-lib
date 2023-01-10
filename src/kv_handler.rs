/// KeyValue store handler
///
use crate::JsonString;
use anyhow::Result;
use async_channel::{Receiver, Sender};
use log::*;
use service_uptime::Uptime;

use crate::worker::{WorkerState, WorkerStatus};

#[derive(Debug, Clone)]
pub enum Command {
    Status(Sender<JsonString>), // request the worker's status
    Shutdown,
}

// the handler loop
pub async fn handler(id: String, rx: Receiver<Command>) -> Result<()> {
    let uptime = Uptime::new();
    let mut state = WorkerState::Idle;
    let mut error_count = 0;
    // now read and respond to requests
    while let Ok(cmd) = rx.recv().await {
        info!("recv cmd: {:?}", cmd);
        match cmd {
            Command::Status(tx) => {
                let status = WorkerStatus::new(
                    "ok".to_string(),
                    state.clone(),
                    uptime.to_string(),
                    error_count,
                );

                let msg = match serde_json::to_string(&status) {
                    Ok(js) => js,
                    Err(e) => {
                        format!(r#"{}"status":"json parse error: {:?}"{}"#, "{", e, "}\n")
                    }
                };

                info!("status response: {}", msg);
                if tx.send(msg).await.is_err() {
                    error_count += 1;
                    error!("error returning status to channel: {:?}", tx);
                }
            }
            Command::Shutdown => {
                state = WorkerState::Shutdown;
                info!("worker id: {}, state: {:?}", id, state);
                break;
            }
        }
    }

    rx.close();

    Ok(())
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn new() {
        assert!(true);

        /*
        async_std::task::block_on(async move {
            let worker = Worker::new().await;
            println!("worker: {:?}", worker);

            assert_eq!(worker.id().len(), 16);
            assert_eq!(worker.get_update_seconds(), 0);
            println!("{}", worker.get_uptime());

            let (send, response_channel) = async_channel::unbounded();

            let request_channel = worker.request_channel();

            let cmd = Command::Status(send);
            let ok = request_channel.send(cmd).await.is_ok();
            assert_eq!(ok, true);
            let resp = response_channel
                .recv()
                .await
                .expect("should respond to status request");
            println!("[t] status response: {}", resp);
            assert!(resp.len() > 6);

            let cmd = Command::Shutdown;
            let ok = request_channel.send(cmd).await.is_ok();
            assert_eq!(ok, true);
        });
        */
    }
}
