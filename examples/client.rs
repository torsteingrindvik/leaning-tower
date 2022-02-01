use std::time::Duration;

use anyhow::Result;
use leaning_tower::{client, shared};
use tower::{buffer::Buffer, MakeService, ServiceExt};
use tracing::{error, info};

fn make_handshake_request(index: usize) -> (String, shared::HandshakeRequest) {
    let string: String = match index % 3 {
        0 => "Ding".into(),
        1 => "Bong".into(),
        2 => "Gang".into(),
        _ => unreachable!(),
    };

    let req = shared::HandshakeRequest::new(string.clone());

    (string, req)
}

fn make_main_request(delay: usize) -> shared::MainRequest {
    shared::MainRequest::new(delay / 200)
}


#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // How many times to do everything:
    //  - Spawn NUM_CLIENTS
    //      - Do handshake, this returns a service to a server on another port
    //      -
    //  - Await all clients
    const NUM_ROUNDS: usize = 1000;

    const NUM_CLIENTS: usize = 10;

    // After a separate connection is established, how many messages loops to do.
    const NUM_MESSAGES: usize = 1000;

    let handshaker = client::connect().await?;
    let handshaker = handshaker.and_then(client::establish);
    let handshaker = Buffer::new(handshaker, 32);

    for round in 0..NUM_ROUNDS {
        info!(%round, "Starting");
        let mut handles = vec![];

        for index in 0..NUM_CLIENTS {
            let client_handshaker = handshaker.clone();
            let mut client_handshaker =
                tower::timeout::Timeout::new(client_handshaker, Duration::from_secs(300));
            let client_id = (round * NUM_CLIENTS) + index + 1;

            let handle = tokio::spawn(async move {
                let (label, req) = make_handshake_request(index);
                let client_res = client_handshaker
                    .ready()
                    .await
                    .expect("Handshaker not ready")
                    .make_service(req)
                    .await;

                let mut client = match client_res {
                    Ok(client) => client,
                    Err(e) => {
                        error!("Problem creating client: {:?}", e);
                        return;
                    }
                };

                // To simulate work, we send a number
                // which will be used server side to simply
                // do an (async) sleep before responding
                for delay in 0..NUM_MESSAGES {
                    let _response = client::established_call(&mut client, make_main_request(delay))
                        .await
                        .expect("Client call should be ok");
                }
                info!(?client_id, ?label, "Done :-)");
            });

            handles.push(handle);
        }

        for handle in handles {
            handle.await?;
        }
    }
    Ok(())
}
