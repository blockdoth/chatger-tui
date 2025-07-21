use std::hash::{Hash, Hasher};
use std::net::{IpAddr, SocketAddr, SocketAddrV4};
use std::ops::DerefMut;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use log::{debug, error, info};
use rustls::pki_types::ServerName;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::mpsc::Sender;
use tokio::sync::{Mutex, MutexGuard};
use tokio::task::JoinHandle;
use tokio_rustls::TlsConnector;

use crate::network::handle_message;
use crate::network::protocol::UserStatus;
use crate::network::protocol::client::{
    Anchor, ClientPacketType, ClientPayload, GetChannelsPacket, GetHistoryPacket, GetUsersPacket, LoginPacket, SendMessagePacket, Serialize,
    StatusPacket, TypingPacket,
};
use crate::network::protocol::header::{Header, PacketType};
use crate::network::protocol::server::{Deserialize, HealthCheckPacket, HealthKind, ServerPayload};
use crate::tui::events::TuiEvent;

pub const MAX_MESSAGE_LENGTH: usize = 16 * 1024; // TODO figure out actual max size

#[derive(Debug, PartialEq, Clone)]
pub enum ServerConnectionStatus {
    Connected,
    Unhealthy,
    Disconnected,
    Reconnecting,
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum ConnectionType {
    Raw,
    #[allow(clippy::upper_case_acronyms)]
    TLS, // domain name for TLS
}

#[derive(Debug, Clone)]
pub struct ServerAddrInfo {
    pub ip: IpAddr,
    pub port: u16,
    pub domain: Option<String>,
    pub connection_type: ConnectionType,
}

// PartialEq and Eq that excludes domain
impl PartialEq for ServerAddrInfo {
    fn eq(&self, other: &Self) -> bool {
        self.ip == other.ip && self.port == other.port && self.connection_type == other.connection_type
    }
}

impl Eq for ServerAddrInfo {}

// Hash that excludes domain
impl Hash for ServerAddrInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ip.hash(state);
        self.port.hash(state);
        self.connection_type.hash(state);
    }
}
pub struct Client {
    write_stream: Option<Box<dyn AsyncWrite + Send + Unpin>>,
    event_send: Sender<TuiEvent>,
    recv_handle: Option<JoinHandle<()>>,
    pub time_since_last_transmit: InteractedTimeStamp,
    pub time_since_last_reconnect: InteractedTimeStamp,
    pub connection_status: ServerConnectionStatus,
}

impl Client {
    pub fn new(event_send: Sender<TuiEvent>) -> Self {
        Client {
            write_stream: None,
            event_send,
            recv_handle: None,
            time_since_last_transmit: InteractedTimeStamp::new(),
            time_since_last_reconnect: InteractedTimeStamp::new(),
            connection_status: ServerConnectionStatus::Disconnected,
        }
    }

    pub fn get_stream(&mut self) -> Result<&mut (dyn AsyncWrite + Send + Unpin)> {
        Ok(self.write_stream.as_mut().ok_or_else(|| anyhow!("Not connected to server"))?)
    }

    pub async fn connect(&mut self, server_connection: &ServerAddrInfo) -> Result<()> {
        if self.write_stream.is_some() {
            if let Some(domain) = &server_connection.domain {
                return Err(anyhow!(
                    "Already connected to {domain}:{} ({})",
                    server_connection.port,
                    server_connection.ip
                ));
            } else {
                return Err(anyhow!("Already connected to {}:{}", server_connection.port, server_connection.ip));
            }
        }

        let target_addr = SocketAddr::new(server_connection.ip, server_connection.port);
        let connection_tcp = TcpStream::connect(target_addr).await?;
        let src_addr = connection_tcp.local_addr().unwrap();

        match server_connection.connection_type {
            ConnectionType::Raw => {
                let (read_stream, write_stream) = connection_tcp.into_split();

                info!("Connected to {target_addr} from {src_addr}");
                self.write_stream = Some(Box::new(write_stream));
                self.recv_handle = Some(self.receiving_task(Box::new(read_stream)).await);
                self.connection_status = ServerConnectionStatus::Connected;
            }
            ConnectionType::TLS => {
                if let Some(domain) = server_connection.domain.clone() {
                    // Source: https://docs.rs/rustls/latest/rustls/
                    let root_store = rustls::RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

                    let config = rustls::ClientConfig::builder().with_root_certificates(root_store).with_no_client_auth();

                    let connector = TlsConnector::from(Arc::new(config));

                    let domain_name = ServerName::try_from(domain)?;

                    let connection_tls = connector.connect(domain_name, connection_tcp).await?;
                    let (read_stream, write_stream) = tokio::io::split(connection_tls);

                    info!("Connected to {target_addr} from {src_addr} over TLS");
                    self.write_stream = Some(Box::new(write_stream));
                    self.recv_handle = Some(self.receiving_task(Box::new(read_stream)).await);
                    self.connection_status = ServerConnectionStatus::Connected;
                } else {
                    return Err(anyhow!("TLS requires a domain"));
                }
            }
        };

        Ok(())
    }

    pub fn disconnect(&mut self) -> Result<()> {
        self.write_stream = None;
        if let Some(recv_handle) = &self.recv_handle {
            recv_handle.abort();
        }
        debug!("Disconnected from server");
        self.connection_status = ServerConnectionStatus::Disconnected;
        Ok(())
    }

    pub async fn reconnect(&mut self, server_address: &ServerAddrInfo, username: String, password: String) -> Result<()> {
        self.disconnect()?;
        self.connection_status = ServerConnectionStatus::Reconnecting;
        self.connect(server_address).await?;
        self.login(username, password).await?;
        self.time_since_last_reconnect.update();
        Ok(())
    }

    pub async fn send_healthcheck(&mut self) -> Result<()> {
        let interacted_ts = self.time_since_last_transmit.clone();
        let mut write_stream = self.get_stream()?;

        Self::send_message(
            write_stream,
            interacted_ts,
            ClientPacketType::Healthcheck,
            ClientPayload::Health(HealthCheckPacket { kind: HealthKind::Pong }),
        )
        .await
    }

    pub async fn login(&mut self, username: String, password: String) -> Result<()> {
        let interacted_ts = self.time_since_last_transmit.clone();
        let mut write_stream = self.get_stream()?;

        Self::send_message(
            &mut write_stream.deref_mut(),
            interacted_ts,
            ClientPacketType::Login,
            ClientPayload::Login(LoginPacket { username, password }),
        )
        .await
    }

    pub async fn request_channels(&mut self, channel_ids: Vec<u64>) -> Result<()> {
        let interacted_ts = self.time_since_last_transmit.clone();
        let mut write_stream = self.get_stream()?;

        Self::send_message(
            write_stream,
            interacted_ts,
            ClientPacketType::Channels,
            ClientPayload::Channels(GetChannelsPacket { channel_ids }),
        )
        .await
    }

    pub async fn request_channel_ids(&mut self) -> Result<()> {
        let interacted_ts = self.time_since_last_transmit.clone();
        let mut write_stream = self.get_stream()?;

        Self::send_message(write_stream, interacted_ts, ClientPacketType::ChannelsList, ClientPayload::ChannelsList).await
    }

    pub async fn request_user_statuses(&mut self) -> Result<()> {
        let interacted_ts = self.time_since_last_transmit.clone();
        let mut write_stream = self.get_stream()?;

        Self::send_message(write_stream, interacted_ts, ClientPacketType::UserStatuses, ClientPayload::UserStatuses).await
    }

    pub async fn request_users(&mut self, user_ids: Vec<u64>) -> Result<()> {
        let interacted_ts = self.time_since_last_transmit.clone();
        let mut write_stream = self.get_stream()?;

        Self::send_message(
            write_stream,
            interacted_ts,
            ClientPacketType::Users,
            ClientPayload::Users(GetUsersPacket { user_ids }),
        )
        .await
    }

    pub async fn request_history_by_timestamp(&mut self, channel_id: u64, timestamp: DateTime<Utc>, num_messages_back: i8) -> Result<()> {
        let interacted_ts = self.time_since_last_transmit.clone();
        let mut write_stream = self.get_stream()?;

        Self::send_message(
            write_stream,
            interacted_ts,
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
        let interacted_ts = self.time_since_last_transmit.clone();
        let mut write_stream = self.get_stream()?;

        Self::send_message(
            write_stream,
            interacted_ts,
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

    pub async fn send_typing(&mut self, channel_id: u64, is_typing: bool) -> Result<()> {
        let interacted_ts = self.time_since_last_transmit.clone();
        let mut write_stream = self.get_stream()?;

        Self::send_message(
            write_stream,
            interacted_ts,
            ClientPacketType::Typing,
            ClientPayload::Typing(TypingPacket { is_typing, channel_id }),
        )
        .await
    }

    pub async fn send_user_status(&mut self, status: UserStatus) -> Result<()> {
        let interacted_ts = self.time_since_last_transmit.clone();
        let mut write_stream = self.get_stream()?;

        Self::send_message(
            write_stream,
            interacted_ts,
            ClientPacketType::Status,
            ClientPayload::Status(StatusPacket { status }),
        )
        .await
    }

    async fn receiving_task(&mut self, mut read_stream: Box<dyn AsyncRead + Send + Unpin>) -> JoinHandle<()> {
        info!("Started receiving task");
        let event_send = self.event_send.clone();
        let interacted_timestamp = self.time_since_last_transmit.clone();

        tokio::spawn(async move {
            let mut header_buffer: [u8; 10] = [0; 10];
            let mut payload_buffer: [u8; MAX_MESSAGE_LENGTH] = [0; MAX_MESSAGE_LENGTH];
            loop {
                match Self::read_message(&mut read_stream, interacted_timestamp.clone(), &mut header_buffer, &mut payload_buffer).await {
                    Ok((payload, _bytes_read)) => {
                        // TODO something with bytes read
                        if let Err(e) = handle_message(payload, event_send.clone()).await {
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
        })
    }
}

// Actual sending and receiving functions
impl Client {
    pub async fn send_message(
        stream: &mut (dyn AsyncWrite + Send + Unpin),
        transmission_timestamp: InteractedTimeStamp,
        packet_type: ClientPacketType,
        payload: ClientPayload,
    ) -> Result<()> {
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
        transmission_timestamp.update();
        Ok(())
    }

    pub async fn read_message(
        stream: &mut (dyn AsyncRead + Send + Unpin),
        transmission_timestamp: InteractedTimeStamp,
        header_buffer: &mut [u8],
        payload_buffer: &mut [u8],
    ) -> Result<(ServerPayload, usize)> {
        stream.read_exact(&mut header_buffer[..]).await?;

        debug!("Received header bytes {header_buffer:?}");
        let header = Header::deserialize(header_buffer)?.0;
        debug!("Received {header:?}");

        let payload_size = header.length;
        if (payload_size + 10) as usize > MAX_MESSAGE_LENGTH {
            return Err(anyhow!("Max message length exceeded to large for packet {:?}", header.packet_type));
        }
        debug!("Waiting to read payload of size {payload_size}");
        stream.read_exact(&mut payload_buffer[0..payload_size as usize]).await?;
        debug!("{payload_size} bytes read");

        let packet_type = match header.packet_type {
            PacketType::Server(packet_type) => packet_type,
            PacketType::Client(packet_type) => return Err(anyhow!("Received packet type {packet_type:?}, which is a client packet")),
        };

        let payload = ServerPayload::deserialize_packet(payload_buffer, packet_type)?;
        debug!("Deserialized payload {payload:?}");
        transmission_timestamp.update();
        Ok(payload)
    }
}

#[derive(Clone)]
pub struct InteractedTimeStamp {
    inner: Arc<AtomicU64>,
}

impl InteractedTimeStamp {
    pub fn new() -> Self {
        InteractedTimeStamp {
            inner: Arc::new(AtomicU64::new(0)),
        }
    }

    fn now_millis() -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64
    }

    pub fn update(&self) {
        self.inner.store(Self::now_millis(), Ordering::Relaxed);
    }

    pub fn elapsed(&self) -> Duration {
        let now = Self::now_millis();
        let last = self.inner.load(Ordering::Relaxed);
        Duration::from_millis(now.saturating_sub(last))
    }
}
