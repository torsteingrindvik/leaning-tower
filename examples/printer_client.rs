use std::time::Duration;

use examples_lib::printer_types;
use leaning_tower::{
    allocator_client::AllocatorClientService, error::Result, mux_client::MuxClient,
};
use tower::{Service, ServiceExt};
use tracing::info;

type PrinterService = MuxClient<printer_types::Action, printer_types::Response>;

type PrinterAllocatorService =
    AllocatorClientService<printer_types::PrinterVariant, PrinterService, printer_types::Action>;

async fn printer_call(
    service: &mut PrinterService,
    request: printer_types::Action,
) -> Result<printer_types::Response> {
    Ok(service.ready().await?.call(request).await?)
}

async fn allocator_call(
    service: &mut PrinterAllocatorService,
    request: printer_types::PrinterVariant,
) -> Result<PrinterService> {
    Ok(service.ready().await?.call(request).await?)
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
    let service = AllocatorClientService::new("0.0.0.0:1235").await?;

    let mut handles = vec![];

    for idx in 1..=50 {
        for variant in [
            printer_types::PrinterVariant::Color,
            printer_types::PrinterVariant::BlackAndWhite,
        ] {
            let mut service_clone = service.clone();
            handles.push(tokio::spawn(async move {
                let mut color_printer = allocator_call(&mut service_clone, variant).await.unwrap();
                let response = printer_call(&mut color_printer, printer_types::Action::Print)
                    .await
                    .unwrap();
                info!(?idx, ?response, "Response received");
                tokio::time::sleep(Duration::from_millis(500)).await;
            }));
        }
    }

    for handle in handles {
        handle.await?;
    }

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
