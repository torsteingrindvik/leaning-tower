use async_bincode::{AsyncBincodeStream, AsyncDestination};
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_tower::multiplex::{self, MultiplexTransport};

use crate::client::SlabStore;

pub(crate) type TokioTowerClient<Request, Response> = multiplex::Client<
    MultiplexTransport<
        AsyncBincodeStream<TcpStream, Response, Request, AsyncDestination>,
        SlabStore,
    >,
    anyhow::Error,
    Request,
>;

pub type Port = u16;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeTagRequest {
    pub(crate) request: String,
    tag: usize,
}

impl HandshakeTagRequest {
    pub fn new(request: String) -> Self {
        Self { request, tag: 0 }
    }

    pub fn set_tag(&mut self, tag: usize) {
        self.tag = tag;
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HandshakeTagResponse {
    pub(crate) response: Port,
    tag: usize,
}

impl HandshakeTagResponse {
    /// Get a reference to the tag response's tag.
    pub fn tag(&self) -> usize {
        self.tag
    }

    pub fn new(request: HandshakeTagRequest, response: Port) -> Self {
        Self {
            response,
            tag: request.tag,
        }
    }
}

pub type HandshakeRequest = HandshakeTagRequest;
pub type HandshakeResponse = HandshakeTagResponse;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MainTagRequest {
    pub(crate) request: usize,
    tag: usize,
}

impl MainTagRequest {
    pub fn new(request: usize) -> Self {
        Self { request, tag: 0 }
    }

    pub fn set_tag(&mut self, tag: usize) {
        self.tag = tag;
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MainTagResponse {
    response: String,
    tag: usize,
}

impl MainTagResponse {
    /// Get a reference to the tag response's tag.
    pub fn tag(&self) -> usize {
        self.tag
    }

    pub fn new(request: MainTagRequest, response: String) -> Self {
        Self {
            response,
            tag: request.tag,
        }
    }
}

pub type MainRequest = MainTagRequest;
pub type MainResponse = MainTagResponse;

pub const SERVER_BIND_ADDR: &str = "127.0.0.1:1234";
