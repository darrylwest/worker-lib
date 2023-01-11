/// worker support structs
///
use serde::{Deserialize, Serialize};

pub type JsonString = String;

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
