use std::{
    pin::Pin,
    task::{Context, Poll},
};

use anyhow::Result;
use async_bincode::{AsyncBincodeStream, AsyncDestination};
use futures::Future;
use serde::{de::DeserializeOwned, Serialize};
use tokio::net::TcpStream;
use tokio_tower::multiplex::{self, MultiplexTransport};
use tower::Service;
use tracing::error;

use crate::{slab_store, tagged};

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
}

impl<Req, Resp> MuxClient<Req, Resp>
where
    Req: Serialize + Send + 'static + Clone,
    Resp: DeserializeOwned + Send + 'static,
{
    pub async fn new(addr: &str) -> anyhow::Result<Self> {
        let tx = TcpStream::connect(addr).await?;
        let tx = AsyncBincodeStream::from(tx).for_async();

        let client = multiplex::Client::with_error_handler(
            multiplex::MultiplexTransport::new(tx, slab_store::SlabStore::default()),
            |e| error!("Client error: {:?}", e),
        );

        Ok(Self { client })
    }
}

impl<Req, Resp> Service<Req> for MuxClient<Req, Resp>
where
    Req: Serialize + Send + 'static + Clone,
    Resp: DeserializeOwned + Send + 'static,
{
    type Response = Resp;
    type Error = tower::BoxError;

    #[allow(clippy::type_complexity)]
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.client.poll_ready(cx)
    }

    fn call(&mut self, request: Req) -> Self::Future {
        let future = self.client.call(tagged::Request::new(request));
        Box::pin(async move { future.await.map(|tagged_response| tagged_response.inner()) })
    }
}
