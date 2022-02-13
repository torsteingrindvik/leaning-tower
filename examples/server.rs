use std::{
    fmt::Display,
    pin::Pin,
    task::{Context, Poll},
};

// use anyhow::Result;
use futures::Future;
use leaning_tower::{resource_filter::Describable, server_runner, shared};
use tower::Service;

#[derive(Debug)]
struct Bleh {
    id: usize,
}

impl Describable<String, usize> for Bleh {
    fn describe(&self) -> String {
        format!("i-am:{}", self.id)
    }

    fn matches(description: &String, request: &usize) -> bool {
        let (_, id) = description.split_once(':').unwrap();

        id == request.to_string()
    }
}

#[derive(Debug)]
enum MyError {
    Hi,
}

impl Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for MyError {}

impl Service<String> for Bleh {
    type Response = usize;
    type Error = MyError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Ok(()).into()
    }

    fn call(&mut self, req: String) -> Self::Future {
        let id = self.id;
        Box::pin(async move { Ok(id) })
    }
}

#[tokio::main]
async fn main() -> Result<(), String> {
    tracing_subscriber::fmt::init();

    let services = vec![Bleh { id: 0 }, Bleh { id: 1 }];

    server_runner::run(shared::SERVER_BIND_ADDR, services).await
}
