use super::*;

mod commands;
mod client;
pub(crate) mod websocket;

pub use client::MomentoWsClient;

use commands::*;

/// Launch tasks with one channel per task as gRPC is mux-enabled.
pub fn launch_tasks(runtime: &mut Runtime, config: Config, work_receiver: Receiver<WorkItem>) {
    debug!("launching momento WS protocol tasks");

    for _ in 0..config.client().unwrap().poolsize() {
        let _guard = runtime.enter();

        // initialize the Momento cache client
        if std::env::var("MOMENTO_AUTHENTICATION").is_err() {
            eprintln!("environment variable `MOMENTO_AUTHENTICATION` is not set");
            std::process::exit(1);
        }
        let auth_token = std::env::var("MOMENTO_AUTHENTICATION")
            .expect("MOMENTO_AUTHENTICATION must be set");

        let credential_provider = ::momento::CredentialProviderBuilder::from_string(auth_token)
            .build()
            .unwrap_or_else(|e| {
                eprintln!("failed to initialize credential provider. error: {e}");
                std::process::exit(1);
            });

        let config = config.clone();
        let work_receiver = work_receiver.clone();
        runtime.spawn(async move {
            let client = Arc::new(MomentoWsClient::new(credential_provider.cache_endpoint, config.target().cache_name().expect("must have a cache name").to_string(), credential_provider.auth_token).await);

            CONNECT.increment();
            CONNECT_CURR.increment();

            for _ in 0..config.client().unwrap().concurrency() {
                tokio::spawn(task(config.clone(), client.clone(), work_receiver.clone()));
            }
        });
    }
}

async fn task(
    config: Config,
    // cache_name: String,
    client: Arc<MomentoWsClient>,
    work_receiver: Receiver<WorkItem>,
) -> Result<()> {
    let cache_name = config.target().cache_name().unwrap_or_else(|| {
        eprintln!("cache name is not specified in the `target` section");
        std::process::exit(1);
    });

    while RUNNING.load(Ordering::Relaxed) {
        let work_item = work_receiver
            .recv()
            .await
            .map_err(|_| Error::new(ErrorKind::Other, "channel closed"))?;

        REQUEST.increment();
        let start = Instant::now();
        let result = match work_item {
            WorkItem::Request { request, .. } => match request {
                /*
                 * KEY-VALUE
                 */
                ClientRequest::Get(r) => get(&client, &config, r).await,
                ClientRequest::Set(r) => set(&client, &config, r).await,

                /*
                 * UNSUPPORTED
                 */
                _ => {
                    REQUEST_UNSUPPORTED.increment();
                    continue;
                }
            },
            WorkItem::Reconnect => {
                continue;
            }
        };

        REQUEST_OK.increment();

        let stop = Instant::now();

        match result {
            Ok(_) => {
                RESPONSE_OK.increment();

                let latency = stop.duration_since(start).as_nanos() as u64;

                let _ = RESPONSE_LATENCY.increment(latency);
            }
            Err(ResponseError::Exception) => {
                RESPONSE_EX.increment();
            }
            Err(ResponseError::Timeout) => {
                RESPONSE_TIMEOUT.increment();
            }
            Err(ResponseError::Ratelimited) => {
                RESPONSE_RATELIMITED.increment();
            }
            Err(ResponseError::BackendTimeout) => {
                RESPONSE_BACKEND_TIMEOUT.increment();
            }
        }
    }

    Ok(())
}
