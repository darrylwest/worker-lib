use anyhow::Result;
use async_channel::bounded;
use async_channel::Sender;
use domain_keys::keys::RouteKey;
use log::*;
// use serde::{Deserialize, Serialize};
use async_channel::Receiver;
use hashbrown::HashMap;
use service_uptime::Uptime;

use crate::worker::{JsonString, WorkerState, WorkerStatus, OK};

#[derive(Debug, Clone)]
pub enum Command {
    Set(String, String, Sender<Option<String>>),
    Get(String, Sender<Option<String>>),
    Remove(String, Sender<Option<String>>),
    Keys(Sender<Vec<String>>),
    Len(Sender<usize>),
    Status(Sender<JsonString>), // request the worker's status
    Shutdown,
}

// the handler loop
pub async fn handler(id: String, rx: Receiver<Command>) -> Result<()> {
    let uptime = Uptime::new();
    let mut state = WorkerState::Idle;
    let mut error_count = 0;

    // should replace this with redis at some point
    let mut cache: HashMap<String, String> = HashMap::new();

    // now read and respond to requests
    while let Ok(cmd) = rx.recv().await {
        info!("recv cmd: {:?}", cmd);
        match cmd {
            Command::Set(key, value, tx) => {
                info!("k: {}, v: {}", key, value);

                if let Some(v) = cache.insert(key, value) {
                    error_count += send_optional_response(Some(v), tx).await;
                } else {
                    error_count += send_optional_response(None, tx).await;
                }
            }
            Command::Get(key, tx) => {
                info!("get key: {}", key);
                error_count += match cache.get(&key) {
                    Some(v) => send_optional_response(Some(v.to_string()), tx).await,
                    None => send_optional_response(None, tx).await,
                }
            }
            Command::Remove(key, tx) => {
                info!("remove key: {}", key);
                if let Some(v) = cache.remove(&key) {
                    error_count += send_optional_response(Some(v), tx).await;
                } else {
                    error_count += send_optional_response(None, tx).await;
                }
            }
            Command::Keys(tx) => {
                let list: Vec<String> = vec![];
                if tx.send(list).await.is_err() {
                    error_count += error_count;
                    error!("error returning keys");
                }
            }
            Command::Len(tx) => {
                let sz = cache.len();
                let _r = tx.send(sz).await;
            }
            Command::Status(tx) => {
                let status = WorkerStatus::new(
                    id.to_string(),
                    OK.to_string(),
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

    // helper functions
    async fn send_optional_response(msg: Option<String>, tx: Sender<Option<String>>) -> u16 {
        if let Err(e) = tx.send(msg).await {
            error!("error sending message: {:?}", e);
            1u16
        } else {
            0u16
        }
    }
    // helper functions
    async fn send_response(msg: String, tx: Sender<String>) -> u16 {
        if let Err(e) = tx.send(msg).await {
            error!("error sending message: {:?}", e);
            1u16
        } else {
            0u16
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

    #[test]
    fn set_get_remove() {
        async_std::task::block_on(async move {
            let worker = Worker::new().await;
            assert_eq!(worker.id.len(), 16);
            assert_eq!(worker.get_update_seconds(), 0);

            // the request
            let request_channel = worker.request_channel();

            // the response channel
            let (responder, rx) = async_channel::bounded(10);
            let msg = Command::Len(responder);
            request_channel
                .send(msg)
                .await
                .expect("IsEmpty should never fail");
            let sz = rx.recv().await.expect("receeve should not fail");
            assert_eq!(sz, 0_usize);

            // now set the first value
            let (responder, rx) = async_channel::bounded(10);
            let key = "my-key";
            let value = "my value";

            let msg = Command::Set(key.to_string(), value.to_string(), responder);
            let resp = request_channel.send(msg).await;
            assert!(resp.is_ok());

            // get the response; should be None on the first insert
            if let Ok(resp) = rx.recv().await {
                println!("set response: {:?}", resp);
                assert_eq!(resp, None);
            } else {
                panic!("status response failed");
            }

            // check the size
            let (responder, rx) = async_channel::bounded(10);
            let msg = Command::Len(responder);
            request_channel
                .send(msg)
                .await
                .expect("IsEmpty should never fail");
            let sz = rx.recv().await.expect("receeve should not fail");
            assert_eq!(sz, 1_usize);

            // get the value from key
            let (responder, rx) = async_channel::bounded(10);
            let msg = Command::Get(key.to_string(), responder);
            request_channel
                .send(msg)
                .await
                .expect("IsEmpty should never fail");

            let v = rx.recv().await.expect("receeve should not fail").unwrap();
            println!("v: {}", v);
            assert_eq!(v, value);

            // remove the value by key
            let (responder, rx) = async_channel::bounded(10);
            let msg = Command::Remove(key.to_string(), responder);
            request_channel
                .send(msg)
                .await
                .expect("IsEmpty should never fail");

            let v = rx.recv().await.expect("receeve should not fail").unwrap();
            println!("v: {}", v);
            assert_eq!(v, value);

            // the response channel
            let (responder, rx) = async_channel::bounded(10);
            let msg = Command::Len(responder);
            request_channel
                .send(msg)
                .await
                .expect("IsEmpty should never fail");
            let sz = rx.recv().await.expect("receeve should not fail");
            assert_eq!(sz, 0_usize);

            // stop/kill the worker
            assert!(request_channel.send(Command::Shutdown).await.is_ok());
        });
    }
}
