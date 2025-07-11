use anyhow::{Result, anyhow};

use crate::network::client::MAX_MESSAGE_LENGTH;
use crate::network::protocol::header::Payload;
use crate::network::protocol::server::{HealthCheckPacket, HealthKind};
use crate::network::protocol::{Anchor, MediaType, UserStatus};

pub trait Serialize {
    fn serialize(self) -> Vec<u8>;
}

#[repr(u8)]
#[derive(Debug, Clone, PartialEq)]
pub enum ClientPacketType {
    Healthcheck = 0x80,
    Login = 0x81,
    Channels = 0x85,
}

impl Serialize for ClientPacketType {
    fn serialize(self) -> Vec<u8> {
        vec![self as u8]
    }
}

#[derive(Debug, Clone)]
pub enum ClientPayload {
    Login(LoginPacket),
    Health(HealthCheckPacket), // Send(SendMediaPacket),
    Channels,
}
impl From<ClientPayload> for Payload {
    fn from(payload: ClientPayload) -> Self {
        Payload::Client(payload)
    }
}

impl Serialize for ClientPayload {
    fn serialize(self) -> Vec<u8> {
        match self {
            ClientPayload::Login(packet) => packet.serialize(),
            ClientPayload::Health(packet) => packet.serialize(),
            ClientPayload::Channels => vec![],
        }
    }
}

impl Serialize for HealthKind {
    fn serialize(self) -> Vec<u8> {
        vec![self as u8]
    }
}

impl Serialize for HealthCheckPacket {
    fn serialize(self) -> Vec<u8> {
        self.kind.serialize()
    }
}

#[derive(Debug, Clone)]
pub struct LoginPacket {
    pub username: String,
    pub password: String,
}

impl Serialize for LoginPacket {
    fn serialize(self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.username.as_bytes());
        bytes.push(b'\0');
        bytes.extend(self.password.as_bytes());
        bytes
    }
}

#[derive(Debug, Clone)]
pub struct SendMessagePacket {
    pub channel_id: u64,
    pub reply_id: u64,
    pub media_ids: Vec<u64>,
    pub message_text: String,
}

#[derive(Debug, Clone)]
pub struct SendMediaPacket {
    pub filename: String,
    pub media_type: MediaType,
    pub media_data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct GetChannelsPacket {
    pub channel_ids: Vec<u64>,
}

#[derive(Debug, Clone)]
pub struct GetHistoryPacket {
    pub channel_id: u64,
    pub anchor: Anchor,
    pub num_messages_back: i8,
}

#[derive(Debug, Clone)]
pub struct GetUsersPacket {
    pub user_ids: Vec<u64>,
}

#[derive(Debug, Clone)]
pub struct GetMediaPacket {
    pub media_id: u64,
}

#[derive(Debug, Clone)]
pub struct TypingPacket {
    pub is_typing: bool,
    pub channel_id: u64,
}

#[derive(Debug, Clone)]
pub struct UserTypingPacket {
    pub is_typing: bool,
    pub user_id: u64,
    pub channel_id: u64,
}

#[derive(Debug, Clone)]
pub struct StatusPacket {
    pub status: UserStatus,
}

#[derive(Debug, Clone)]
pub struct UserStatusPacket {
    pub status: UserStatus,
    pub user_id: u64,
}
