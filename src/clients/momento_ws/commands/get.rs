use tokio::time::timeout;

use crate::{clients::{momento_ws::{MomentoWsClient, websocket::{SocketRequest, socket_request::{self, CacheRequest, cache_request}, Get, self}}, ResponseError}, Config, workload};
use super::*;

/// Retrieve a key-value pair from the cache.
pub async fn get(
    client: &MomentoWsClient,
    config: &Config,
    request: workload::client::Get,
) -> std::result::Result<(), ResponseError> {
    GET.increment();
    match timeout(
        config.client().unwrap().request_timeout(),
        client.send(SocketRequest {
            request_id: client.next_request_id() as u64,
            kind: Some(
                socket_request::Kind::Cache(
                    CacheRequest {
                        kind: Some(
                            cache_request::Kind::Get(
                                Get {
                                    cache_key: request.key.to_vec(),
                                }
                            )
                        )
                    }
                )
            ),
        }),
    )
    .await
    {
        Ok(Ok(r)) => match r.kind {
            Some(kind) => match kind {
                websocket::socket_response::Kind::Cache(response) => match response.kind {
                    Some(kind) => match kind {
                        websocket::socket_response::cache_response::Kind::Get(response) => match response.kind {
                            Some(kind) => match kind {
                                websocket::get_response::Kind::Hit(_hit) => {
                                    GET_OK.increment();
                                    RESPONSE_HIT.increment();
                                    GET_KEY_HIT.increment();
                                    Ok(())
                                }
                                websocket::get_response::Kind::Miss(_miss) => {
                                    GET_OK.increment();
                                    RESPONSE_MISS.increment();
                                    GET_KEY_MISS.increment();
                                    Ok(())
                                }
                            },
                            None => {
                                GET_EX.increment();
                                Err(ResponseError::Exception)
                            }
                        }
                        _ => {
                            GET_EX.increment();
                            Err(ResponseError::Exception)
                        }
                    }
                    None => {
                        GET_EX.increment();
                        Err(ResponseError::Exception)
                    }
                },
                websocket::socket_response::Kind::Error(e) => {
                    GET_EX.increment();
                    Err(ResponseError::Exception)
                }
            },
            None => {
                GET_EX.increment();
                Err(ResponseError::Exception)
            }
        },
        Ok(Err(e)) => {
            GET_EX.increment();
            Err(ResponseError::Exception)
        }
        Err(_) => {
            GET_TIMEOUT.increment();
            Err(ResponseError::Timeout)
        }
    }
}
