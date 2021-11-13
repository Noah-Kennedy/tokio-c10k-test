use crate::CollectedData;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{Barrier, Notify};

#[derive(Clone)]
pub struct Client {
    pub data: CollectedData,
    pub barrier: Arc<Barrier>,
    pub stop_signal: Arc<Notify>,
    pub write_data: Arc<[u8]>,
}

impl Client {
    pub async fn run(self) {
        let mut notified_fut = Box::pin(self.stop_signal.notified());

        let mut local_data = Vec::new();

        let mut read_buf = vec![0; self.write_data.len()];

        self.barrier.wait().await;

        loop {
            let start = tokio::time::Instant::now();

            let io_fut = async {
                let mut connection = TcpStream::connect(c10k_common::INET_ADDR).await.unwrap();

                connection.write_all(&self.write_data).await.unwrap();

                connection.read_exact(&mut read_buf).await.unwrap();
            };

            tokio::select! {
                biased;

                _ = io_fut => {
                    let delta = start.elapsed().as_millis() as u64;
                    local_data.push(delta);
                }
                _ = &mut notified_fut => {
                    break;
                }
            }
        }

        self.cleanup(&mut local_data).await;
    }

    async fn cleanup(&self, data: &mut Vec<u64>) {
        self.data.append_data(data);
        self.barrier.wait().await;
    }
}
