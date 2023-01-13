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
        assert_eq!(status.len(), 1);
        for sts in status.iter() {
            assert_eq!(sts.worker_id.len(), 16);
            assert_eq!(sts.status, OK);
            assert_eq!(sts.state, WorkerState::Idle);
            assert!(sts.uptime.starts_with("0 days, 00:00"));
            assert_eq!(sts.error_count, 0);
        }

        // get the count and keyx, should be zero
        assert_eq!(supervisor.len().await, 0);

        // set a value

        // return the set value

        // set a range of data

        // return one or more

        // return all keys

        // get total count, remove one, verify count = count - 1

        // get keys and iterate to pull values

        // check status

        // shut down
        assert!(supervisor.shutdown().await.is_ok());
    });
}

#[test]
fn worker_pool() {
    assert!(true);

    // create a small worker pool (4..8)

    // loop to set about 50 values to ensure all workers are invoked

    // read back the list of keys and ensure that all are in the list (count == count)

    // read each value

    // remove one or more and verify

    // shutdown
}
