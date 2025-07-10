use anyhow::{Result, anyhow};

use crate::network::client::MAX_MESSAGE_LENGTH;

pub trait Serialize {
    fn serialize(self) -> Vec<u8>;
}
pub trait Deserialize: Sized {
    fn deserialize(bytes: &[u8]) -> Result<Self>;
}

#[repr(u8)]
#[derive(Debug, Clone, PartialEq)]
pub enum PacketType {
    // Client
    ClientHealthcheck = 0x80,
    Login = 0x81,
    // Server
    ServerHealthcheck = 0x00,
    LoginAck = 0x01,
}

impl Serialize for PacketType {
    fn serialize(self) -> Vec<u8> {
        vec![self as u8]
    }
}
impl Deserialize for PacketType {
    fn deserialize(bytes: &[u8]) -> Result<Self> {
        match bytes[0] {
            0x00 => Ok(PacketType::ServerHealthcheck),
            0x01 => Ok(PacketType::LoginAck),
            other => Err(anyhow!("Unknown PacketType value: {:#04x}", other)),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone)]
pub enum PacketVersion {
    V1 = 0x01,
}

impl Deserialize for PacketVersion {
    fn deserialize(bytes: &[u8]) -> Result<Self> {
        match bytes[0] {
            0x01 => Ok(PacketVersion::V1),
            other => Err(anyhow!("Unknown PacketVersion value: {:#04x}", other)),
        }
    }
}

#[derive(Debug)]
pub struct Header {
    pub magic_number: [u8; 4],   // 4 bytes "CHTG"
    pub version: PacketVersion,  // 1 byte
    pub packet_type: PacketType, // 1 byte [is_user|1][packet_id|7]
    pub length: u32,             // 4 bytes length of content in bytes
}

impl Serialize for Header {
    fn serialize(self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(10);
        bytes.extend_from_slice(&self.magic_number); // 4 bytes
        bytes.push(self.version.clone() as u8); // 1 byte
        bytes.push(self.packet_type as u8); // 1 byte
        bytes.extend_from_slice(&self.length.to_be_bytes()); // 4 bytes (assumed big endian)
        bytes
    }
}

impl Deserialize for Header {
    fn deserialize(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 10 {
            return Err(anyhow!("Not enough bytes to deserialize Header"));
        }

        let magic_number = bytes[0..4].try_into()?;

        if magic_number != [b'C', b'H', b'T', b'G'] {
            return Err(anyhow!("Invalid magic number"));
        }
        let version = PacketVersion::deserialize(&bytes[4..5])?;
        let packet_type = PacketType::deserialize(&[bytes[5]])?;
        let length = u32::from_be_bytes(bytes[6..10].try_into()?);

        Ok(Header {
            magic_number,
            version,
            packet_type,
            length,
        })
    }
}

impl Header {
    pub fn new(packet_type: PacketType, length: u32) -> Header {
        Header {
            magic_number: [b'C', b'H', b'T', b'G'],
            version: PacketVersion::V1,
            packet_type,
            length,
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

#[repr(u8)]
#[derive(Debug, Clone)]
pub enum MediaType {
    Raw = 0x00,
    Text = 0x01,
    Audio = 0x02,
    Image = 0x03,
    Video = 0x04,
}

#[repr(u8)]
#[derive(Debug, Clone)]
pub enum UserStatus {
    Offline = 0x00,
    Online = 0x01,
    Idle = 0x02,
    DoNotDisturb = 0x03,
}

pub struct Packet {
    pub packet_type: PacketType,
    pub payload: Payload,
}

#[derive(Debug, Clone)]
pub enum Payload {
    HealthCheck(HealthCheckPacket),
    Login(LoginPacket),
    LoginAck(LoginAckPacket),
    // SendMessage(SendMessagePacket),
}

impl Serialize for Payload {
    fn serialize(self) -> Vec<u8> {
        match self {
            Payload::HealthCheck(packet) => packet.serialize(),
            Payload::Login(packet) => packet.serialize(),
            load => todo!("Payload {load:?} not implemented"),
        }
    }
}

impl Payload {
    pub fn deserialize_packet(bytes: &[u8], packet_type: PacketType) -> Result<Self> {
        match packet_type {
            PacketType::LoginAck => {
                let status = match bytes[0] {
                    0x00 => Status::Success,
                    0x01 => Status::Failed,
                    _ => return Err(anyhow!("Unknown status byte")),
                };

                let error_message = if status == Status::Failed {
                    let length = bytes.iter().position(|&b| b == 0).unwrap_or(MAX_MESSAGE_LENGTH);
                    Some(String::from_utf8(bytes[1..length].to_vec())?)
                } else {
                    None
                };

                Ok(Payload::LoginAck(LoginAckPacket { status, error_message }))
            }
            PacketType::ServerHealthcheck => {
                let kind = match bytes[0] {
                    0x00 => HealthKind::Ping,
                    0x01 => HealthKind::Pong,
                    k => return Err(anyhow!("Unknown health check kind {k}")),
                };
                Ok(Payload::HealthCheck(HealthCheckPacket { kind }))
            }
            packet => Err(anyhow!("deserialization not yet implemented for {packet:?}")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LoginAckPacket {
    pub status: Status,
    pub error_message: Option<String>,
}

#[repr(u8)]
#[derive(Debug, Clone, PartialEq)]
pub enum HealthKind {
    Ping = 0x00,
    Pong = 0x01,
}

#[derive(Debug, Clone)]
pub struct HealthCheckPacket {
    pub kind: HealthKind,
}

impl Serialize for HealthCheckPacket {
    fn serialize(self) -> Vec<u8> {
        vec![self.kind as u8]
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
pub struct SendMessageAckPacket {
    pub status: Status,
    pub message_id: u64,
    pub error_message: String,
}

#[derive(Debug, Clone)]
pub struct SendMediaPacket {
    pub filename: String,
    pub media_type: MediaType,
    pub media_data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct SendMediaAckPacket {
    pub status: Status,
    pub media_id: u64,
    pub error_message: String,
}

#[derive(Debug, Clone)]
pub struct GetChannelsPacket {
    pub channel_ids: Vec<u64>,
}

#[derive(Debug, Clone)]
pub struct ChannelsListPacket {
    pub status: Status,
    pub channel_ids: Vec<u64>,
    pub error_message: String,
}

#[derive(Debug, Clone)]
pub struct Channel {
    pub channel_id: u64,
    pub name: String,
    pub icon_id: u64,
}

#[derive(Debug, Clone)]
pub struct GetChannelsResponsePacket {
    pub status: Status,
    pub channels: Vec<Channel>,
    pub error_message: String,
}

#[derive(Debug, Clone)]
pub enum Anchor {
    Timestamp(u64), // MSB = 0
    MessageId(u64), // MSB = 1
}

#[derive(Debug, Clone)]
pub struct GetHistoryPacket {
    pub channel_id: u64,
    pub anchor: Anchor,
    pub num_messages_back: i8,
}

#[derive(Debug, Clone)]
pub struct HistoryMessage {
    pub message_id: u64,
    pub sent_timestamp: u64,
    pub user_id: u64,
    pub channel_id: u64,
    pub reply_id: u64,
    pub message_text: String,
    pub media_ids: Vec<u64>,
}

#[derive(Debug, Clone)]
pub struct HistoryPacket {
    pub status: Status,
    pub messages: Vec<HistoryMessage>,
    pub error_message: String,
}

#[derive(Debug, Clone)]
pub struct GetUsersPacket {
    pub user_ids: Vec<u64>,
}

#[derive(Debug, Clone)]
pub struct UsersListPacket {
    pub status: Status,
    pub users: Vec<(u64, UserStatus)>,
    pub error_message: String,
}

#[derive(Debug, Clone)]
pub struct UserData {
    pub user_id: u64,
    pub status: UserStatus,
    pub username: String,
    pub pfp_id: u64,
    pub bio: String,
}

#[derive(Debug, Clone)]
pub struct UsersPacket {
    pub status: Status,
    pub users: Vec<UserData>,
    pub error_message: String,
}

#[derive(Debug, Clone)]
pub struct GetMediaPacket {
    pub media_id: u64,
}

#[derive(Debug, Clone)]
pub struct MediaPacket {
    pub status: Status,
    pub filename: String,
    pub media_type: MediaType,
    pub media_data: Vec<u8>,
    pub error_message: String,
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

#[derive(Debug, Clone)]
pub struct UserConfigSetPacket {
    // TODO: Define fields
}

#[derive(Debug, Clone)]
pub struct UserConfigAckPacket {
    // TODO: Define fields
}
