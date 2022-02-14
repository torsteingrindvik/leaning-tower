use anyhow::Result;
use examples_lib::printer_types;
use leaning_tower::{allocator_client::AllocatorClientService, mux_client::MuxClient};
use tower::{Service, ServiceExt};
use tracing::info;

type PrinterService = MuxClient<printer_types::Action, printer_types::Response>;

type PrinterAllocatorService =
    AllocatorClientService<printer_types::PrinterVariant, PrinterService, printer_types::Action>;

async fn printer_call(
    service: &mut PrinterService,
    request: printer_types::Action,
) -> Result<printer_types::Response> {
    Ok(service
        .ready()
        .await
        .map_err(|e| anyhow::anyhow!(e))?
        .call(request)
        .await
        .map_err(|e| anyhow::anyhow!(e))?)
}

async fn allocator_call(
    service: &mut PrinterAllocatorService,
    request: printer_types::PrinterVariant,
) -> Result<PrinterService> {
    Ok(service
        .ready()
        .await
        .map_err(|e| anyhow::anyhow!(e))?
        .call(request)
        .await
        .map_err(|e| anyhow::anyhow!(e))?)
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
    let mut service = AllocatorClientService::new("0.0.0.0:1235").await?;

    let mut printer_1 = allocator_call(&mut service, printer_types::PrinterVariant::Color).await?;
    let response = printer_call(&mut printer_1, printer_types::Action::Print).await?;
    info!(?response, "Response 1 received");
    drop(printer_1);

    let mut printer_2 =
        allocator_call(&mut service, printer_types::PrinterVariant::BlackAndWhite).await?;
    let response = printer_call(&mut printer_2, printer_types::Action::Print).await?;
    info!(?response, "Response 2 received");
    drop(printer_2);

    let mut printer_3 = allocator_call(&mut service, printer_types::PrinterVariant::Color).await?;
    let response = printer_call(&mut printer_3, printer_types::Action::Print).await?;
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
