use anyhow::Result;
use async_channel::bounded;
use async_channel::Sender;
use domain_keys::keys::RouteKey;
use log::*;
// use serde::{Deserialize, Serialize};
use crate::JsonString;
use async_channel::Receiver;
use service_uptime::Uptime;

use crate::worker::{WorkerState, WorkerStatus};

#[derive(Debug, Clone)]
pub enum Command {
    Status(Sender<JsonString>), // request the worker's status
    Set(String, String, Sender<JsonString>),
    Get(String, Sender<JsonString>),
    Remove(String, Sender<JsonString>),
    Shutdown,
}

// the handler loop
pub async fn handler(id: String, rx: Receiver<Command>) -> Result<()> {
    let uptime = Uptime::new();
    let mut state = WorkerState::Idle;
    let mut error_count = 0;

    async fn send_response(msg: String, tx: Sender<String>) -> u16 {
        if let Err(e) = tx.send(msg).await {
            error!("error sending message: {:?}", e);
            1u16
        } else {
            0u16
        }
    }

    // now read and respond to requests
    while let Ok(cmd) = rx.recv().await {
        info!("recv cmd: {:?}", cmd);
        match cmd {
            Command::Set(key, value, tx) => {
                info!("k: {}, v: {}", key, value);
                error_count += send_response("Ok".to_string(), tx).await;
            }
            Command::Get(key, tx) => {
                info!("get key: {}", key);
                error_count += send_response("Ok".to_string(), tx).await;
            }
            Command::Remove(key, tx) => {
                info!("remove key: {}", key);
                error_count += send_response("Ok".to_string(), tx).await;
            }
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

#[derive(Debug, Clone)]
pub struct Worker {
    id: String,
    uptime: Uptime,
    request_tx: Sender<Command>,
}

//
impl Worker {
    /// create and start a new worker.
    pub async fn new() -> Worker {
        let uptime = Uptime::new();
        let id = RouteKey::create();

        // this is for the worker struct
        let wid = id.clone();

        info!("starting up worker, id: {}", id);

        let (request_tx, request_receiver) = bounded(250);

        // run the handler loop as a background task
        async_std::task::spawn(async move {
            match handler(id.clone(), request_receiver).await {
                Ok(()) => info!("worker handler exit for worker id: {}", id),
                Err(e) => error!("worker exex with error: {:?}", e),
            }
        });

        // define here, before the async loop to ensure it does not get moved
        let worker = Worker {
            id: wid,
            uptime,
            request_tx,
        };

        info!("worker created: {:?}", &worker);

        // return this worker
        worker
    }

    /// return the worker's id
    pub fn id(&self) -> String {
        self.id.to_string()
    }

    /// return the number of seconds this worker has been alive
    pub fn get_uptime(&self) -> String {
        self.uptime.to_string()
    }

    pub fn get_update_seconds(&self) -> u64 {
        self.uptime.get_uptime_seconds()
    }

    /// This is invoked by the client to enable sending command request to
    /// the worker
    pub fn request_channel(&self) -> Sender<Command> {
        self.request_tx.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        async_std::task::block_on(async move {
            let worker = Worker::new().await;
            assert_eq!(worker.id.len(), 16);
            assert_eq!(worker.get_update_seconds(), 0);

            // the request
            let request_channel = worker.request_channel();

            // the response channel
            let (responder, rx) = async_channel::bounded(10);

            let msg = Command::Status(responder);
            let resp = request_channel.send(msg).await;
            println!("{:?}", resp);
            assert!(resp.is_ok());
            if let Ok(resp) = rx.recv().await {
                println!("{}", resp);
            } else {
                panic!("status response failed");
            }

            assert!(request_channel.send(Command::Shutdown).await.is_ok());
        });
    }
}
