use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;

use anyhow::{Result, anyhow};
use clap::Error;
use futures::lock;
use log::{debug, error, info};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::{Mutex, oneshot};
use tokio::task::JoinHandle;
use tokio::time::{Duration, sleep};

use crate::network::protocol::{
    Deserialize, Header, HealthCheckPacket, HealthKind, LoginPacket, Packet, PacketType, PacketVersion, Payload, Serialize,
};
use crate::tui::TuiEvent;

pub const MAX_MESSAGE_LENGTH: usize = 1024; // TODO figure out actual max size

pub type HandlerFuture = Box<dyn FnMut(Payload) -> Pin<Box<dyn Future<Output = bool> + Send>> + Send>;
// type HandlerFuture = Box<dyn FnMut(Payload) -> bool + Send>;
pub struct Client {
    is_connected: bool,
    write_stream: Option<OwnedWriteHalf>,
    pending_message_handlers: Arc<Mutex<Vec<HandlerFuture>>>,
}

impl Client {
    pub fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Client {
            is_connected: false,
            write_stream: None,
            pending_message_handlers: Arc::new(Mutex::new(vec![])),
        }))
    }

    pub async fn add_response_handler(&self, handler: HandlerFuture) {
        self.pending_message_handlers.lock().await.push(handler);
        debug!("Added response handler");
    }

    pub async fn connect(&mut self, target_addr: SocketAddr) -> Result<()> {
        if self.is_connected {
            return Err(anyhow!("Already connected to {}", target_addr));
        }

        let connection = TcpStream::connect(target_addr).await?;
        let (mut read_stream, write_stream) = connection.into_split();
        let src_addr = read_stream.local_addr().unwrap();
        self.write_stream = Some(write_stream);
        info!("Connected to {target_addr} from {src_addr}");

        self.receiving_task(read_stream).await;

        Ok(())
    }

    async fn receiving_task(&mut self, mut read_stream: OwnedReadHalf) {
        let pending_message_handlers = self.pending_message_handlers.clone();
        info!("Started receiving task");
        tokio::spawn(async move {
            let mut header_buffer: [u8; 10] = [0; 10];
            let mut payload_buffer: [u8; MAX_MESSAGE_LENGTH] = [0; MAX_MESSAGE_LENGTH];
            let mut response_payload_buffer: Vec<Payload> = vec![];

            loop {
                match Self::read_message(&mut read_stream, &mut header_buffer, &mut payload_buffer).await {
                    Ok(payload) => {
                        // info!("Received message {payload:?}");
                        response_payload_buffer.push(payload.clone());

                        let mut handlers = pending_message_handlers.lock().await;

                        let mut i = 0;
                        while i < handlers.len() {
                            if handlers[i](payload.clone()).await {
                                i += 1;
                            } else {
                                debug!("Removed response handler");
                                handlers.remove(i);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error while reading message: {e:?}");
                        break;
                    }
                }
            }
            info!("Stopped receiving task");
        });
    }

    pub async fn login(&mut self, username: String, password: String) -> Result<()> {
        let (response_send, response_recv) = oneshot::channel::<Payload>();

        let mut response_send_opt = Some(response_send); // Required to make the handler FnMut, moves response_send ONCE into closure

        let handler: HandlerFuture = Box::new(move |response: Payload| {
            let mut response_send_opt_taken = response_send_opt.take();
            Box::pin(async move {
                if let Payload::LoginAck(_) = response {
                    debug!("Running login response handler");
                    if let Some(response_send) = response_send_opt_taken {
                        let _ = response_send.send(response);
                        return true; // Remove handler if succeeded
                    }
                }
                false
            })
        });

        self.add_response_handler(handler).await;

        self.send_message(PacketType::Login, Payload::Login(LoginPacket { username, password }))
            .await?;

        let response = response_recv.await?;

        Ok(())
    }

    pub async fn server_healthcheck(client: Arc<Mutex<Client>>, response_recv: &mut Receiver<Payload>) -> Result<()> {
        client
            .lock()
            .await
            .send_message(
                PacketType::ClientHealthcheck,
                Payload::HealthCheck(HealthCheckPacket { kind: HealthKind::Ping }),
            )
            .await?;
        match response_recv.recv().await {
            Some(r) => {
                info!("Channel {r:?}");
            }
            None => {
                error!("Channel closed");
            }
        };
        Ok(())
    }
}

impl Client {
    pub async fn start_polling(
        client: Arc<Mutex<Client>>,
        address: SocketAddr,
        polling_interval: u64,
        event_send: Sender<TuiEvent>,
    ) -> JoinHandle<()> {
        let (response_send, response_recv) = mpsc::channel::<Payload>(10);

        let mut response_send_opt = Some(response_send); // Required to make the handler FnMut

        let handler: HandlerFuture = Box::new(move |response: Payload| {
            let mut response_send_opt_taken = response_send_opt.take();
            Box::pin(async move {
                if let Payload::HealthCheck(HealthCheckPacket { kind }) = &response
                    && *kind == HealthKind::Pong
                {
                    debug!("Running server healthcheck response handler");
                    // Only respond to pong packaet, for the server also sends ping packets
                    if let Some(response_send) = response_send_opt_taken {
                        match response_send.send(response).await {
                            Ok(r) => {
                                info!("Send response");
                            }
                            Err(e) => {
                                error!("Failed to send response");
                            }
                        };
                    }
                }
                false
            })
        });
        client.lock().await.add_response_handler(handler).await;

        tokio::spawn(async move {
            info!("Started polling task");
            let mut is_healthy = true;
            let mut response_recv = response_recv;
            loop {
                // if !is_healthy {
                //     match client.lock().await.connect(address).await {
                //         Ok(_) => {
                //             info!("Successfully reconnected to server {address:?}");
                //             is_healthy = true;
                //         }
                //         Err(e) => {
                //             error!("Failed to reconnect to server {address:?}: {e}");
                //         }
                //     }
                // }

                match Client::server_healthcheck(client.clone(), &mut response_recv).await {
                    Ok(_) => {
                        event_send.send(TuiEvent::HealthCheck(true)).await.expect("TODO error handling");
                        info!("Healthcheck succeeded");
                        is_healthy = true;
                    }
                    Err(e) => {
                        event_send.send(TuiEvent::HealthCheck(false)).await.expect("TODO error handling");
                        error!("Healthcheck failed: {e}");
                        is_healthy = false;
                    }
                }

                sleep(Duration::from_secs(polling_interval)).await;
            }
            info!("Ended polling task");
        })
    }

    pub async fn register_client_healthcheck_handler(client: Arc<Mutex<Client>>) -> Result<()> {
        let client = client.clone();
        let client_moved = client.clone();

        let handler: HandlerFuture = Box::new(move |response: Payload| {
            let client = client_moved.clone();
            Box::pin(async move {
                if let Payload::HealthCheck(HealthCheckPacket { kind }) = &response
                    && *kind == HealthKind::Ping
                {
                    debug!("Running client healthcheck handler");
                    let mut locked_client = client.lock().await;
                    debug!("locked client");
                    locked_client
                        .send_message(
                            PacketType::ClientHealthcheck,
                            Payload::HealthCheck(HealthCheckPacket { kind: HealthKind::Pong }),
                        )
                        .await
                        .expect("error handling")
                }
                false
            })
        });
        client.lock().await.add_response_handler(handler).await;

        Ok(())
    }
}

impl Client {
    pub async fn send_message(&mut self, packet_type: PacketType, payload: Payload) -> Result<()> {
        let mut stream = self
            .write_stream
            .as_mut()
            .ok_or_else(|| anyhow!("Cannot send message if not connected"))?;

        debug!("Sending packet type: {packet_type:?}");

        let payload_serialized = payload.serialize();
        let header = Header::new(packet_type, payload_serialized.len() as u32);
        // debug!("Header {header:?}");
        let mut packet = header.serialize();

        debug!("Send header bytes: {packet:?}");
        debug!("Send payload bytes: {payload_serialized:?}");

        packet.extend(payload_serialized);

        stream.write_all(&packet).await?;

        stream.flush().await?;
        Ok(())
    }

    pub async fn read_message(stream: &mut OwnedReadHalf, header_buffer: &mut [u8], payload_buffer: &mut [u8]) -> Result<Payload> {
        debug!("Waiting to read header");
        stream.read_exact(&mut header_buffer[..]).await?;

        debug!("Received header bytes {header_buffer:?}");
        let header = Header::deserialize(header_buffer)?;
        debug!("Received {header:?}");

        let payload_size = header.length;
        debug!("Waiting to read payload of size {payload_size}");
        stream.read_exact(&mut payload_buffer[0..payload_size as usize]).await?;
        debug!("{payload_size} bytes read");

        let payload = Payload::deserialize_packet(payload_buffer, header.packet_type)?;

        Ok(payload)
    }
}
