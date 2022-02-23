use anyhow::Result;
use examples_lib::data_discarder_types;
use leaning_tower::{allocator_client::AllocatorClientService, mux_client::MuxClient};
use rand::Rng;
use tower::{Service, ServiceExt};
use tracing::{error, info};

type DataDiscarderService = MuxClient<data_discarder_types::Action, data_discarder_types::Response>;
type DataDiscarderAllocatorService = AllocatorClientService<
    data_discarder_types::DataDiscarderVariant,
    DataDiscarderService,
    data_discarder_types::Action,
>;

async fn discarder_call(
    service: &mut DataDiscarderService,
    request: data_discarder_types::Action,
) -> Result<data_discarder_types::Response> {
    Ok(service
        .ready()
        .await
        .map_err(|e| anyhow::anyhow!(e))?
        .call(request)
        .await
        .map_err(|e| anyhow::anyhow!(e))?)
}

async fn allocator_call(
    service: &mut DataDiscarderAllocatorService,
    request: data_discarder_types::DataDiscarderVariant,
) -> Result<DataDiscarderService> {
    Ok(service
        .ready()
        .await
        .map_err(|e| anyhow::anyhow!(e))?
        .call(request)
        .await
        .map_err(|e| anyhow::anyhow!(e))?)
}

fn random_payload() -> Vec<u8> {
    let payload_size = rand::thread_rng().gen_range(10..=500);
    let mut payload = Vec::with_capacity(payload_size);
    payload.fill(rand::thread_rng().gen());

    payload
}

async fn use_forever() -> Result<()> {
    let allocator = AllocatorClientService::new("0.0.0.0:1234").await?;

    let mut robustness_round = 0;
    loop {
        robustness_round += 1;
        info!("Starting round {robustness_round}");

        let mut handles = vec![];

        // Spawn 50 of each variant- 150 clients in total.
        for _idx in 1..=50 {
            for variant in [
                data_discarder_types::DataDiscarderVariant::Fast,
                data_discarder_types::DataDiscarderVariant::Medium,
                data_discarder_types::DataDiscarderVariant::Slow,
            ] {
                // The allocator can be cloned such that each async task
                // can own a handle to it.
                let mut allocator_handle = allocator.clone();

                handles.push(tokio::spawn(async move {
                    // Wait here until we can get a hold of the data discarder type we want.
                    let mut discarder = match allocator_call(&mut allocator_handle, variant).await {
                        Ok(discarder) => discarder,
                        Err(e) => return Err(e),
                    };

                    let mut rounds = 0;
                    let response = loop {
                        let response = discarder_call(
                            &mut discarder,
                            data_discarder_types::Action {
                                payload: random_payload(),
                            },
                        )
                        .await
                        .unwrap();

                        rounds += 1;
                        if rounds == 100 {
                            break response;
                        }
                    };

                    Ok(response)
                }));
            }
        }

        for handle in handles {
            // Don't really care about the responses.
            let _ = handle.await?;
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("Running data discarder client");

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("ctrl-c, stopping")
        }
        val = use_forever() => {
            error!("Robustness testing stopped: {:?}", val)
        }
    };

    Ok(())
}
