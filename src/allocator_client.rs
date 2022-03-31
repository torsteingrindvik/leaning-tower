use std::{
    fmt::Debug,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use futures_core::Future;
use serde::{de::DeserializeOwned, Serialize};
use tower::{buffer::Buffer, BoxError, Service, ServiceExt};
use tracing::{debug, info_span, warn, Instrument};

use crate::error::Result;
use crate::mux_client::MuxClient;

pub struct AllocatorClientService<D, S, Req>
where
    D: Clone + PartialEq + Serialize + Send + 'static,
{
    allocator: Buffer<MuxClient<D, u16>, D>,
    label: Option<String>,
    service: PhantomData<S>,
    request: PhantomData<Req>,
}

impl<D, S, Req> Drop for AllocatorClientService<D, S, Req>
where
    D: Clone + PartialEq + Serialize + Send + 'static,
{
    fn drop(&mut self) {
        debug!("Dropping allocator client: {:?}", self.label)
    }
}

impl<D, S, Req> Debug for AllocatorClientService<D, S, Req>
where
    D: Clone + PartialEq + Serialize + Send + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AllocatorClientService")
            // Too much bounds juggling for now
            .field("allocator", &"{no debug impl}")
            .field("service", &self.service)
            .field("request", &self.request)
            .finish()
    }
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
            label: self.label.clone().map(|label| format!("{label}-clone")),
        }
    }
}

impl<D, S, Req> AllocatorClientService<D, S, Req>
where
    D: Clone + PartialEq + Serialize + Send + 'static,
{
    pub(crate) async fn new_impl(addr: &str, label: Option<String>) -> Result<Self> {
        Ok(Self {
            allocator: Buffer::new(MuxClient::new_impl(addr, label.clone()).await?, 1),
            service: Default::default(),
            request: Default::default(),
            label,
        })
    }

    pub async fn new(addr: &str) -> Result<Self> {
        Self::new_impl(addr, None).await
    }

    pub async fn new_labelled(addr: &str, label: &str) -> Result<Self> {
        Self::new_impl(addr, Some(label.to_string())).await
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
    type Error = BoxError;

    #[allow(clippy::type_complexity)]
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<()>> {
        debug!("Polling ready");
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: D) -> Self::Future {
        debug!("Calling");
        let mut allocator_handle = self.allocator.clone();
        let label = self.label.clone();

        Box::pin(
            async move {
                debug!("Attempting allocation of resource");
                let port = {
                    // allocator_handle.ready().await?.call(request).await?
                    let ready = match allocator_handle.ready().await {
                        Ok(ready) => ready,
                        Err(e) => {
                            warn!("Was not ready: {e:?}");
                            return Err(e);
                        },
                    };

                    let response = match ready.call(request).await {
                        Ok(port) => port,
                        Err(e) => {
                            warn!("Did not get allocated resource on port: {e:?}");
                            return Err(e);
                        },
                    };

                    response
                };
                debug!("Got resource allocated ready on port {port}. Setting up a client on this port.");

                let label = label.clone().map(|label| format!("{label:?}-{port}"));
                let client = MuxClient::new_impl(&format!("0.0.0.0:{port}"), label).await?;

                debug!("Client allocated, returning");
                Ok(client)
            }
            .instrument(info_span!("allocator-client-fut")),
        )
    }
}
