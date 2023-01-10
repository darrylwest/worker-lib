/// general purpose worker
///
use async_channel::bounded;
use async_channel::Sender;
use domain_keys::keys::RouteKey;
use log::*;
use serde::{Deserialize, Serialize};
use service_uptime::Uptime;

use crate::kv_handler::handler;
use crate::kv_handler::Command;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub enum WorkerState {
    #[default]
    Idle,
    Busy,
    Broken,
    Shutdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerStatus {
    status: String,
    state: WorkerState,
    uptime: String,
    error_count: u16,
}

impl WorkerStatus {
    pub fn new(
        status: String,
        state: WorkerState,
        uptime: String,
        error_count: u16,
    ) -> WorkerStatus {
        WorkerStatus {
            status,
            state,
            uptime,
            error_count,
        }
    }
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
    //  use super::*;

    #[test]
    fn bounded_tests() {
        async_std::task::block_on(async move {
            let (s, r) = async_channel::bounded(2);
            assert_eq!(r.is_empty(), true);
            assert_eq!(s.send(10).await, Ok(()));
            assert_eq!(s.send(12).await, Ok(()));

            assert_eq!(r.is_full(), true);
            assert_eq!(r.recv().await, Ok(10));
            assert_eq!(r.recv().await, Ok(12));
            assert_eq!(r.is_empty(), true);

            // second test
            println!("r empty? {}", r.is_empty());
            assert_eq!(s.send(14).await, Ok(()));
            assert_eq!(s.send(16).await, Ok(()));

            // if you try sending more than the buffer allows here, the
            // process will just wait until there is room in the queue

            println!("r empty? {}", r.is_empty());
            assert_eq!(r.recv().await, Ok(14));
            assert_eq!(r.recv().await, Ok(16));

            assert_eq!(s.close(), true);
            assert_eq!(s.is_closed(), true);

            // closing the sender shuts down the receiver as well
            assert_eq!(r.is_closed(), true);

            match r.recv().await {
                Ok(_) => assert!(false, "should not work here"),
                Err(e) => {
                    println!("error: {:?}", e);
                    assert!(true);
                }
            }
        });
    }
}
