use super::worker::Worker;

// add generics to this based on the WorkerTrait
#[derive(Debug, Default)]
pub struct Supervisor {
    pub workers: Vec<Worker>,
}

impl Supervisor {
    pub fn new() -> Supervisor {
        Supervisor { workers: vec![] }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let pool_size = 0usize;
        let supervisor = Supervisor::new();
        assert_eq!(supervisor.workers.len(), pool_size);
    }
}
