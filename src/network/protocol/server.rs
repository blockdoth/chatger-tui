use anyhow::{Result, anyhow};
use log::{debug, error, info};

use crate::network::client::MAX_MESSAGE_LENGTH;
use crate::network::protocol::{MediaType, UserStatus};
use crate::tui::events::{ChannelId, IconId, MediaId, MessageId, ProfilePicId, UserId};

pub trait Deserialize: Sized {
    fn deserialize(bytes: &[u8]) -> Result<(Self, usize)>;
}
pub trait DeserializeByte: Sized {
    fn deserialize_byte(byte: u8) -> Result<Self>;
}

#[repr(u8)]
#[derive(Debug, Clone, PartialEq)]
pub enum ServerPacketType {
    Healthcheck = 0x00,
    LoginAck = 0x01,
    SendMessageAck = 0x02,
    SendMediaAck = 0x03,
    ChannelList = 0x04,
    Channels = 0x05,
    History = 0x06,
    UserStatuses = 0x07,
    Users = 0x08,
    Media = 0x09,
    Typing = 0x0A,
    UserStatus = 0x0B,
}

impl DeserializeByte for ServerPacketType {
    fn deserialize_byte(byte: u8) -> Result<Self> {
        use ServerPacketType::*;
        match byte {
            0x00 => Ok(Healthcheck),
            0x01 => Ok(LoginAck),
            0x02 => Ok(SendMessageAck),
            0x03 => Ok(SendMediaAck),
            0x04 => Ok(ChannelList),
            0x05 => Ok(Channels),
            0x06 => Ok(History),
            0x07 => Ok(UserStatuses),
            0x08 => Ok(Users),
            0x09 => Ok(Media),
            0x0A => Ok(Typing),
            0x0B => Ok(UserStatus),
            other => Err(anyhow!("Unknown ServerPacketType: {}", other)),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ServerPayload {
    Health(HealthCheckPacket),
    Login(LoginAckPacket),
    SendMessageAck(SendMessageAckPacket),
    SendMediaAck(SendMediaAckPacket),
    Channels(GetChannelsResponsePacket),
    ChannelsList(ChannelsListPacket),
    UserStatuses(UserStatusesPacket),
    Users(UsersPacket),
    History(HistoryPacket),
    Media(MediaPacket),
    Typing(UserTypingPacket),
    Status(UserStatusPacket),
}

fn deserialize_error(bytes: &[u8], status: &ReturnStatus) -> Result<(Option<String>, usize)> {
    if *status == ReturnStatus::Failed {
        let (msg, len) = String::deserialize(bytes)?;
        Ok((Some(msg), len))
    } else {
        Ok((None, 0))
    }
}

macro_rules! deserialize_variant {
    ($bytes:ident, $variant:path, $packet:ty) => {{
        let (packet, len) = <$packet>::deserialize($bytes)?;
        Ok(($variant(packet), len))
    }};
}

impl ServerPayload {
    pub fn deserialize_packet(bytes: &[u8], packet_type: ServerPacketType) -> Result<(Self, usize)> {
        use ServerPacketType::*;
        match packet_type {
            LoginAck => deserialize_variant!(bytes, ServerPayload::Login, LoginAckPacket),
            Healthcheck => deserialize_variant!(bytes, ServerPayload::Health, HealthCheckPacket),
            SendMessageAck => deserialize_variant!(bytes, ServerPayload::SendMessageAck, SendMessageAckPacket),
            SendMediaAck => deserialize_variant!(bytes, ServerPayload::SendMediaAck, SendMediaAckPacket),
            ChannelList => deserialize_variant!(bytes, ServerPayload::ChannelsList, ChannelsListPacket),
            Channels => deserialize_variant!(bytes, ServerPayload::Channels, GetChannelsResponsePacket),
            History => deserialize_variant!(bytes, ServerPayload::History, HistoryPacket),
            UserStatuses => deserialize_variant!(bytes, ServerPayload::UserStatuses, UserStatusesPacket),
            Users => deserialize_variant!(bytes, ServerPayload::Users, UsersPacket),
            Media => deserialize_variant!(bytes, ServerPayload::Media, MediaPacket),
            Typing => deserialize_variant!(bytes, ServerPayload::Typing, UserTypingPacket),
            UserStatus => deserialize_variant!(bytes, ServerPayload::Status, UserStatusPacket),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, PartialEq)]
pub enum ReturnStatus {
    Success = 0x00,
    Failed = 0x01,
    Notification = 0x02, // Only used for HISTORY
}

impl DeserializeByte for ReturnStatus {
    fn deserialize_byte(bytes: u8) -> Result<Self> {
        match bytes {
            0x00 => Ok(ReturnStatus::Success),
            0x01 => Ok(ReturnStatus::Failed),
            0x02 => Ok(ReturnStatus::Notification),
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
    fn deserialize_byte(byte: u8) -> Result<Self> {
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

impl Deserialize for HealthCheckPacket {
    fn deserialize(bytes: &[u8]) -> Result<(Self, usize)> {
        let kind = HealthKind::deserialize_byte(bytes[0])?;
        Ok((HealthCheckPacket { kind }, 1))
    }
}

#[derive(Debug, Clone)]
pub struct LoginAckPacket {
    pub status: ReturnStatus,
    pub error_message: Option<String>,
}

impl Deserialize for LoginAckPacket {
    fn deserialize(bytes: &[u8]) -> Result<(Self, usize)> {
        let status = ReturnStatus::deserialize_byte(bytes[0])?;
        let mut byte_index = 1;
        let (error_message, error_len) = deserialize_error(&bytes[byte_index..], &status)?;
        byte_index += error_len;
        Ok((LoginAckPacket { status, error_message }, byte_index))
    }
}

#[derive(Debug, Clone)]
pub struct SendMessageAckPacket {
    pub status: ReturnStatus,
    pub message_id: MessageId,
    pub error_message: Option<String>,
}

impl Deserialize for SendMessageAckPacket {
    fn deserialize(bytes: &[u8]) -> Result<(Self, usize)> {
        let status = ReturnStatus::deserialize_byte(bytes[0])?;
        let mut byte_index = 1;

        let message_id = MessageId::from_be_bytes(bytes[byte_index..byte_index + 8].try_into()?);
        byte_index += 8;

        let (error_message, error_len) = deserialize_error(&bytes[byte_index..], &status)?;
        byte_index += error_len;
        Ok((
            SendMessageAckPacket {
                status,
                message_id,
                error_message,
            },
            byte_index,
        ))
    }
}

#[derive(Debug, Clone)]
pub struct SendMediaAckPacket {
    pub status: ReturnStatus,
    pub media_id: MessageId,
    pub error_message: Option<String>,
}

impl Deserialize for SendMediaAckPacket {
    fn deserialize(bytes: &[u8]) -> Result<(Self, usize)> {
        let status = ReturnStatus::deserialize_byte(bytes[0])?;
        let mut byte_index = 1;

        let media_id = MessageId::from_be_bytes(bytes[byte_index..byte_index + 8].try_into()?);
        byte_index += 8;

        let (error_message, error_len) = deserialize_error(&bytes[byte_index..], &status)?;
        byte_index += error_len;
        Ok((
            SendMediaAckPacket {
                status,
                media_id,
                error_message,
            },
            byte_index,
        ))
    }
}

#[derive(Debug, Clone)]
pub struct ChannelsListPacket {
    pub status: ReturnStatus,
    pub channel_ids: Vec<ChannelId>,
    pub error_message: Option<String>,
}

impl Deserialize for ChannelsListPacket {
    fn deserialize(bytes: &[u8]) -> Result<(Self, usize)> {
        let status = ReturnStatus::deserialize_byte(bytes[0])?;

        let channels_count = u16::from_be_bytes(bytes[1..3].try_into()?) as usize;
        let mut channel_ids = Vec::with_capacity(channels_count);

        let mut byte_index = 3;
        for _ in 0..channels_count {
            let channel_id = ChannelId::from_be_bytes(bytes[byte_index..byte_index + 8].try_into()?);
            channel_ids.push(channel_id);
            byte_index += 8;
        }

        let (error_message, error_len) = deserialize_error(&bytes[byte_index..], &status)?;
        byte_index += error_len;
        Ok((
            ChannelsListPacket {
                status,
                channel_ids,
                error_message,
            },
            byte_index,
        ))
    }
}

#[derive(Debug, Clone)]
pub struct GetChannelsResponsePacket {
    pub status: ReturnStatus,
    pub channels: Vec<Channel>,
    pub error_message: Option<String>,
}

impl Deserialize for GetChannelsResponsePacket {
    fn deserialize(bytes: &[u8]) -> Result<(Self, usize)> {
        let status = ReturnStatus::deserialize_byte(bytes[0])?;

        let channel_count = u16::from_be_bytes(bytes[1..3].try_into()?) as usize;
        let mut channels = Vec::with_capacity(channel_count);

        let mut byte_index = 3;
        for _ in 0..channel_count {
            let (channel, read_bytes) = Channel::deserialize(&bytes[byte_index..])?;
            channels.push(channel);
            byte_index += read_bytes;
        }

        let (error_message, error_len) = deserialize_error(&bytes[byte_index..], &status)?;
        byte_index += error_len;
        Ok((
            GetChannelsResponsePacket {
                status,
                channels,
                error_message,
            },
            byte_index,
        ))
    }
}

#[derive(Debug, Clone)]
pub struct Channel {
    pub channel_id: ChannelId,
    pub name: String,
    pub icon_id: IconId,
}

//[channel_id1|8][name_len|1][channel_name][icon_id|8]
impl Deserialize for Channel {
    fn deserialize(bytes: &[u8]) -> Result<(Self, usize)> {
        let channel_id = ChannelId::from_be_bytes(bytes[0..8].try_into()?);
        let name_len = u8::from_be_bytes(bytes[8..9].try_into()?) as usize;
        let name = String::from_utf8(bytes[9..9 + name_len].to_vec())?;
        let icon_id_start = 8 + name_len + 1;
        let icon_id = IconId::from_be_bytes(bytes[icon_id_start..icon_id_start + 8].try_into()?);

        Ok((Channel { channel_id, name, icon_id }, icon_id_start + 8))
    }
}

#[derive(Debug, Clone)]
pub struct UsersPacket {
    pub status: ReturnStatus,
    pub users: Vec<UserData>,
    pub error_message: Option<String>,
}

impl Deserialize for UsersPacket {
    fn deserialize(bytes: &[u8]) -> Result<(Self, usize)> {
        let status = ReturnStatus::deserialize_byte(bytes[0])?;
        let mut byte_index = 1;

        let user_count = u8::from_be_bytes(bytes[byte_index..byte_index + 1].try_into()?) as usize;
        byte_index += 1;

        let mut users = Vec::with_capacity(user_count);

        for _ in 0..user_count {
            let (user, read_bytes) = UserData::deserialize(&bytes[byte_index..])?;
            users.push(user);
            byte_index += read_bytes;
        }

        let (error_message, error_len) = deserialize_error(&bytes[byte_index..], &status)?;
        byte_index += error_len;
        Ok((
            UsersPacket {
                status,
                users,
                error_message,
            },
            byte_index,
        ))
    }
}

#[derive(Debug, Clone)]
pub struct UserData {
    pub user_id: UserId,
    pub status: UserStatus,
    pub username: String,
    pub pfp_id: ProfilePicId,
    pub bio: String,
}

// [user_id1|8][status_id|1][username_length|1][username][pfp_id|8][bio_length|2][bio]
impl Deserialize for UserData {
    fn deserialize(bytes: &[u8]) -> Result<(Self, usize)> {
        let user_id = UserId::from_be_bytes(bytes[0..8].try_into()?);
        let mut byte_index = 8;
        let status = UserStatus::deserialize_byte(bytes[byte_index])?;
        byte_index += 1;

        let username_length = u8::from_be_bytes(bytes[byte_index..byte_index + 1].try_into()?) as usize;
        byte_index += 1;

        let username = String::from_utf8(bytes[byte_index..byte_index + username_length].to_vec())?;
        byte_index += username_length;

        let pfp_id = ProfilePicId::from_be_bytes(bytes[byte_index..byte_index + 8].try_into()?);
        byte_index += 8;

        let bio_length = u16::from_be_bytes(bytes[byte_index..byte_index + 2].try_into()?) as usize;
        byte_index += 2;

        let bio = String::from_utf8(bytes[byte_index..byte_index + bio_length].to_vec())?;
        byte_index += bio_length;

        Ok((
            UserData {
                user_id,
                status,
                username,
                pfp_id,
                bio,
            },
            byte_index,
        ))
    }
}

#[derive(Debug, Clone)]
pub struct HistoryPacket {
    pub status: ReturnStatus,
    pub messages: Vec<HistoryMessage>,
    pub error_message: Option<String>,
}

impl Deserialize for HistoryPacket {
    fn deserialize(bytes: &[u8]) -> Result<(Self, usize)> {
        let status = ReturnStatus::deserialize_byte(bytes[0])?;

        let message_count = u8::from_be_bytes(bytes[1..2].try_into()?) as usize;
        let mut messages = Vec::with_capacity(message_count);

        let mut byte_index = 2;
        for _ in 0..message_count {
            let (user, read_bytes) = HistoryMessage::deserialize(&bytes[byte_index..])?;
            messages.push(user);
            byte_index += read_bytes;
        }
        let (error_message, error_len) = deserialize_error(&bytes[byte_index..], &status)?;
        byte_index += error_len;
        Ok((
            HistoryPacket {
                status,
                messages,
                error_message,
            },
            byte_index,
        ))
    }
}

#[derive(Debug, Clone)]
pub struct HistoryMessage {
    pub message_id: MessageId,
    pub sent_timestamp: u64,
    pub user_id: UserId,
    pub channel_id: ChannelId,
    pub reply_id: MessageId,
    pub message_text: String,
    pub media_ids: Vec<MediaId>,
}

// [message_id1|8][sent_timestamp|8][user_id|8][channel_id|8][reply_id|8][message_len|2][message_text][num_media|1][media_id1|8][media_id2|8]...[media_idnum|8]
impl Deserialize for HistoryMessage {
    fn deserialize(bytes: &[u8]) -> Result<(Self, usize)> {
        let message_id = MessageId::from_be_bytes(bytes[0..8].try_into()?);
        let sent_timestamp = u64::from_be_bytes(bytes[8..16].try_into()?);
        let user_id = UserId::from_be_bytes(bytes[16..24].try_into()?);
        let channel_id = ChannelId::from_be_bytes(bytes[24..32].try_into()?);
        let reply_id = MessageId::from_be_bytes(bytes[32..40].try_into()?);

        let message_len = u16::from_be_bytes(bytes[40..42].try_into()?) as usize;
        let message_text = String::from_utf8(bytes[42..42 + message_len].to_vec())?;
        let mut byte_index = 42 + message_len;

        let num_media = u8::from_be_bytes(bytes[byte_index..byte_index + 1].try_into()?) as usize;
        byte_index += 1;

        let mut media_ids = Vec::with_capacity(num_media);
        for i in 0..num_media {
            let media_id = MediaId::from_be_bytes(bytes[byte_index..byte_index + 8].try_into()?);
            byte_index += 8;
            media_ids.push(media_id);
        }

        Ok((
            HistoryMessage {
                message_id,
                sent_timestamp,
                user_id,
                channel_id,
                reply_id,
                message_text,
                media_ids,
            },
            byte_index,
        ))
    }
}

#[derive(Debug, Clone)]
pub struct UserStatusesPacket {
    pub status: ReturnStatus,
    pub users: Vec<(UserId, UserStatus)>,
    pub error_message: Option<String>,
}

impl Deserialize for UserStatusesPacket {
    fn deserialize(bytes: &[u8]) -> Result<(Self, usize)> {
        let status = ReturnStatus::deserialize_byte(bytes[0])?;

        let user_count = u16::from_be_bytes(bytes[1..3].try_into()?) as usize;
        let mut users = Vec::with_capacity(user_count);

        let mut byte_index = 3;
        for i in 0..user_count {
            // info!("decode {i}");
            // info!("{:?}",&bytes[byte_index..80]);
            // info!("{:?}", users);

            let user_id = UserId::from_be_bytes(bytes[byte_index..byte_index + 8].try_into()?);
            byte_index += 8;
            let user_status = UserStatus::deserialize_byte(bytes[byte_index])?;
            byte_index += 1;
            users.push((user_id, user_status));
        }
        let (error_message, error_len) = deserialize_error(&bytes[byte_index..], &status)?;
        byte_index += error_len;
        Ok((
            UserStatusesPacket {
                status,
                users,
                error_message,
            },
            byte_index,
        ))
    }
}

#[derive(Debug, Clone)]
pub struct MediaPacket {
    pub status: ReturnStatus,
    pub filename: String,
    pub media_type: MediaType,
    pub media_data: Vec<u8>,
    pub error_message: Option<String>,
}

impl Deserialize for MediaPacket {
    fn deserialize(bytes: &[u8]) -> Result<(Self, usize)> {
        let status = ReturnStatus::deserialize_byte(bytes[0])?;

        let filename_length = u8::from_be_bytes(bytes[1..2].try_into()?) as usize;
        let mut byte_index = 2;

        let filename = String::from_utf8(bytes[byte_index..byte_index + filename_length].to_vec())?;
        byte_index += filename_length;

        let media_type = MediaType::deserialize_byte(bytes[byte_index])?;
        byte_index += 1;

        let media_length = u32::from_be_bytes(bytes[byte_index..byte_index + 4].try_into()?) as usize;
        byte_index += 4;

        let media_data = bytes[byte_index..byte_index + media_length].to_vec();
        byte_index += media_length;

        let (error_message, error_len) = deserialize_error(&bytes[byte_index..], &status)?;
        byte_index += error_len;

        Ok((
            MediaPacket {
                status,
                filename,
                media_type,
                media_data,
                error_message,
            },
            byte_index,
        ))
    }
}

#[derive(Debug, Clone)]
pub struct UserTypingPacket {
    pub is_typing: bool,
    pub user_id: UserId,
    pub channel_id: ChannelId,
}

impl Deserialize for UserTypingPacket {
    fn deserialize(bytes: &[u8]) -> Result<(Self, usize)> {
        let is_typing = match bytes[0] {
            0x00 => false,
            0x01 => true,
            b => return Err(anyhow!("Failed to deserialize is_typing field {b}")),
        };
        let mut byte_index = 1;

        let user_id = UserId::from_be_bytes(bytes[byte_index..byte_index + 8].try_into()?);
        byte_index += 8;
        let channel_id = ChannelId::from_be_bytes(bytes[byte_index..byte_index + 8].try_into()?);
        byte_index += 8;

        Ok((
            UserTypingPacket {
                is_typing,
                user_id,
                channel_id,
            },
            byte_index,
        ))
    }
}

#[derive(Debug, Clone)]
pub struct UserStatusPacket {
    pub status: UserStatus,
    pub user_id: UserId,
}

impl Deserialize for UserStatusPacket {
    fn deserialize(bytes: &[u8]) -> Result<(Self, usize)> {
        let status = UserStatus::deserialize_byte(bytes[0])?;
        let mut byte_index = 1;
        let user_id = UserId::from_be_bytes(bytes[byte_index..byte_index + 8].try_into()?);
        byte_index += 8;

        Ok((UserStatusPacket { status, user_id }, byte_index))
    }
}
