use chrono::{DateTime, NaiveDateTime, Utc};

use crate::network::protocol::Channel;

pub type ChannelId = u64;

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
pub struct ChatMessage {
    pub id: u64,
    pub author_name: String,
    pub author_id: u64,
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

pub struct CurrentUser {
    pub id: u64,
    pub name: String,
}

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
#[derive(PartialEq, Debug, Clone)]
pub enum UserStatus {
    Online,
    Offline,
}
