use crate::network::protocol::header::Payload;
use crate::network::protocol::server::{HealthCheckPacket, HealthKind};
use crate::network::protocol::{MediaType, UserStatus};

pub trait Serialize {
    fn serialize(self) -> Vec<u8>;
}

#[repr(u8)]
#[derive(Debug, Clone, PartialEq)]
pub enum ClientPacketType {
    Healthcheck = 0x80,
    Login = 0x81,
    SendMessage = 0x82,
    ChannelsList = 0x84,
    Channels = 0x85,
    History = 0x86,
    UserStatuses = 0x87,
    Users = 0x88,
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
    ChannelsList,
    UserStatuses,
    Users(GetUsersPacket),
    History(GetHistoryPacket),
}

impl Serialize for ClientPayload {
    fn serialize(self) -> Vec<u8> {
        use ClientPayload::*; // Cool trick      
        match self {
            Login(packet) => packet.serialize(),
            Health(packet) => packet.serialize(),
            SendMessage(packet) => packet.serialize(),
            Channels(packet) => packet.serialize(),
            ChannelsList => vec![],
            UserStatuses => vec![],
            Users(packet) => packet.serialize(),
            History(packet) => packet.serialize(),
        }
    }
}

impl From<ClientPayload> for Payload {
    fn from(payload: ClientPayload) -> Self {
        Payload::Client(payload)
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
    pub channel_ids: Vec<u64>,
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
    pub user_ids: Vec<u64>,
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
    pub channel_id: u64,
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
    pub channel_id: u64,
    pub reply_id: u64,
    pub media_ids: Vec<u64>,
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
pub struct SendMediaPacket {
    pub filename: String,
    pub media_type: MediaType,
    pub media_data: Vec<u8>,
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
