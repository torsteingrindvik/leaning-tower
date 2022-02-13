use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Copy)]
pub enum PrinterVariant {
    Color,
    BlackAndWhite,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    Print,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    Print(String),
}
