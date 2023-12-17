use std::{net::SocketAddr, time::SystemTime};

use opensubsonic::service::OpenSubsonicServer;
use tokio::net::TcpListener;

struct TestServer;

#[async_trait::async_trait]
impl OpenSubsonicServer for TestServer {}

fn now_milliseconds() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    axum::serve(
        TcpListener::bind("0.0.0.0:3000".parse::<SocketAddr>().unwrap())
            .await
            .unwrap(),
        opensubsonic::service::router(TestServer).await.unwrap(),
    )
    .await
    .unwrap();
}
