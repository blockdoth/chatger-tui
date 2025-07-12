use anyhow::{Result, anyhow};
use futures::channel;
use log::Log;

use crate::network::client::MAX_MESSAGE_LENGTH;
use crate::network::protocol::header::Payload;
use crate::network::protocol::{Channel, HistoryMessage, MediaType, UserData, UserStatus};

pub trait Deserialize: Sized {
    fn deserialize(bytes: &[u8]) -> Result<(Self, usize)>;
}
pub trait DeserializeByte: Sized {
    fn deserialize_byte(byte: u8) -> Result<(Self)>;
}

#[repr(u8)]
#[derive(Debug, Clone, PartialEq)]
pub enum ServerPacketType {
    Healthcheck = 0x00,
    LoginAck = 0x01,
    SendMessageAck = 0x02,
    ChannelList = 0x04,
    Channels = 0x05,
    History = 0x06,
    UserStatuses = 0x07,
    Users = 0x08,
}

impl DeserializeByte for ServerPacketType {
    fn deserialize_byte(byte: u8) -> Result<(Self)> {
        match byte {
            0x00 => Ok(ServerPacketType::Healthcheck),
            0x01 => Ok(ServerPacketType::LoginAck),
            0x02 => Ok(ServerPacketType::SendMessageAck),
            0x04 => Ok(ServerPacketType::ChannelList),
            0x05 => Ok(ServerPacketType::Channels),
            0x06 => Ok(ServerPacketType::History),
            0x07 => Ok(ServerPacketType::UserStatuses),
            0x08 => Ok(ServerPacketType::Users),
            other => Err(anyhow!("Unknown ServerPacketType value: {}", other)),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ServerPayload {
    Health(HealthCheckPacket),
    Login(LoginAckPacket),
    SendMessageAck(SendMessageAckPacket),
    Channels(GetChannelsResponsePacket),
    ChannelsList(ChannelsListPacket),
    UserStatuses(UserStatusesPacket),
    Users(UsersPacket),
    History(HistoryPacket),
}

impl From<ServerPayload> for Payload {
    fn from(payload: ServerPayload) -> Self {
        Payload::Server(payload)
    }
}

impl ServerPayload {
    pub fn deserialize_packet(bytes: &[u8], packet_type: ServerPacketType) -> Result<Self> {
        use ServerPacketType::*;
        match packet_type {
            LoginAck => {
                let status = Status::deserialize_byte(bytes[0])?;

                let error_message = if status == Status::Failed {
                    Some(String::deserialize(&bytes[1..])?.0)
                } else {
                    None
                };

                Ok(ServerPayload::Login(LoginAckPacket { status, error_message }))
            }
            Healthcheck => {
                let kind = HealthKind::deserialize_byte(bytes[0])?;
                Ok(ServerPayload::Health(HealthCheckPacket { kind }))
            }
            ChannelList => {
                let status = Status::deserialize_byte(bytes[0])?;
                let channels_count = u16::from_be_bytes(bytes[1..3].try_into()?) as usize;

                let mut byte_index = 3;
                let mut channel_ids = Vec::with_capacity(channels_count);
                for _ in 0..channels_count {
                    let channel_id = u64::from_be_bytes(bytes[byte_index..byte_index + 8].try_into()?);
                    channel_ids.push(channel_id);
                    byte_index += 8;
                }

                let error_message = if status == Status::Failed {
                    Some(String::deserialize(&bytes[1..])?.0)
                } else {
                    None
                };

                Ok(ServerPayload::ChannelsList(ChannelsListPacket {
                    status,
                    channel_ids,
                    error_message,
                }))
            }
            Channels => {
                let status = Status::deserialize_byte(bytes[0])?;
                let channels_count = u16::from_be_bytes(bytes[1..3].try_into()?) as usize;

                let mut channels = Vec::with_capacity(channels_count);

                let mut byte_index = 3;
                for _ in 0..channels_count {
                    let (channel, read_bytes) = Channel::deserialize(&bytes[byte_index..])?;
                    channels.push(channel);
                    byte_index += read_bytes;
                }

                let error_message = if status == Status::Failed {
                    Some(String::deserialize(&bytes[1..])?.0)
                } else {
                    None
                };

                Ok(ServerPayload::Channels(GetChannelsResponsePacket {
                    status,
                    channels,
                    error_message,
                }))
            }
            UserStatuses => {
                let status = Status::deserialize_byte(bytes[0])?;
                let user_count = u16::from_be_bytes(bytes[1..3].try_into()?) as usize;

                let mut user_statuses = Vec::with_capacity(user_count);

                let mut byte_index = 3;
                for _ in 0..user_count {
                    let user_id = u64::from_be_bytes(bytes[byte_index..byte_index + 8].try_into()?);
                    byte_index += 8;
                    let (user_status, _) = UserStatus::deserialize(&bytes[byte_index..byte_index + 1])?;
                    byte_index += 1;
                    user_statuses.push((user_id, user_status));
                }

                let error_message = if status == Status::Failed {
                    Some(String::deserialize(&bytes[1..])?.0)
                } else {
                    None
                };

                Ok(ServerPayload::UserStatuses(UserStatusesPacket {
                    status,
                    users: user_statuses,
                    error_message,
                }))
            }
            Users => {
                let status = Status::deserialize_byte(bytes[0])?;
                let user_count = u8::from_be_bytes(bytes[1..2].try_into()?) as usize;

                let mut users = Vec::with_capacity(user_count);

                let mut byte_index = 2;
                for _ in 0..user_count {
                    let (user, read_bytes) = UserData::deserialize(&bytes[byte_index..])?;
                    users.push(user);
                    byte_index += read_bytes;
                }

                let error_message = if status == Status::Failed {
                    Some(String::deserialize(&bytes[1..])?.0)
                } else {
                    None
                };

                Ok(ServerPayload::Users(UsersPacket {
                    status,
                    users,
                    error_message,
                }))
            }
            History => {
                let status = Status::deserialize_byte(bytes[0])?;
                let message_count = u8::from_be_bytes(bytes[1..2].try_into()?) as usize;

                let mut messages = Vec::with_capacity(message_count);

                let mut byte_index = 2;
                for _ in 0..message_count {
                    let (user, read_bytes) = HistoryMessage::deserialize(&bytes[byte_index..])?;
                    messages.push(user);
                    byte_index += read_bytes;
                }

                let error_message = if status == Status::Failed {
                    Some(String::deserialize(&bytes[1..])?.0)
                } else {
                    None
                };

                Ok(ServerPayload::History(HistoryPacket {
                    status,
                    messages,
                    error_message,
                }))
            }
            SendMessageAck => {
                let status = Status::deserialize_byte(bytes[0])?;

                let message_id = u64::from_be_bytes(bytes[1..9].try_into()?);

                let error_message = if status == Status::Failed {
                    Some(String::deserialize(&bytes[1..])?.0)
                } else {
                    None
                };

                Ok(ServerPayload::SendMessageAck(SendMessageAckPacket {
                    status,
                    message_id,
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

impl DeserializeByte for Status {
    fn deserialize_byte(bytes: u8) -> Result<Self> {
        match bytes {
            0x00 => Ok(Status::Success),
            0x01 => Ok(Status::Failed),
            0x02 => Ok(Status::Notification),
            _ => Err(anyhow!("Unknown status byte")),
        }
    }
}

impl Deserialize for String {
    fn deserialize(bytes: &[u8]) -> Result<(Self, usize)> {
        let length = bytes.iter().position(|&b| b == 0).unwrap_or(MAX_MESSAGE_LENGTH);
        let string = String::from_utf8(bytes[0..length].to_vec())?;
        Ok((string, length))
    }
}

#[repr(u8)]
#[derive(Debug, Clone, PartialEq)]
pub enum HealthKind {
    Ping = 0x00,
    Pong = 0x01,
}

impl DeserializeByte for HealthKind {
    fn deserialize_byte(byte: u8) -> Result<(Self)> {
        match byte {
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
    pub error_message: Option<String>,
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
    pub error_message: Option<String>,
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
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct HistoryPacket {
    pub status: Status,
    pub messages: Vec<HistoryMessage>,
    pub error_message: Option<String>,
}
#[derive(Debug, Clone)]
pub struct UserStatusesPacket {
    pub status: Status,
    pub users: Vec<(u64, UserStatus)>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MediaPacket {
    pub status: Status,
    pub filename: String,
    pub media_type: MediaType,
    pub media_data: Vec<u8>,
    pub error_message: Option<String>,
}
