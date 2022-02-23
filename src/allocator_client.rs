use anyhow::Result;
use std::{
    fmt::Debug,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use futures_core::Future;
use serde::{de::DeserializeOwned, Serialize};
use tower::{buffer::Buffer, Service, ServiceExt};
use tracing::{info_span, Instrument};

use crate::mux_client::MuxClient;

pub struct AllocatorClientService<D, S, Req>
where
    D: Clone + PartialEq + Serialize + Send + 'static,
{
    allocator: Buffer<MuxClient<D, u16>, D>,
    service: PhantomData<S>,
    request: PhantomData<Req>,
}

impl<D, S, Req> Clone for AllocatorClientService<D, S, Req>
where
    D: Clone + PartialEq + Serialize + Send + 'static,
{
    fn clone(&self) -> Self {
        Self {
            allocator: self.allocator.clone(),
            service: self.service,
            request: self.request,
        }
    }
}

impl<D, S, Req> AllocatorClientService<D, S, Req>
where
    D: Clone + PartialEq + Serialize + Send + 'static,
{
    pub async fn new(addr: &str) -> Result<Self> {
        Ok(Self {
            allocator: Buffer::new(MuxClient::new(addr).await?, 256),
            service: Default::default(),
            request: Default::default(),
        })
    }
}

impl<D, S, Req> Service<D> for AllocatorClientService<D, S, Req>
where
    D: Debug + PartialEq + Send + Clone + Sync + 'static + Serialize,
    S: Service<Req>,
    S::Response: DeserializeOwned + Send + 'static,
    Req: Serialize + Send + Clone + 'static,
{
    type Response = MuxClient<Req, S::Response>;
    type Error = anyhow::Error;

    #[allow(clippy::type_complexity)]
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: D) -> Self::Future {
        let mut allocator_handle = self.allocator.clone();

        Box::pin(
            async move {
                let port = allocator_handle
                    .ready()
                    .await
                    .map_err(|e| anyhow::anyhow!(e))?
                    .call(request)
                    .await
                    .map_err(|e| anyhow::anyhow!(e))?;

                let client = MuxClient::new(&format!("0.0.0.0:{port}")).await?;
                Ok(client)
            }
            .instrument(info_span!("allocator-client-fut")),
        )
    }
}
