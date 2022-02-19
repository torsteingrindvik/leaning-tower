use anyhow::Result;
use futures::Future;
use leaning_tower::resource_filter::Describable;
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use tower::Service;

use crate::printer_types::{Action, PrinterVariant, Response};

#[derive(Debug)]
pub struct Printer {
    pub variant: PrinterVariant,
}

impl Printer {
    pub fn new(variant: PrinterVariant) -> Self {
        Self { variant }
    }
}

impl Describable<PrinterVariant> for Printer {
    fn describe(&self) -> PrinterVariant {
        self.variant
    }
}

impl Service<Action> for Printer {
    type Response = Response;
    type Error = tower::BoxError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Ok(()).into()
    }

    fn call(&mut self, _request: Action) -> Self::Future {
        let response = match self.variant {
            PrinterVariant::Color => Response::Print("Printing in color!".to_string()),
            PrinterVariant::BlackAndWhite => Response::Print("No colors!".to_string()),
        };

        Box::pin(async move { Ok(response) })
    }
}
