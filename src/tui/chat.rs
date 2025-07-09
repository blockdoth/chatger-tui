use chrono::{DateTime, NaiveDateTime, Utc};

pub type ChannelId = u64;

pub struct Channel {
    pub id: ChannelId,
    pub name: String,
    pub status: ChannelStatus,
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
