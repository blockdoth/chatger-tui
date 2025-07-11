use anyhow::Result;

use crate::network::protocol::server::Deserialize;
pub mod client;
pub mod header;
pub mod server;

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

#[derive(Debug, Clone)]
pub enum Anchor {
    Timestamp(u64), // MSB = 0
    MessageId(u64), // MSB = 1
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
pub struct Channel {
    pub channel_id: u64,
    pub name: String,
    pub icon_id: u64,
}

impl Deserialize for Channel {
    fn deserialize(bytes: &[u8]) -> Result<Self> {
        let channel_id = u64::from_be_bytes(bytes[0..8].try_into()?);

        let name = String::deserialize(&bytes[8..])?;
        let icon_id_start = 8 + name.len() + 1;
        let icon_id = u64::from_be_bytes(bytes[icon_id_start..icon_id_start + 8].try_into()?);

        Ok(Channel {
            channel_id,
            name: todo!(),
            icon_id: todo!(),
        })
    }
}
