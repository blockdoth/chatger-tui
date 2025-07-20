use chrono::{DateTime, Utc};

use crate::network::protocol::server::Channel;
use crate::network::protocol::{MediaType, UserStatus};
use crate::tui::events::{ChannelId, MessageId, UserId};

#[derive(Clone, Debug)]
pub struct DisplayChannel {
    pub id: ChannelId,
    pub name: String,
    pub status: ChannelStatus,
    pub selection_offset: usize,
}

impl From<Channel> for DisplayChannel {
    fn from(channel: Channel) -> Self {
        DisplayChannel {
            id: channel.channel_id,
            name: channel.name,
            status: ChannelStatus::Read,
            selection_offset: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChatMessage {
    pub message_id: MessageId,
    pub reply_id: MessageId,
    pub author_name: String,
    pub author_id: UserId,
    pub timestamp: DateTime<Utc>,
    pub message: String,
    pub status: ChatMessageStatus,
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: UserId,
    pub name: String,
    pub status: UserStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChatMessageStatus {
    Sending,
    Send,
    FailedToSend,
}

#[derive(Clone, Debug)]
pub enum ChannelStatus {
    Read,
    Unread,
    Muted,
}

#[derive(Debug)]
pub struct MediaMessage {
    pub filename: String,
    pub media_type: MediaType,
    pub media_data: Vec<u8>,
}
