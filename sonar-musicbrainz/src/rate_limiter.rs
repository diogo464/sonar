use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct RateLimiter(Arc<Mutex<Inner>>);

#[derive(Debug)]
struct Inner {
    rps: f32,
    last_req: Option<Instant>,
}

impl RateLimiter {
    pub fn new(rps: f32) -> Self {
        Self(Arc::new(Mutex::new(Inner {
            rps,
            last_req: None,
        })))
    }

    pub async fn request(&self) {
        let mut inner = self.0.lock().await;
        if let Some(elapsed) = inner.last_req.map(|i| i.elapsed()) {
            let interval = Duration::from_secs_f32(1.0 / inner.rps);
            if elapsed < interval {
                let sleep = interval - elapsed;
                tokio::time::sleep(sleep).await;
            }
        }
        inner.last_req = Some(Instant::now());
    }
}
