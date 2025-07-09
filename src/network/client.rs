use std::net::SocketAddr;

use anyhow::{Ok, Result, anyhow};
use clap::Error;
use log::{debug, error, info};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

use crate::network::protocol::{
    Deserialize, Header, HealthCheckPacket, HealthKind, LoginPacket, Packet, PacketType, PacketVersion, Payload, Serialize,
};

pub const MAX_MESSAGE_LENGTH: usize = 1024; // TODO figure out actual max size

pub struct Client {
    is_connected: bool,
    read_stream: Option<OwnedReadHalf>,
    write_stream: Option<OwnedWriteHalf>,
    header_buffer: [u8; 10],
    payload_buffer: [u8; MAX_MESSAGE_LENGTH],
}

impl Client {
    pub fn new() -> Self {
        Client {
            is_connected: false,
            read_stream: None,
            write_stream: None,
            header_buffer: [0; 10],
            payload_buffer: [0; MAX_MESSAGE_LENGTH],
        }
    }

    pub async fn connect(&mut self, target_addr: SocketAddr) -> Result<()> {
        if self.is_connected {
            return Err(anyhow!("Already connected to {}", target_addr));
        }

        let connection = TcpStream::connect(target_addr).await?;
        let (read_stream, mut write_stream) = connection.into_split();
        let src_addr = write_stream.local_addr().unwrap();

        self.read_stream = Some(read_stream);
        self.write_stream = Some(write_stream);

        info!("Connected to {target_addr} from {src_addr}");

        Ok(())
    }

    pub async fn login(&mut self, username: String, password: String) -> Result<()> {
        self.send_message(PacketType::Login, Payload::Login(LoginPacket { username, password }))
            .await?;

        let response = self.read_message().await?;
        info!("{response:?}");
        Ok(())
    }

    pub async fn healthcheck(&mut self) -> Result<()> {
        self.send_message(PacketType::Ping, Payload::HealthCheck(HealthCheckPacket { kind: HealthKind::Ping }))
            .await?;
        let response = self.read_message().await?;
        info!("{response:?}");
        Ok(())
    }

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

        debug!("Header bytes: {packet:?}");
        debug!("Payload bytes: {payload_serialized:?}");

        packet.extend(payload_serialized);

        stream.write_all(&packet).await?;

        stream.flush().await?;
        Ok(())
    }

    pub async fn read_message(&mut self) -> Result<Payload> {
        let mut stream = self.read_stream.as_mut().ok_or_else(|| anyhow!("Cannot read message if not connected"))?;
        let mut header_buffer = &mut self.header_buffer;
        let mut payload_buffer = &mut self.payload_buffer;

        stream.read_exact(&mut header_buffer[..]).await?;

        let header = Header::deserialize(header_buffer)?;
        debug!("Received header {header_buffer:?}");
        debug!("Received header {header:?}");

        let mut bytes_read: usize = 0;
        let payload_length = header.length as usize;
        while bytes_read < payload_length {
            let n_read: usize = stream.read(&mut payload_buffer[bytes_read..payload_length]).await?;
            debug!("Read {n_read} bytes from buffer");
            if n_read == 0 {
                error!("stream closed before all bytes were read ({bytes_read}/{payload_length})");
                return Err(anyhow!("stream closed"));
            }
            bytes_read += n_read;
        }

        let payload = Payload::deserialize_packet(payload_buffer, header.packet_type)?;

        Ok(payload)
    }
}
