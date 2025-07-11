use anyhow::{Result, anyhow};
use log::Log;

use crate::network::client::MAX_MESSAGE_LENGTH;
use crate::network::protocol::header::Payload;
use crate::network::protocol::{Channel, HistoryMessage, MediaType, UserData, UserStatus};

pub trait Deserialize: Sized {
    fn deserialize(bytes: &[u8]) -> Result<Self>;
}

#[repr(u8)]
#[derive(Debug, Clone, PartialEq)]
pub enum ServerPacketType {
    Healthcheck = 0x00,
    LoginAck = 0x01,
    Channels = 0x05,
}

impl Deserialize for ServerPacketType {
    fn deserialize(bytes: &[u8]) -> Result<Self> {
        match bytes[0] {
            0x00 => Ok(ServerPacketType::Healthcheck),
            0x01 => Ok(ServerPacketType::LoginAck),
            other => Err(anyhow!("Unknown ServerPacketType value: {:#04x}", other)),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ServerPayload {
    Health(HealthCheckPacket),
    Login(LoginAckPacket),
    Channels(GetChannelsResponsePacket),
}

impl From<ServerPayload> for Payload {
    fn from(payload: ServerPayload) -> Self {
        Payload::Server(payload)
    }
}

impl ServerPayload {
    pub fn deserialize_packet(bytes: &[u8], packet_type: ServerPacketType) -> Result<Self> {
        match packet_type {
            ServerPacketType::LoginAck => {
                let status = Status::deserialize(&bytes[0..1])?;

                let error_message = if status == Status::Failed {
                    Some(String::deserialize(&bytes[1..])?)
                } else {
                    None
                };

                Ok(ServerPayload::Login(LoginAckPacket { status, error_message }))
            }
            ServerPacketType::Healthcheck => {
                let kind = HealthKind::deserialize(&bytes[0..1])?;
                Ok(ServerPayload::Health(HealthCheckPacket { kind }))
            }
            ServerPacketType::Channels => {
                let status = Status::deserialize(&bytes[0..1])?;
                let channels = vec![];

                let error_message = if status == Status::Failed {
                    Some(String::deserialize(&bytes[1..])?)
                } else {
                    None
                };

                Ok(ServerPayload::Channels(GetChannelsResponsePacket {
                    status,
                    channels,
                    error_message,
                }))
            }
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    Success = 0x00,
    Failed = 0x01,
    Notification = 0x02, // Only used for HISTORY
}

impl Deserialize for Status {
    fn deserialize(bytes: &[u8]) -> Result<Self> {
        match bytes[0] {
            0x00 => Ok(Status::Success),
            0x01 => Ok(Status::Failed),
            0x02 => Ok(Status::Notification),
            _ => Err(anyhow!("Unknown status byte")),
        }
    }
}

impl Deserialize for String {
    fn deserialize(bytes: &[u8]) -> Result<Self> {
        let length = bytes.iter().position(|&b| b == 0).unwrap_or(MAX_MESSAGE_LENGTH);
        let string = String::from_utf8(bytes[0..length].to_vec())?;
        Ok(string)
    }
}

#[repr(u8)]
#[derive(Debug, Clone, PartialEq)]
pub enum HealthKind {
    Ping = 0x00,
    Pong = 0x01,
}

impl Deserialize for HealthKind {
    fn deserialize(bytes: &[u8]) -> Result<Self> {
        match bytes[0] {
            0x00 => Ok(HealthKind::Ping),
            0x01 => Ok(HealthKind::Pong),
            k => Err(anyhow!("Unknown health check kind {k}")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HealthCheckPacket {
    pub kind: HealthKind,
}

#[derive(Debug, Clone)]
pub struct LoginAckPacket {
    pub status: Status,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SendMessageAckPacket {
    pub status: Status,
    pub message_id: u64,
    pub error_message: String,
}

#[derive(Debug, Clone)]
pub struct SendMediaAckPacket {
    pub status: Status,
    pub media_id: u64,
    pub error_message: String,
}

#[derive(Debug, Clone)]
pub struct ChannelsListPacket {
    pub status: Status,
    pub channel_ids: Vec<u64>,
    pub error_message: String,
}

#[derive(Debug, Clone)]
pub struct GetChannelsResponsePacket {
    pub status: Status,
    pub channels: Vec<Channel>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UsersPacket {
    pub status: Status,
    pub users: Vec<UserData>,
    pub error_message: String,
}

#[derive(Debug, Clone)]
pub struct HistoryPacket {
    pub status: Status,
    pub messages: Vec<HistoryMessage>,
    pub error_message: String,
}
#[derive(Debug, Clone)]
pub struct UsersListPacket {
    pub status: Status,
    pub users: Vec<(u64, UserStatus)>,
    pub error_message: String,
}

#[derive(Debug, Clone)]
pub struct MediaPacket {
    pub status: Status,
    pub filename: String,
    pub media_type: MediaType,
    pub media_data: Vec<u8>,
    pub error_message: String,
}
