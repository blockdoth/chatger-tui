use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;

use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use clap::Error;
use clap::builder::Str;
use futures::lock;
use log::{debug, error, info};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::{Mutex, oneshot};
use tokio::task::JoinHandle;
use tokio::time::{Duration, sleep};

use crate::network::protocol::client::{
    Anchor, ClientPacketType, ClientPayload, GetChannelsPacket, GetHistoryPacket, GetUsersPacket, LoginPacket, SendMessagePacket, Serialize,
};
use crate::network::protocol::header::{Header, PacketType};
use crate::network::protocol::server::{Deserialize, HealthCheckPacket, HealthKind, ServerPacketType, ServerPayload, Status};
use crate::tui::chat::ChatMessage;
use crate::tui::events::TuiEvent;
use crate::tui::framework::Tui;

pub const MAX_MESSAGE_LENGTH: usize = 1024; // TODO figure out actual max size

pub struct Client {
    is_connected: bool,
    write_stream: Option<Arc<Mutex<OwnedWriteHalf>>>,
    event_send: Sender<TuiEvent>,
}

impl Client {
    pub fn new(event_send: Sender<TuiEvent>) -> Self {
        Client {
            is_connected: false,
            write_stream: None,
            event_send,
        }
    }
    pub async fn connect(&mut self, target_addr: SocketAddr) -> Result<()> {
        if self.is_connected {
            return Err(anyhow!("Already connected to {}", target_addr));
        }

        let connection = TcpStream::connect(target_addr).await?;
        let (mut read_stream, write_stream) = connection.into_split();
        let write_stream = Arc::new(Mutex::new(write_stream));
        let src_addr = read_stream.local_addr().unwrap();

        self.write_stream = Some(write_stream.clone());
        info!("Connected to {target_addr} from {src_addr}");

        self.receiving_task(read_stream, write_stream).await;
        self.event_send.send(TuiEvent::HealthCheck).await?;
        Ok(())
    }

    pub async fn login(&mut self, username: String, password: String) -> Result<()> {
        let mut write_stream = self.write_stream.as_mut().ok_or_else(|| anyhow!("Not connected to server"))?.lock().await;

        Self::send_message(
            &mut write_stream,
            ClientPacketType::Login,
            ClientPayload::Login(LoginPacket { username, password }),
        )
        .await
    }

    pub async fn request_channels(&mut self, channel_ids: Vec<u64>) -> Result<()> {
        let mut write_stream = self.write_stream.as_mut().ok_or_else(|| anyhow!("Not connected to server"))?.lock().await;

        Self::send_message(
            &mut write_stream,
            ClientPacketType::Channels,
            ClientPayload::Channels(GetChannelsPacket { channel_ids }),
        )
        .await
    }

    pub async fn request_channel_ids(&mut self) -> Result<()> {
        let mut write_stream = self.write_stream.as_mut().ok_or_else(|| anyhow!("Not connected to server"))?.lock().await;

        Self::send_message(&mut write_stream, ClientPacketType::ChannelsList, ClientPayload::ChannelsList).await
    }

    pub async fn request_user_statuses(&mut self) -> Result<()> {
        let mut write_stream = self.write_stream.as_mut().ok_or_else(|| anyhow!("Not connected to server"))?.lock().await;

        Self::send_message(&mut write_stream, ClientPacketType::UserStatuses, ClientPayload::UserStatuses).await
    }

    pub async fn request_users(&mut self, user_ids: Vec<u64>) -> Result<()> {
        let mut write_stream = self.write_stream.as_mut().ok_or_else(|| anyhow!("Not connected to server"))?.lock().await;

        Self::send_message(
            &mut write_stream,
            ClientPacketType::Users,
            ClientPayload::Users(GetUsersPacket { user_ids }),
        )
        .await
    }

    pub async fn request_history_by_timestamp(&mut self, channel_id: u64, timestamp: DateTime<Utc>, num_messages_back: i8) -> Result<()> {
        let mut write_stream = self.write_stream.as_mut().ok_or_else(|| anyhow!("Not connected to server"))?.lock().await;

        Self::send_message(
            &mut write_stream,
            ClientPacketType::History,
            ClientPayload::History(GetHistoryPacket {
                channel_id,
                anchor: Anchor::Timestamp(timestamp.timestamp() as u64),
                num_messages_back,
            }),
        )
        .await
    }

    pub async fn send_chat_message(&mut self, channel_id: u64, reply_id: u64, message_text: String, media_ids: Vec<u64>) -> Result<()> {
        let mut write_stream = self.write_stream.as_mut().ok_or_else(|| anyhow!("Not connected to server"))?.lock().await;
        Self::send_message(
            &mut write_stream,
            ClientPacketType::SendMessage,
            ClientPayload::SendMessage(SendMessagePacket {
                channel_id,
                reply_id,
                message_text,
                media_ids,
            }),
        )
        .await
    }

    async fn receiving_task(&mut self, mut read_stream: OwnedReadHalf, write_stream: Arc<Mutex<OwnedWriteHalf>>) {
        info!("Started receiving task");
        let write_stream = self.write_stream.clone();
        let event_send = self.event_send.clone();
        tokio::spawn(async move {
            let mut header_buffer: [u8; 10] = [0; 10];
            let mut payload_buffer: [u8; MAX_MESSAGE_LENGTH] = [0; MAX_MESSAGE_LENGTH];
            let mut stream = if let Some(stream) = write_stream {
                stream
            } else {
                error!("No write stream available");
                return;
            };
            loop {
                match Self::read_message(&mut read_stream, &mut header_buffer, &mut payload_buffer).await {
                    Ok(payload) => {
                        if let Err(e) = Self::handle_message(payload, &mut stream, event_send.clone()).await {
                            error!("Error while handling message: {e:?}");
                        }
                    }
                    Err(e) => {
                        error!("Error while reading message: {e:?}");
                        let _ = event_send.send(TuiEvent::Disconnected).await;
                        break;
                    }
                }
            }

            info!("Stopped receiving task");
        });
    }

    async fn handle_message(payload: ServerPayload, stream: &mut Arc<Mutex<OwnedWriteHalf>>, event_send: Sender<TuiEvent>) -> Result<()> {
        match payload {
            ServerPayload::Health(packet) => match packet.kind {
                HealthKind::Ping => {
                    let mut stream = stream.lock().await;
                    let payload = ClientPayload::Health(HealthCheckPacket { kind: HealthKind::Pong });
                    Self::send_message(&mut stream, ClientPacketType::Healthcheck, payload).await?;
                    event_send.send(TuiEvent::HealthCheck).await?;
                    Ok(())
                }
                HealthKind::Pong => panic!("todo"),
            },
            ServerPayload::Login(packet) => match packet.status {
                Status::Success => {
                    info!("Succefully logged in");
                    event_send.send(TuiEvent::LoggedIn).await?;
                    Ok(())
                }
                Status::Failed => match packet.error_message {
                    Some(message) => {
                        error!("failed to log in {message}");
                        Err(anyhow!("failed to log in {}", message))
                    }
                    None => {
                        error!("failed to log in");
                        Err(anyhow!("failed to log in"))
                    }
                },
                Status::Notification => panic!("todo"),
            },
            ServerPayload::Channels(packet) => match packet.status {
                Status::Success => {
                    event_send.send(TuiEvent::Channels(packet.channels)).await?;
                    Ok(())
                }
                Status::Failed => todo!(),
                Status::Notification => panic!("todo"),
            },
            ServerPayload::ChannelsList(packet) => match packet.status {
                Status::Success => {
                    event_send.send(TuiEvent::ChannelIDs(packet.channel_ids)).await?;
                    Ok(())
                }
                Status::Failed => todo!(),
                Status::Notification => panic!("todo"),
            },
            ServerPayload::UserStatuses(packet) => match packet.status {
                Status::Success => {
                    event_send.send(TuiEvent::UserStatusesUpdate(packet.users)).await?;
                    Ok(())
                }
                Status::Failed => todo!(),
                Status::Notification => panic!("todo"),
            },
            ServerPayload::Users(packet) => match packet.status {
                Status::Success => {
                    event_send.send(TuiEvent::Users(packet.users)).await?;
                    Ok(())
                }
                Status::Failed => todo!(),
                Status::Notification => panic!("todo"),
            },
            ServerPayload::History(packet) => match packet.status {
                Status::Success => {
                    event_send.send(TuiEvent::HistoryUpdate(packet.messages)).await?;
                    Ok(())
                }
                Status::Failed => todo!(),
                Status::Notification => panic!("todo"),
            },
            ServerPayload::SendMessageAck(packet) => match packet.status {
                Status::Success => {
                    event_send.send(TuiEvent::MessageSendAck(packet.message_id)).await?;
                    Ok(())
                }
                Status::Failed => {
                    error!("Failed to send message {:?}", packet.error_message);
                    Ok(())
                }
                Status::Notification => {
                    info!("Got message notification from server TODO handle");
                    Ok(())
                }
            },
        }
    }
}

impl Client {
    pub async fn send_message(stream: &mut OwnedWriteHalf, packet_type: ClientPacketType, payload: ClientPayload) -> Result<()> {
        debug!("Sending packet type: {packet_type:?}");

        let payload_serialized = payload.serialize();
        let header = Header::new(packet_type.into(), payload_serialized.len() as u32);
        // debug!("Header {header:?}");
        let mut packet = header.serialize();

        debug!("Send header bytes: {packet:?}");
        debug!("Send payload bytes: {payload_serialized:?}");

        packet.extend(payload_serialized);

        stream.write_all(&packet).await?;

        stream.flush().await?;
        Ok(())
    }

    pub async fn read_message(stream: &mut OwnedReadHalf, header_buffer: &mut [u8], payload_buffer: &mut [u8]) -> Result<ServerPayload> {
        stream.read_exact(&mut header_buffer[..]).await?;

        debug!("Received header bytes {header_buffer:?}");
        let header = Header::deserialize(header_buffer)?.0;
        debug!("Received {header:?}");

        let payload_size = header.length;
        debug!("Waiting to read payload of size {payload_size}");
        stream.read_exact(&mut payload_buffer[0..payload_size as usize]).await?;
        debug!("{payload_size} bytes read");

        let packet_type = match header.packet_type {
            PacketType::Server(packet_type) => packet_type,
            PacketType::Client(packet_type) => return Err(anyhow!("Recevied packet type {packet_type:?}, which is a client packet")),
        };

        let payload = ServerPayload::deserialize_packet(payload_buffer, packet_type)?;
        debug!("Deserialized payload {payload:?}");

        Ok(payload)
    }
}
