use std::{fmt, time::Duration};

use anyhow::Result;
use async_bincode::AsyncBincodeStream;
use serde::Serialize;
use tokio::net::TcpListener;
use tokio_tower::multiplex;
use tower::Service;
use tracing::{debug, error, info, info_span, Instrument};

use crate::shared;

async fn accept_and_await<S, T>(listener: TcpListener, service: S)
where
    S: Service<T> + Send + 'static,
    <S as Service<T>>::Response: Serialize + Send,
    <S as Service<T>>::Error: fmt::Debug + Send,
    <S as Service<T>>::Future: Send,
    T: for<'a> serde::Deserialize<'a> + Send,
{
    let timeout_fut = tokio::time::timeout(Duration::from_secs(5), listener.accept());

    info!("Starting to wait for peer");
    let inner_result = match timeout_fut.await {
        Ok(res) => res,
        Err(e) => {
            error!("Could not accept in time: {:?}", e);
            return;
        }
    };

    let (stream, peer) = match inner_result {
        Ok(v) => v,
        Err(e) => {
            error!("Could not accept on socket: {:?}", e);
            return;
        }
    };

    info!(?peer, "Peer accepted");

    let transport = AsyncBincodeStream::from(stream).for_async();
    let server = multiplex::server::Server::new(transport, service);

    match server.await {
        Ok(()) => debug!(?peer, "Server stopped"),
        Err(e) => error!("Server stopped with an issue: {:?}", e),
    }
}

pub(crate) async fn spawn_new_transport<S, T>(request: shared::HandshakeRequest, service: S) -> Result<shared::HandshakeResponse>
where
    S: Service<T> + Send + 'static,
    <S as Service<T>>::Response: Serialize + Send,
    <S as Service<T>>::Error: fmt::Debug + Send,
    <S as Service<T>>::Future: Send,
    T: for<'a> serde::Deserialize<'a> + Send,
    T: 'static,
{
    let tcp = TcpListener::bind("127.0.0.1:0").await?;
    let addr = tcp.local_addr()?;

    let port = addr.port();

    // TODO: What's the simplest way to just store away handles and check them for panics?
    tokio::spawn(accept_and_await(tcp, service).instrument(info_span!("peer-mux-handle", port)));

    let response = shared::HandshakeResponse::new(request, port);

    Ok(response)
}
