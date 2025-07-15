use crate::network::protocol::server::{HealthCheckPacket, HealthKind};
use crate::network::protocol::{MediaType, UserStatus};
use crate::tui::events::{ChannelId, MediaId, MessageId, UserId};

pub trait Serialize {
    fn serialize(self) -> Vec<u8>;
}

#[repr(u8)]
#[derive(Debug, Clone, PartialEq)]
pub enum ClientPacketType {
    Healthcheck = 0x80,
    Login = 0x81,
    SendMessage = 0x82,
    SendMedia = 0x83,
    ChannelsList = 0x84,
    Channels = 0x85,
    History = 0x86,
    UserStatuses = 0x87,
    Users = 0x88,
    Media = 0x89,
    Typing = 0x8A,
    Status = 0x8B,
}

impl Serialize for ClientPacketType {
    fn serialize(self) -> Vec<u8> {
        vec![self as u8]
    }
}

#[derive(Debug, Clone)]
pub enum ClientPayload {
    Login(LoginPacket),
    Health(HealthCheckPacket),
    Channels(GetChannelsPacket),
    SendMessage(SendMessagePacket),
    SendMedia(SendMediaPacket),
    ChannelsList,
    UserStatuses,
    Users(GetUsersPacket),
    History(GetHistoryPacket),
    Media(GetMediaPacket),
    Typing(TypingPacket),
    Status(StatusPacket),
}

impl Serialize for ClientPayload {
    fn serialize(self) -> Vec<u8> {
        use ClientPayload::*; // Cool trick      
        match self {
            Login(packet) => packet.serialize(),
            Health(packet) => packet.serialize(),
            SendMessage(packet) => packet.serialize(),
            SendMedia(packet) => packet.serialize(),
            Channels(packet) => packet.serialize(),
            ChannelsList => vec![],
            UserStatuses => vec![],
            Users(packet) => packet.serialize(),
            History(packet) => packet.serialize(),
            Media(packet) => packet.serialize(),
            Typing(packet) => packet.serialize(),
            Status(packet) => packet.serialize(),
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
pub struct GetChannelsPacket {
    pub channel_ids: Vec<ChannelId>,
}

impl Serialize for GetChannelsPacket {
    fn serialize(self) -> Vec<u8> {
        let channel_count = self.channel_ids.len();
        let mut bytes = Vec::with_capacity(channel_count * 8 + 2);
        bytes.extend((channel_count as u16).to_be_bytes());
        for channel_id in self.channel_ids {
            bytes.extend_from_slice(&channel_id.to_be_bytes());
        }
        bytes
    }
}

#[derive(Debug, Clone)]
pub struct GetUsersPacket {
    pub user_ids: Vec<UserId>,
}

impl Serialize for GetUsersPacket {
    fn serialize(self) -> Vec<u8> {
        let user_count = self.user_ids.len();
        let mut bytes = Vec::with_capacity(user_count * 8 + 2);
        bytes.extend((user_count as u8).to_be_bytes());
        for user_id in self.user_ids {
            bytes.extend_from_slice(&user_id.to_be_bytes());
        }
        bytes
    }
}

#[derive(Debug, Clone)]
pub enum Anchor {
    Timestamp(u64), // MSB = 0
    MessageId(u64), // MSB = 1
}

impl Serialize for Anchor {
    fn serialize(self) -> Vec<u8> {
        match self {
            Anchor::Timestamp(anchor) => anchor.to_be_bytes().to_vec(),
            Anchor::MessageId(anchor) => anchor.to_be_bytes().to_vec(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GetHistoryPacket {
    pub channel_id: ChannelId,
    pub anchor: Anchor,
    pub num_messages_back: i8,
}

// [length|4]: 17
// [packet content]: [channel_id|8][ anchor | 8 ][num_messages_back|1]
//       [ anchor ]: [ [is_message_id|1bit] [message_id/unix_timestamp|63bit] | 8 ]
//  num_messages_back is a 2s-complimment signed 8bit value (-128 to 127), positive values will request messages backwards in time while negative values forward
//  is_message_id 0x0: interpret anchor as unix_timestamp (with 0 as MSB) to use as history origin
//  is_message_id 0x1: interpret anchor as message_id     (with 0 as MSB) to use as history origin
impl Serialize for GetHistoryPacket {
    fn serialize(self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(17);
        bytes.extend(self.channel_id.to_be_bytes());
        bytes.extend(self.anchor.serialize());
        bytes.extend(self.num_messages_back.to_be_bytes());
        bytes
    }
}

#[derive(Debug, Clone)]
pub struct SendMessagePacket {
    pub channel_id: ChannelId,
    pub reply_id: MessageId,
    pub media_ids: Vec<MediaId>,
    pub message_text: String,
}

// [packet content]: [channel_id|8][reply_id|8][num_media|1][media_id1|8][media_id2|8]...[media_idnum|8][message_text]
impl Serialize for SendMessagePacket {
    fn serialize(self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend(self.channel_id.to_be_bytes());
        bytes.extend(self.reply_id.to_be_bytes());
        bytes.push(self.media_ids.len() as u8);

        for media_id in &self.media_ids {
            bytes.extend(media_id.to_be_bytes());
        }

        bytes.extend(self.message_text.as_bytes());
        bytes
    }
}

#[derive(Debug, Clone)]
pub struct GetMediaPacket {
    pub media_id: MediaId,
}

impl Serialize for GetMediaPacket {
    fn serialize(self) -> Vec<u8> {
        self.media_id.to_be_bytes().to_vec()
    }
}

#[derive(Debug, Clone)]
pub struct SendMediaPacket {
    pub filename: String,
    pub media_type: MediaType,
    pub media_data: Vec<u8>,
}

impl Serialize for SendMediaPacket {
    fn serialize(self) -> Vec<u8> {
        let filename_bytes = self.filename.as_bytes();
        let filename_len = filename_bytes.len();
        let media_data_len = self.media_data.len();

        let mut bytes = Vec::with_capacity(4 + filename_len + 1 + 4 + media_data_len);

        bytes.extend_from_slice(&(filename_len as u32).to_be_bytes());
        bytes.extend_from_slice(filename_bytes);
        bytes.extend_from_slice(&self.media_type.serialize());
        bytes.extend_from_slice(&(media_data_len as u32).to_be_bytes());
        bytes.extend_from_slice(&self.media_data);
        bytes
    }
}

#[derive(Debug, Clone)]
pub struct TypingPacket {
    pub is_typing: bool,
    pub channel_id: ChannelId,
}

impl Serialize for TypingPacket {
    fn serialize(self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(9);
        bytes.push(self.is_typing as u8);
        bytes.extend_from_slice(&self.channel_id.to_be_bytes());
        bytes
    }
}

#[derive(Debug, Clone)]
pub struct StatusPacket {
    pub status: UserStatus,
}

impl Serialize for StatusPacket {
    fn serialize(self) -> Vec<u8> {
        self.status.serialize()
    }
}
