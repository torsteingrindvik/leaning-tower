use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Copy)]
pub enum DataDiscarderVariant {
    Fast,
    Medium,
    Slow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    /// In order to stress the transport a bit, carry a payload of some size.
    pub payload: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    /// The number of bytes in the request's payload.
    pub payload_size: usize,
}
