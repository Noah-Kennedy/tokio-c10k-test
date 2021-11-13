use crate::client::Client;
use crate::statistics::ProcessedStatistics;
use rand::{thread_rng, Rng};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::{Barrier, Notify};

mod client;
mod statistics;

#[cfg(not(target_env = "msvc"))]
use jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

const CONNECTIONS: usize = 20_000;
const N_BYTES: usize = 4 * 1024 * 1024;
const TEST_DURATION: Duration = Duration::from_secs(60);

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    println!("Starting client");
    let write_data = make_random(N_BYTES);

    let client_prototype = Client {
        data: CollectedData::new(),
        barrier: Arc::new(Barrier::new(CONNECTIONS + 1)),
        stop_signal: Arc::new(Notify::new()),
        write_data,
    };

    for _ in 0..CONNECTIONS {
        tokio::spawn(client_prototype.clone().run());
    }

    println!("Kicked off clients");

    // start gates
    client_prototype.barrier.wait().await;

    println!("Running test");
    tokio::time::sleep(TEST_DURATION).await;

    println!("Finishing test");
    client_prototype.stop_signal.notify_waiters();

    // wait for term
    client_prototype.barrier.wait().await;

    let data = ProcessedStatistics::from(&client_prototype.data);

    println!("Data: {:?}", data);
}

fn make_random(n_bytes: usize) -> Arc<[u8]> {
    let mut rng = thread_rng();

    (0..n_bytes).map(|_| rng.gen()).collect()
}

#[derive(Clone, Default)]
pub struct CollectedData {
    data: Arc<Mutex<Vec<u64>>>,
}

impl CollectedData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn append_data(&self, new: &mut Vec<u64>) {
        let mut guard = self.data.lock().unwrap();

        guard.append(new)
    }
}
