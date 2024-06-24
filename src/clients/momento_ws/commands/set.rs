use tokio::time::timeout;

use crate::{clients::{momento_ws::{MomentoWsClient, websocket::{SocketRequest, socket_request::{self, CacheRequest, cache_request}, Set, self}}, ResponseError}, Config, workload};
use super::*;

/// Retrieve a key-value pair from the cache.
pub async fn set(
    client: &MomentoWsClient,
    config: &Config,
    request: workload::client::Set,
) -> std::result::Result<(), ResponseError> {
    SET.increment();
    match timeout(
        config.client().unwrap().request_timeout(),
        client.send(SocketRequest {
            request_id: 0,
            kind: Some(
                socket_request::Kind::Cache(
                    CacheRequest {
                        kind: Some(
                            cache_request::Kind::Set(
                                Set {
                                    cache_key: request.key.to_vec(),
                                    cache_body: request.value,
                                    ttl_milliseconds: request.ttl.map(|d| d.as_millis() as u64).unwrap_or(60000),
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
                        websocket::socket_response::cache_response::Kind::Set(_response) => {
                            SET_STORED.increment();
                            Ok(())
                        }
                        _ => {
                            SET_EX.increment();
                            Err(ResponseError::Exception)
                        }
                    }
                    None => {
                        SET_EX.increment();
                        Err(ResponseError::Exception)
                    }
                },
                websocket::socket_response::Kind::Error(e) => {
                    SET_EX.increment();
                    Err(ResponseError::Exception)
                }
            },
            None => {
                SET_EX.increment();
                Err(ResponseError::Exception)
            }
        },
        Ok(Err(e)) => {
            SET_EX.increment();
            Err(ResponseError::Exception)
        }
        Err(_) => {
            SET_TIMEOUT.increment();
            Err(ResponseError::Timeout)
        }
    }
}
