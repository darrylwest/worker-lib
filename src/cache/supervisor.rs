/// The worker supervisor is responsible for creating, monitoring and destroying workers in it's pool.
/// It also serves as the primary API to the outside clients specific to it's domain.  For the cache
/// worker pool the
use crate::{
    cache::worker::{Command, Worker},
    worker::{JsonString, WorkerStatus},
};
use anyhow::{anyhow, Result};
use domain_keys::keys::RouteKey;
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

    pub fn get_route(&self, key: &str) -> usize {
        if self.pool_size > 1 {
            let rcount = self.pool_size as u8;
            match RouteKey::parse_route(key, rcount) {
                Ok(x) => x as usize,
                _ => 0_usize,
            }
        } else {
            0_usize
        }
    }

    /// store the value (json blob)
    pub async fn set(&self, key: String, value: JsonString) -> Result<Option<String>> {
        let route = self.get_route(&key);
        let worker = &self.workers[route];

        let worker_id = worker.id();

        let request_channel = worker.request_channel();
        let (responder, rx) = async_channel::bounded(10);
        let msg = Command::Set(key, value, responder);

        let resp = request_channel.send(msg).await;
        if resp.is_err() {
            let msg = format!("worker id {} request channel is down", worker_id);
            error!("{}", msg);
            return Err(anyhow!(msg));
        }

        let resp = if let Some(json) = rx.recv().await? {
            info!("{}", json);
            Some(json)
        } else {
            None
        };

        Ok(resp)
    }

    /// return the status of each worker; if a worker is non-responsive, send worker down response.
    /// NOTE: *good candidate for paralell ops...*
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
            return WorkerStatus::worker_down(worker.id());
        }

        if let Ok(json) = rx.recv().await {
            serde_json::from_str(&json).expect("should always decode")
        } else {
            WorkerStatus::worker_down(worker.id())
        }
    }

    /// return the total number of entries from all workers
    /// NOTE: *good candidate for paralell ops...*
    pub async fn len(&self) -> usize {
        let mut sz = 0_usize;
        for worker in self.workers.iter() {
            sz += Self::worker_len(worker).await;
        }

        sz
    }

    async fn worker_len(worker: &Worker) -> usize {
        let request_channel = worker.request_channel();
        let (responder, rx) = async_channel::bounded(10);
        let msg = Command::Len(responder);
        request_channel
            .send(msg)
            .await
            .expect("should always return");

        let sz = rx.recv().await.expect("should always return a size");

        sz
    }
}

#[cfg(test)]
mod tests {
    use serde::Serialize;

    use super::*;
    use crate::worker::{WorkerState, OK};

    #[derive(Debug, Default, Clone, Serialize)]
    pub struct TestStruct {
        pub id: String,
        pub name: String,
        pub age: u8,
    }

    fn random_word() -> String {
        let mut word = String::new();
        word.push(fastrand::uppercase());
        for _ in 0..10 {
            word.push(fastrand::lowercase());
        }

        word
    }
    impl TestStruct {
        fn new() -> TestStruct {
            TestStruct {
                id: RouteKey::create(),
                name: random_word(),
                age: fastrand::u8(21..95),
            }
        }
    }

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
            assert_eq!(status.len(), pool_size);
            for sts in status.iter() {
                println!("{}", sts.uptime);
                assert_eq!(sts.worker_id.len(), 16);
                assert_eq!(sts.status, OK);
                assert_eq!(sts.state, WorkerState::Idle);
                assert!(sts.uptime.starts_with("0 days, 00:00"));
                assert_eq!(sts.error_count, 0);
            }

            assert_eq!(supervisor.len().await, 0);

            // set a number of of values
            let set_count: usize = 20;
            let mut ids: Vec<String> = Vec::with_capacity(set_count);
            for _n in 0..set_count {
                let tst = TestStruct::new();
                ids.push(tst.id.to_string());
                assert_eq!(tst.id.len(), 16);
                let route = supervisor.get_route(&tst.id);
                println!("{:?} {}", tst, route);
                assert!(route < pool_size);

                let json = serde_json::to_string(&tst).unwrap();
                let r = supervisor
                    .set(tst.id, json)
                    .await
                    .expect("should return ok from set");
                assert_eq!(r, None, "should be none on first set");
            }

            assert_eq!(ids.len(), set_count);
            assert_eq!(supervisor.len().await, set_count);

            assert!(supervisor.shutdown().await.is_ok());
        });
    }
}
