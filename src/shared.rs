use async_bincode::{AsyncBincodeStream, AsyncDestination};
use tokio::net::TcpStream;
use tokio_tower::multiplex::{self, MultiplexTransport};

use crate::{slab_store::SlabStore, tagged};

pub(crate) type TokioTowerClient<Request, Response> = multiplex::Client<
    MultiplexTransport<
        AsyncBincodeStream<TcpStream, Response, Request, AsyncDestination>,
        SlabStore,
    >,
    anyhow::Error,
    Request,
>;

pub type Port = u16;

pub type HandshakeRequest = tagged::Request<String>;
pub type HandshakeResponse = tagged::Response<Port>;

pub type MainRequest = tagged::Request<usize>;
pub type MainResponse = tagged::Response<String>;

pub const SERVER_BIND_ADDR: &str = "127.0.0.1:1234";
