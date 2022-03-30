use examples_lib::{printer_service::Printer, printer_types::PrinterVariant};
use leaning_tower::{allocator::AllocatorService, error::Result, mux_server};
use tracing::{info, Level};

// Serve a single printer with colors at this endpoint.
async fn serve_one_resource() -> Result<()> {
    // The user's service.
    let service = Printer::new(PrinterVariant::Color);

    let handle = mux_server::run("0.0.0.0:1234", service).await?;
    info!("Letting the one resource printer service run forever");

    let _ = handle.await?;
    info!("Done serving one resource");

    Ok(())
}

// If we have many services at some endpoint,
// we may wrap it in an allocator service.
// This service handles first-come-first-serve allocation
// of services matching the description clients send.
//
// The server will reply to clients with a port.
// When a client receives this port number, they should then
// connect to that port to resume normal usage of that service.
async fn serve_many_resources() -> Result<()> {
    let services = vec![
        Printer::new(PrinterVariant::Color),
        Printer::new(PrinterVariant::BlackAndWhite),
        Printer::new(PrinterVariant::Color),
        Printer::new(PrinterVariant::Color),
        Printer::new(PrinterVariant::Color),
    ];

    let service = AllocatorService::new(services);

    let handle = mux_server::run("0.0.0.0:1235", service).await?;
    info!("Letting the many resources printers allocator run forever");

    let _ = handle.await?;
    info!("Done serving many resource");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .pretty()
        .with_max_level(Level::DEBUG)
        .init();
    info!("Running server");

    let (single_result, multi_result) = tokio::join!(serve_one_resource(), serve_many_resources());
    let _ = single_result?;
    let _ = multi_result?;

    Ok(())
}
