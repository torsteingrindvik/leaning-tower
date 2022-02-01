use async_bincode::{AsyncBincodeStream, AsyncDestination};
use tokio::net::TcpStream;
use tokio_tower::pipeline;

pub(crate) type TokioTowerClient<Request, Response> = pipeline::Client<
    AsyncBincodeStream<TcpStream, Response, Request, AsyncDestination>,
    anyhow::Error,
    Request,
>;

pub type Port = u16;

pub type HandshakeRequest = String;
pub type HandshakeResponse = Port;


pub type MainRequest = usize;
pub type MainResponse = String;

pub const SERVER_BIND_ADDR: &str = "127.0.0.1:1234";
