use std::time::SystemTime;

use opensubsonic::service::OpenSubsonicServer;

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
    opensubsonic::service::serve("0.0.0.0:3000".parse().unwrap(), TestServer)
        .await
        .unwrap();
}
