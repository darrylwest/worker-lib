/// The worker supervisor is responsible for creating, monitoring and destroying workers in it's pool.
/// It also serves as the primary API to the outside clients specific to it's domain.  For the cache
/// worker pool the
use crate::{
    cache::worker::{Command, Worker},
    worker::WorkerStatus,
};
use anyhow::Result;
use log::*;

// add generics to this based on the WorkerTrait
#[derive(Debug, Default)]
pub struct Supervisor {
    pub pool_size: usize,
    pub auto_routing: bool,
    pub workers: Vec<Worker>,
}

impl Supervisor {
    /// create and start the worker pool
    pub async fn new(pool_size: usize) -> Result<Supervisor> {
        let auto_routing = true;
        let mut workers = vec![];

        for _ in 0..pool_size {
            let worker = Worker::new().await;
            workers.push(worker);
        }

        Ok(Supervisor {
            pool_size,
            auto_routing,
            workers,
        })
    }

    /// shut the workers down
    pub async fn shutdown(&self) -> Result<()> {
        for worker in self.workers.iter() {
            info!("shut worker, id: {} down", worker.id());
            let tx = worker.request_channel();
            let r = tx.send(Command::Shutdown).await;
            info!("ok? {:?}", r);
        }

        Ok(())
    }

    // should this auto-route?  if the key is a valid route-key, and there are multiple
    // tasks, and auto-routing is not switched off, then yes.

    /// return the status of each worker; if a worker is non-responsive, send worker down response.
    pub async fn status(&self) -> Vec<WorkerStatus> {
        let mut status = vec![];
        for worker in self.workers.iter() {
            let ws = Self::worker_status(worker).await;

            status.push(ws);
        }

        status
    }

    async fn worker_status(worker: &Worker) -> WorkerStatus {
        let request_channel = worker.request_channel();
        let (responder, rx) = async_channel::bounded(10);
        let msg = Command::Status(responder);

        let resp = request_channel.send(msg).await;
        if resp.is_err() {
            return WorkerStatus::worker_down();
        }

        if let Ok(json) = rx.recv().await {
            serde_json::from_str(&json).expect("should always decode")
        } else {
            WorkerStatus::worker_down()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        async_std::task::block_on(async move {
            let pool_size: usize = 4;
            let supervisor = Supervisor::new(pool_size)
                .await
                .expect("should create the supervisor");
            assert_eq!(supervisor.workers.len(), pool_size);

            let status = supervisor.status().await;
            println!("{:?}", status);

            assert!(supervisor.shutdown().await.is_ok());
        });
    }
}
