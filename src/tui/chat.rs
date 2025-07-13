use chrono::{DateTime, Utc};

use crate::network::protocol::server::Channel;
use crate::network::protocol::{MediaType, UserStatus};
use crate::tui::events::ChannelId;

pub struct DisplayChannel {
    pub id: ChannelId,
    pub name: String,
    pub status: ChannelStatus,
}

impl From<Channel> for DisplayChannel {
    fn from(channel: Channel) -> Self {
        DisplayChannel {
            id: channel.channel_id,
            name: channel.name,
            status: ChannelStatus::Read,
        }
    }
}

#[derive(Debug)]
pub struct ChatMessage {
    pub message_id: Option<u64>,
    pub author_name: String,
    pub author_id: u64,
    pub reply_id: u64,
    pub timestamp: DateTime<Utc>,
    pub message: String,
    pub status: ChatMessageStatus,
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub status: UserStatus,
}

#[derive(Debug)]
pub enum ChatMessageStatus {
    Sending,
    Send,
    FailedToSend,
}
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
