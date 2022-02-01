use std::pin::Pin;

use anyhow::Result;
use async_bincode::AsyncBincodeStream;
use slab::Slab;
use tokio::net::TcpStream;
use tokio_tower::multiplex::{self, MultiplexTransport, TagStore};
use tower::{Service, ServiceExt};

use crate::shared;

pub struct SlabStore(Slab<()>);

impl TagStore<shared::HandshakeRequest, shared::HandshakeResponse> for SlabStore {
    type Tag = usize;
    fn assign_tag(mut self: Pin<&mut Self>, request: &mut shared::HandshakeRequest) -> usize {
        let tag = self.0.insert(());
        request.set_tag(tag);
        tag
    }
    fn finish_tag(mut self: Pin<&mut Self>, response: &shared::HandshakeResponse) -> usize {
        let tag = response.tag();
        self.0.remove(tag);
        tag
    }
}

impl TagStore<shared::MainRequest, shared::MainResponse> for SlabStore {
    type Tag = usize;
    fn assign_tag(mut self: Pin<&mut Self>, request: &mut shared::MainRequest) -> usize {
        let tag = self.0.insert(());
        request.set_tag(tag);
        tag
    }
    fn finish_tag(mut self: Pin<&mut Self>, response: &shared::MainResponse) -> usize {
        let tag = response.tag();
        self.0.remove(tag);
        tag
    }
}

type HandshakeClient =
    shared::TokioTowerClient<shared::HandshakeRequest, shared::HandshakeResponse>;

pub type EstablishedClient = shared::TokioTowerClient<shared::MainRequest, shared::MainResponse>;

pub async fn establish(response: shared::HandshakeResponse) -> Result<EstablishedClient> {
    let tx = TcpStream::connect(format!("127.0.0.1:{}", response.response)).await?;
    let tx = AsyncBincodeStream::from(tx).for_async();

    let client = multiplex::Client::new(MultiplexTransport::new(tx, SlabStore(Slab::new())));

    Ok(client)
}

pub async fn connect() -> Result<HandshakeClient> {
    let tx = TcpStream::connect(shared::SERVER_BIND_ADDR).await?;
    let tx = AsyncBincodeStream::from(tx).for_async();

    let client = multiplex::Client::new(MultiplexTransport::new(tx, SlabStore(Slab::new())));

    Ok(client)
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
