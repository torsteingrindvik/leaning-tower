use std::{
    pin::Pin,
    task::{Context, Poll},
};

use async_bincode::{AsyncBincodeStream, AsyncDestination};
use futures::Future;
use serde::{de::DeserializeOwned, Serialize};
use tokio::net::TcpStream;
use tokio_tower::multiplex::{self, MultiplexTransport};
use tower::{BoxError, Service};
use tracing::{debug, error};

use crate::{error::Result, slab_store, tagged};

/// Multiplexing client which automatically tags requests and de-tags responses.
/// Must target a multiplexing server.
#[derive(Debug)]
pub struct MuxClient<Req, Resp>
where
    Req: Serialize,
    Resp: DeserializeOwned,
{
    #[allow(clippy::type_complexity)]
    client: multiplex::Client<
        MultiplexTransport<
            AsyncBincodeStream<
                TcpStream,
                tagged::Response<Resp>,
                tagged::Request<Req>,
                AsyncDestination,
            >,
            slab_store::SlabStore,
        >,
        tower::BoxError,
        tagged::Request<Req>,
    >,
    label: Option<String>,
}

impl<Req, Resp> MuxClient<Req, Resp>
where
    Req: Serialize + Send + 'static + Clone,
    Resp: DeserializeOwned + Send + 'static,
{
    pub(crate) async fn new_impl(addr: &str, label: Option<String>) -> Result<Self> {
        let tx = TcpStream::connect(addr).await?;
        let tx = AsyncBincodeStream::from(tx).for_async();

        let client = multiplex::Client::with_error_handler(
            multiplex::MultiplexTransport::new(tx, slab_store::SlabStore::default()),
            |e| error!("Client error: {:?}", e),
        );

        Ok(Self { client, label })
    }

    pub async fn new(addr: &str) -> Result<Self> {
        Self::new_impl(addr, None).await
    }

    pub async fn new_labelled(addr: &str, label: &str) -> Result<Self> {
        Self::new_impl(addr, Some(label.to_string())).await
    }
}

impl<Req, Resp> Service<Req> for MuxClient<Req, Resp>
where
    Req: Serialize + Send + 'static + Clone,
    Resp: DeserializeOwned + Send + 'static,
{
    type Response = Resp;
    type Error = BoxError;

    #[allow(clippy::type_complexity)]
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<()>> {
        self.client.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, request: Req) -> Self::Future {
        let future = self.client.call(tagged::Request::new(request));
        Box::pin(async move { future.await.map(|tagged_response| tagged_response.inner()) })
    }
}

impl<Req, Resp> Drop for MuxClient<Req, Resp>
where
    Req: Serialize,
    Resp: DeserializeOwned,
{
    fn drop(&mut self) {
        debug!("Dropping mux client: {:?}", self.label)
    }
}
