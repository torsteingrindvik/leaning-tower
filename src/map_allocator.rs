use std::{
    collections::HashMap,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use anyhow::Result;
use futures_core::Future;
use tokio::sync::Semaphore;
use tower::Service;
use tracing::{info, info_span, Instrument};

use crate::{server, server_service, shared, vocal_semaphore::VocalSemaphore};

#[derive(Debug)]
pub struct MapAllocatorService {
    num_times_called: usize,
    permits: HashMap<shared::HandshakeRequest, Arc<Semaphore>>,
}

impl MapAllocatorService {
    pub fn new(resources: Vec<shared::HandshakeRequest>) -> Self {
        Self {
            num_times_called: 0,
            permits: resources
                .into_iter()
                .map(|resource| (resource, Arc::new(Semaphore::new(1))))
                .collect(),
        }
    }
}

impl Service<shared::HandshakeRequest> for MapAllocatorService {
    type Response = shared::HandshakeResponse;
    type Error = anyhow::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: shared::HandshakeRequest) -> Self::Future {
        self.num_times_called += 1;

        let res = self
            .permits
            .get(&req)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No such resource: `{}`", req));

        let id = self.num_times_called;
        let label = format!("#{id}-{req}");
        let label_move = format!("#{id}-{req}");

        Box::pin(
            async move {
                match res {
                    Ok(semaphore) => {
                        let mut sem = VocalSemaphore::new(semaphore, label_move.clone());
                        sem.acquire().instrument(info_span!("acquire")).await?;

                        info!(%label_move, "Making new transport for acquired");
                        server::spawn_new_transport(server_service::MainService::new(id, sem))
                            .instrument(info_span!("spawn-transport"))
                            .await
                    }
                    Err(e) => Err(e),
                }
            }
            .instrument(info_span!("handshake-fut", %label)),
        )
    }
}
