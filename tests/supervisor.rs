use domain_keys::keys::RouteKey;
use serde::{Deserialize, Serialize};
/// integration tests to ensure workers are created and respond to commands
///
use worker_lib::cache::supervisor::Supervisor;
use worker_lib::worker::{WorkerState, OK /* DOWN */};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TestStruct {
    pub id: String,
    pub name: String,
    pub age: u8,
}

pub fn random_word() -> String {
    let mut word = String::new();
    word.push(fastrand::uppercase());
    for _ in 0..10 {
        word.push(fastrand::lowercase());
    }

    word
}
impl TestStruct {
    pub fn new() -> TestStruct {
        TestStruct {
            id: RouteKey::create(),
            name: random_word(),
            age: fastrand::u8(21..95),
        }
    }
}

#[test]
fn worker_pool() {
    async_std::task::block_on(async move {
        let pool_size = 32;
        let supervisor = Supervisor::new(pool_size)
            .await
            .expect("should create the supervisor");

        assert_eq!(supervisor.pool_size, pool_size);
        assert_eq!(supervisor.workers.len(), pool_size);

        // now get the status, should be ok
        let status = supervisor.status().await;
        println!("{:?}", status);
        assert_eq!(status.len(), pool_size);

        for sts in status.iter() {
            assert_eq!(sts.worker_id.len(), 16);
            assert_eq!(sts.status, OK);
            assert_eq!(sts.state, WorkerState::Idle);
            assert!(sts.uptime.starts_with("0 days, 00:00"));
            assert_eq!(sts.error_count, 0);
        }

        // get the count and keyx, should be zero
        assert_eq!(supervisor.len().await, 0);

        // set a number of of values
        let set_count: usize = 1000;
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

        // now read them all back
        for id in ids.iter() {
            let resp = supervisor
                .get(id.to_string())
                .await
                .expect("should return an Optional<string> vaule");
            if let Some(json) = resp {
                println!("{}", json);
                let tst: TestStruct = serde_json::from_str(&json).expect("should be able to parse");
                assert_eq!(tst.id, id.to_string());
            } else {
                assert!(false, "should not bew None");
            }
        }

        // read back the list of keys and ensure that all are in the list (count == count)

        // read each value

        // remove one or more and verify

        // shut down
        assert!(supervisor.shutdown().await.is_ok());
    });
}

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
