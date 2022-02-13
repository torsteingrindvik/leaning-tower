use async_bincode::AsyncBincodeStream;
use serde::{de::DeserializeOwned, Serialize};
use tokio::net::TcpStream;
use tokio_tower::multiplex;
use tower::{buffer::Buffer, Service};
use tracing::{error, info};

use crate::{allocator::AllocatorService, resource_filter::Describable, tagged};

pub async fn handle<S, R, HR>(
    stream: TcpStream,
    service: Buffer<AllocatorService<S, R, HR>, HR>,
) where
    S: Service<R> + Send + 'static,
    S: Describable<HR>,
    R: Send + 'static,
    S::Future: Send,
    S::Error: Send + Sync + Into<tower::BoxError>,
    AllocatorService<S, R, HR>: Service<HR>,
    HR: Serialize + DeserializeOwned + std::fmt::Debug + Send + Clone,
    <AllocatorService<S, R, HR> as Service<HR>>::Error:
        Send + Sync + std::error::Error + 'static,
    <AllocatorService<S, R, HR> as Service<HR>>::Response: Serialize + DeserializeOwned,
{
    info!("Peer accepted");

    let transport = AsyncBincodeStream::from(stream).for_async();
    let server = multiplex::server::Server::new(transport, service);

    match server.await {
        Ok(()) => info!("Peer handler stopped"),
        Err(e) => error!("Peer handler stopped with an issue: {:?}", e),
    };
}
