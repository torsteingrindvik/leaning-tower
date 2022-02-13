use std::{
    error::Error,
    fmt::Display,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use anyhow::Result;
use async_bincode::{AsyncBincodeStream, AsyncDestination};
use futures::Future;
use serde::{de::DeserializeOwned, Serialize};
use tokio::{
    net::{TcpListener, TcpStream},
    task::JoinHandle,
};
use tokio_tower::multiplex;
use tower::{buffer::Buffer, Service};
use tracing::{error, info};

use crate::tagged;

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
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
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

pub struct MuxServer<S, Req>
where
    S: Service<Req>,
    S::Response: Serialize,
    S::Future: Send + 'static,
    Req: Send + Clone + DeserializeOwned + 'static,
{
    #[allow(clippy::type_complexity)]
    server: multiplex::Server<
        AsyncBincodeStream<
            TcpStream,
            tagged::Request<Req>,
            tagged::Response<<S as Service<Req>>::Response>,
            AsyncDestination,
        >,
        Detagger<S>,
    >,
}

impl<S, Req> MuxServer<S, Req>
where
    S: Service<Req> + Send + 'static,
    S::Response: Serialize + Send,
    S::Future: Send + 'static,
    S::Error: Send + Sync + Into<tower::BoxError> + std::fmt::Debug,
    Req: Clone + Send + DeserializeOwned + 'static,
{
    pub async fn once(bind: &str, service: S) -> Result<JoinHandle<()>> {
        let rx = TcpListener::bind(bind).await?;

        let handle = tokio::spawn(async move {
            // TODO: Timeout fut
            // This ensure that if the client left, we won't hold onto the
            // semaphore for more than this amount of seconds.
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
            info!("Alrighty");

            let rx = AsyncBincodeStream::from(rx).for_async();

            let server = multiplex::Server::new(rx, Detagger::new(service));
            match server.await {
                Ok(_) => info!("Done"),
                Err(e) => error!(?e, "Erry"),
            }
        });

        Ok(handle)
    }

    pub async fn run(bind: &str, service: S) -> Result<JoinHandle<()>> {
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
                        error!(?e, "Error :(");
                        return;
                    }
                };
                let rx = AsyncBincodeStream::from(rx).for_async();

                let server = multiplex::Server::new(rx, Detagger::new(service_for_iteration));
                match server.await {
                    Ok(_) => info!("Done"),
                    Err(e) => error!(?e, "Erry"),
                }
            }
        });

        Ok(handle)
    }
}
