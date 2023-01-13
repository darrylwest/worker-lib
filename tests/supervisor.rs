/// integration tests to ensure workers are created and respond to commands
///
use worker_lib::cache::supervisor::Supervisor;
use worker_lib::worker::{WorkerState, OK /* DOWN */};

#[test]
fn single_worker() {
    async_std::task::block_on(async move {
        let supervisor = Supervisor::new(1)
            .await
            .expect("should create the supervisor");

        assert_eq!(supervisor.pool_size, 1);
        assert_eq!(supervisor.workers.len(), 1);

        // now get the status, should be ok
        let status = supervisor.status().await;
        println!("{:?}", status);
        for sts in status.iter() {
            assert_eq!(sts.status, OK);
            assert_eq!(sts.state, WorkerState::Idle);
        }

        // get the count and keyx, should be zero

        // set a value

        // return the set value

        // set a range of data

        // return one or more

        // return all keys

        // get total count, remove one, verify count = count - 1

        // get keys and iterate to pull values

        // check status

        // shut down
    });
}
