use tokio::net::TcpSocket;

#[cfg(not(target_env = "msvc"))]
use jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

const NUM_THREADS: usize = 4;

fn main() {
    for _ in 0..(NUM_THREADS - 1) {
        std::thread::spawn(start_loop);
    }

    start_loop();
}

fn start_loop() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    runtime.block_on(entry_point());
}

async fn entry_point() {
    let sock = TcpSocket::new_v4().unwrap();

    sock.set_reuseport(true).unwrap();

    let lookup = tokio::net::lookup_host(c10k_common::INET_ADDR)
        .await
        .unwrap()
        .next()
        .unwrap();

    sock.bind(lookup).unwrap();

    let listener = sock.listen(10_000).unwrap();

    loop {
        let (mut stream, _addr) = listener.accept().await.unwrap();

        tokio::task::yield_now().await;

        let task = async move {
            let (mut rx, mut tx) = stream.split();
            let _ = tokio::io::copy(&mut rx, &mut tx).await;
        };

        tokio::spawn(task);
    }
}
