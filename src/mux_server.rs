use std::{
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use async_bincode::AsyncBincodeStream;
use futures::Future;
use serde::{de::DeserializeOwned, Serialize};
use tokio::{net::TcpListener, task::JoinHandle};
use tokio_tower::multiplex;
use tower::{buffer::Buffer, Service};
use tracing::{debug, error, info};

use crate::{error::Result, tagged};

// TODO: Could be a layer? Probably more idiomatic.
pub struct Detagger<S> {
    inner: S,
}

impl<S> Detagger<S> {
    pub fn new(inner: S) -> Self {
        Self { inner }
    }
}

impl<Req, S> Service<tagged::Request<Req>> for Detagger<S>
where
    S: Service<Req>,
    S::Future: Send + 'static,
    Req: Send + 'static + Clone,
{
    type Response = tagged::Response<S::Response>;
    type Error = S::Error;
    #[allow(clippy::type_complexity)]
    type Future =
        Pin<Box<dyn Future<Output = std::result::Result<Self::Response, S::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<std::result::Result<(), S::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: tagged::Request<Req>) -> Self::Future {
        let detagged = request.clone_inner();
        let future = self.inner.call(detagged);

        Box::pin(async move {
            future
                .await
                .map(|response| tagged::Response::new(request, response))
        })
    }
}

/// Run a multiplexed server for a single connection.
/// The service will be available on the bind address provided.
///
/// The task will be alive as long as the connection to the bind address is kept alive.
pub async fn once<S, Req>(bind: &str, service: S) -> Result<(JoinHandle<()>, u16)>
where
    S: Service<Req> + Send + 'static,
    S::Response: Serialize + Send,
    S::Future: Send + 'static,
    S::Error: Send + Sync + Into<tower::BoxError> + std::fmt::Debug,
    Req: Clone + Send + DeserializeOwned + 'static,
{
    let rx = TcpListener::bind(bind).await?;
    let port = rx.local_addr()?.port();

    let handle = tokio::spawn(async move {
        // This ensure that if the client left, we won't hold onto the
        // semaphore for more than this amount of seconds.
        //
        // TODO: Configurable timeout.
        // Or let the caller handle timeouts?
        let timeout_fut = tokio::time::timeout(Duration::from_secs(5), rx.accept());

        let rx = match timeout_fut.await {
            Ok(Ok((rx, _))) => rx,
            Ok(Err(e)) => {
                error!("Problem setting up server: {:?}", e);
                return;
            }
            Err(e) => {
                error!("Could not accept in time: {:?}", e);
                return;
            }
        };
        info!(%port, "Client connected, setting up server");

        let rx = AsyncBincodeStream::from(rx).for_async();
        let server = multiplex::Server::new(rx, Detagger::new(service));
        match server.await {
            Ok(_) => debug!("Done"),
            Err(e) => error!(?e, "Problem in multiplexed server"),
        }
    });

    Ok((handle, port))
}

/// Run a TCP listener on the given bind address.
/// Connections will be served the given service on a multiplexed transport.
pub async fn run<S, Req>(bind: &str, service: S) -> Result<JoinHandle<()>>
where
    S: Service<Req> + Send + 'static,
    S::Response: Serialize + Send,
    S::Future: Send + 'static,
    S::Error: Send + Sync + Into<tower::BoxError> + std::fmt::Debug,
    Req: Clone + Send + DeserializeOwned + 'static,
{
    let rx = TcpListener::bind(bind).await?;
    let service = Buffer::new(service, 32);

    let handle = tokio::spawn(async move {
        loop {
            let service_for_iteration = service.clone();

            // TODO: Timeout fut
            // This ensure that if the client left, we won't hold onto the
            // semaphore for more than this amount of seconds.
            // let timeout_fut =
            //     tokio::time::timeout(Duration::from_secs(5), server);

            let (rx, _) = match rx.accept().await {
                Ok(rx) => rx,
                Err(e) => {
                    error!(?e, "Problem accepting on TCP listener");
                    return;
                }
            };
            let rx = AsyncBincodeStream::from(rx).for_async();

            // TODO: We want to spawn a task for this right?
            let server = multiplex::Server::new(rx, Detagger::new(service_for_iteration));
            match server.await {
                Ok(_) => debug!("Done"),
                Err(e) => error!(?e, "Problem in multiplexed server"),
            }
        }
    });

    Ok(handle)
}
