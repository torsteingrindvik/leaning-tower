use anyhow::Result;
use async_bincode::AsyncBincodeStream;
use tokio::net::TcpStream;
use tokio_tower::pipeline;
use tower::{Service, ServiceExt};

use crate::shared;

type HandshakeClient =
    shared::TokioTowerClient<shared::HandshakeRequest, shared::HandshakeResponse>;
pub type EstablishedClient = shared::TokioTowerClient<shared::MainRequest, shared::MainResponse>;

pub async fn establish(port: shared::Port) -> Result<EstablishedClient> {
    let tx = TcpStream::connect(format!("127.0.0.1:{}", port)).await?;
    let tx = AsyncBincodeStream::from(tx).for_async();
    let client = pipeline::Client::new(tx);

    Ok(client)
}

pub async fn connect() -> Result<HandshakeClient> {
    let tx = TcpStream::connect(shared::SERVER_BIND_ADDR).await?;
    let tx = AsyncBincodeStream::from(tx).for_async();

    Ok(pipeline::Client::new(tx))
}

pub async fn established_call(
    client: &mut EstablishedClient,
    request: shared::MainRequest,
) -> Result<shared::MainResponse> {
    Ok(client
        .ready()
        .await
        .map_err(|e| anyhow::anyhow!(e))?
        .call(request)
        .await
        .map_err(|e| anyhow::anyhow!(e))?)
}
