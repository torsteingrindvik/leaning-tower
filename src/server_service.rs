use anyhow::Result;
use futures_core::Future;
use std::{
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};
use tower::Service;
use tracing::info;

use crate::{shared, vocal_semaphore::VocalSemaphore};

#[derive(Debug)]
pub(crate) struct MainService {
    id: usize,
    num_times_called: usize,
    _permit: VocalSemaphore,
}

impl Drop for MainService {
    fn drop(&mut self) {
        info!(self.id, "Dropping a permit");
    }
}

impl MainService {
    pub(crate) fn new(id: usize, permit: VocalSemaphore) -> Self {
        Self {
            id,
            num_times_called: 0,
            _permit: permit,
        }
    }
}

impl Service<shared::MainRequest> for MainService {
    type Response = shared::MainResponse;
    type Error = anyhow::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Ok(()).into()
    }

    fn call(&mut self, req: shared::MainRequest) -> Self::Future {
        self.num_times_called += 1;

        let times = self.num_times_called;
        // let wait_ms = rand::thread_rng().gen_range(500..=500);

        // Simulate some amount of work
        // let wait_ms = Duration::from_millis(10);
        let id = self.id;

        let delay = req.inner;
        let response = shared::MainResponse::new(
            req.clone(),
            format!(
                "(String) You said `{:?}`, I been called {} times (My id is: {})",
                req, times, id
            ),
        );

        Box::pin(async move {
            tokio::time::sleep(Duration::from_millis(delay as u64)).await;
            Ok(response)
        })
    }
}
