/// The worker supervisor is responsible for creating, monitoring and destroying workers in it's pool.
/// It also serves as the primary API to the outside clients specific to it's domain.  For the cache
/// worker pool the
use crate::cache::worker::{Command, Worker};
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

            assert!(supervisor.shutdown().await.is_ok());
        });
    }
}
