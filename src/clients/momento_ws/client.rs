use std::{pin::{Pin, pin}, task::{Context, Poll}, collections::VecDeque, sync::{Arc, atomic::AtomicUsize}};

use futures::{StreamExt, Future, Stream, Sink, SinkExt};
use prost::Message;
use ringlog::{error, debug, trace};
use tokio::{task::JoinHandle, sync::{mpsc, oneshot}};
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;

use super::websocket;


pub struct MomentoWsClient {
    handle: JoinHandle<()>,
    command_sender: mpsc::Sender<NewCommand>,
}

impl Drop for MomentoWsClient {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

impl MomentoWsClient {
    pub async fn new(url: String, cache: String, auth_token: String) -> Self {
        debug!("making connection to {url}");
        let request = hyper::Request::builder()
            .uri(format!("{}/command_socket", url.replace("https", "wss")))
            .header("authorization", auth_token)
            .header("cache", cache)
            .header("content-encoding", "proto")

            // a bunch of headers that tungstenite is dying without
            .header("Sec-WebSocket-Key", "GMs4FwNEuRqrRZXUbv3zgQ==")
            .header("Sec-WebSocket-Version", "13")
            .header("Host", url.replace("https://", ""))
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")

            .body(())
            .expect("must be able to create a valid request");

        let config = WebSocketConfig::default();

        let socket = match tokio_tungstenite::connect_async_with_config(request, Some(config), true).await {
            Ok((socket, _)) => socket,
            Err(e) => {
                panic!("must be able to connect: {e:?}")
            }
        };

        let (command_sender, command_stream) = mpsc::channel(16);
        let handle = tokio::spawn(Socket { socket, command_stream, pending_commands: Default::default(), command_id: 0 });

        Self {
            handle,
            command_sender,
        }
    }

    pub async fn send(&self, mut request: websocket::SocketRequest) -> Result<websocket::SocketResponse, Error> {
        let (response, receiver) = oneshot::channel();
        match self.command_sender.try_send(NewCommand { request, response }) {
            Ok(_) => (),
            Err(e) => {
                return match e {
                    mpsc::error::TrySendError::Full(_) => Err(Error::Full),
                    mpsc::error::TrySendError::Closed(_) => Err(Error::Closed),
                }
            },
        }

        match receiver.await {
            Ok(reply) => {
                trace!("got: {reply:?}");
                reply
            }
            Err(e) => {
                trace!("got: {e:?}");
                Err(Error::Cancelled)
            }
        }
    }
}

struct Socket {
    socket: tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    command_stream: mpsc::Receiver<NewCommand>,
    pending_commands: VecDeque<PendingCommand>,
    command_id: usize,
}

#[derive(Debug)]
struct NewCommand {
    request: websocket::SocketRequest,
    response: oneshot::Sender<Result<websocket::SocketResponse, Error>>,
}

struct PendingCommand {
    id: usize,
    response: oneshot::Sender<Result<websocket::SocketResponse, Error>>,
}

#[derive(thiserror::Error, std::fmt::Debug, Clone)]
pub enum Error {
    #[error("socket error")]
    Socket(#[from] Arc<tokio_tungstenite::tungstenite::Error>),
    #[error("decode format error")]
    Decode(#[from] prost::DecodeError),
    #[error("could not send: Closed")]
    Closed,
    #[error("could not send: Full")]
    Full,
    #[error("request was cancelled")]
    Cancelled,
}

impl Future for Socket {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(early_out) = self.as_mut().drain_replies(context) {
            return early_out;
        }
        // we are now pending wake for replies on the socket

        loop {
            match self.as_mut().output_has_room(context) {
                Ok(has_room) => {
                    if !has_room {
                        break;
                    }
                    // fall through
                }
                Err(early_out) => {
                    return early_out;
                }
            }

            match self.command_stream.poll_recv(context) {
                Poll::Ready(next) => {
                    match next {
                        Some(next) => {
                            let NewCommand { mut request, response } = next;
                            let id = self.command_id;
                            self.command_id += 1;

                            request.request_id = id as u64;

                            let message = tokio_tungstenite::tungstenite::Message::Binary(request.encode_to_vec());
                            self.pending_commands.push_back(PendingCommand { id, response });

                            match pin!(&mut self.socket).start_send(message) {
                                Ok(_) => {
                                    continue; // Check for room and send more
                                }
                                Err(e) => {
                                    self.fail(Error::Socket(e.into()));
                                    return Poll::Ready(());
                                }
                            }
                        }
                        None => {
                            return Poll::Ready(());
                        }
                    }
                }
                Poll::Pending => {
                    break;
                }
            }
        }
        // we are now pending wake for room in the output OR new commands from the command_stream

        match pin!(&mut self.socket).poll_flush(context) {
            Poll::Ready(result) => {
                match result {
                    Ok(_) => (),
                    Err(e) => {
                        self.fail(Error::Socket(e.into()));
                        return Poll::Ready(());
                    }
                }
            }
            Poll::Pending => (),
        }

        Poll::Pending
    }
}

impl Socket {
    fn fail(&mut self, e: Error) {
        for responder in self.pending_commands.drain(..) {
            let _ = responder.response.send(Err(e.clone()));
        }
    }

    fn drain_replies(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Option<Poll<()>> {
        loop {
            match pin!(&mut self.socket).poll_next(context) {
                Poll::Ready(response) => {
                    match response {
                        Some(result) => {
                            match result {
                                Ok(reply) => {
                                    match websocket::SocketResponse::decode(reply.into_data().as_slice()) {
                                        Ok(response) => {
                                            let request_id = response.request_id as usize;
                                            match self.pending_commands.binary_search_by_key(&request_id, |pending| pending.id) {
                                                Ok(index) => {
                                                    let _ = self.pending_commands.remove(index).expect("it was found").response.send(Ok(response));
                                                }
                                                Err(_) => {
                                                    eprintln!("misaligned response id");
                                                }
                                            }
                                            continue;
                                        }
                                        Err(decode_error) => {
                                            self.fail(decode_error.into());
                                            return Some(Poll::Ready(()));
                                        }
                                    }
                                }
                                Err(e) => {
                                    self.fail(Error::Socket(Arc::new(e).into()));
                                    return Some(Poll::Ready(()))
                                }
                            }
                        }
                        None => {
                            error!("connection lost");
                            return Some(Poll::Ready(()))
                        }
                    }
                }
                Poll::Pending => {
                    break None; // done consuming replies
                }
            }
        }
    }

    fn output_has_room(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Result<bool, Poll<()>> {
        match pin!(&mut self.socket).poll_ready(context) {
            Poll::Ready(r) => {
                match r {
                    Ok(_) => {
                        Ok(true)
                    }
                    Err(e) => {
                        self.fail(Error::Socket(Arc::new(e)));
                        Err(Poll::Ready(()))
                    }
                }
            }
            Poll::Pending => {
                Ok(false)
            }
        }
    }
}
