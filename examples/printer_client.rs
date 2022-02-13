use anyhow::Result;
use examples_lib::printer_types;
use leaning_tower::mux_client::MuxClient;
use tower::{Service, ServiceExt};
use tracing::info;

type PrinterService = MuxClient<printer_types::Action, printer_types::Response>;
type PrinterAllocatorService = MuxClient<printer_types::PrinterVariant, u16>;

async fn printer_call(
    service: &mut PrinterService,
    request: printer_types::Action,
) -> printer_types::Response {
    service.ready().await.unwrap().call(request).await.unwrap()
}

async fn allocator_call(
    service: &mut PrinterAllocatorService,
    request: printer_types::PrinterVariant,
) -> u16 {
    service.ready().await.unwrap().call(request).await.unwrap()
}

async fn use_single_resource_server() -> Result<()> {
    let mut service: PrinterService = MuxClient::new("0.0.0.0:1234").await?;

    let response = printer_call(&mut service, printer_types::Action::Print).await;
    info!(?response, "Response 1 received");

    let response = printer_call(&mut service, printer_types::Action::Print).await;
    info!(?response, "Response 2 received");

    let response = printer_call(&mut service, printer_types::Action::Print).await;
    info!(?response, "Response 3 received");

    Ok(())
}

async fn use_many_resources_server() -> Result<()> {
    let mut service: PrinterAllocatorService = MuxClient::new("0.0.0.0:1235").await?;

    let response = allocator_call(&mut service, printer_types::PrinterVariant::Color).await;
    info!(?response, "Response 1 received");

    let response = allocator_call(&mut service, printer_types::PrinterVariant::Color).await;
    info!(?response, "Response 2 received");

    let response = allocator_call(&mut service, printer_types::PrinterVariant::BlackAndWhite).await;
    info!(?response, "Response 3 received");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("Running client");

    use_single_resource_server().await?;

    use_many_resources_server().await?;

    Ok(())
}
