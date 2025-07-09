use chrono::{DateTime, NaiveDateTime, Utc};

pub type ChannelId = u64;

pub struct Channel {
    pub id: ChannelId,
    pub name: String,
    pub status: ChannelStatus,
}

pub struct ChatMessage {
    pub id: u64,
    pub author: User,
    pub timestamp: DateTime<Utc>,
    pub message: String,
}

pub struct User {
    pub id: u64,
    pub name: String,
    pub status: UserStatus,
}

pub enum ChannelStatus {
    Read,
    Unread,
    Muted,
}
#[derive(PartialEq)]
pub enum UserStatus {
    Online,
    Offline,
}
