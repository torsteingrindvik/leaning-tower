use async_bincode::AsyncBincodeStream;
use tokio::net::TcpStream;
use tokio_tower::multiplex;
use tower::buffer::Buffer;
use tracing::{error, info};

use crate::{map_allocator::MapAllocatorService, shared};

pub async fn handle(
    stream: TcpStream,
    service: Buffer<MapAllocatorService, shared::HandshakeRequest>,
) {
    info!("Peer accepted");

    let transport = AsyncBincodeStream::from(stream).for_async();
    let server = multiplex::server::Server::new(transport, service);

    match server.await {
        Ok(()) => info!("Peer handler stopped"),
        Err(e) => error!("Peer handler stopped with an issue: {:?}", e),
    };
}
