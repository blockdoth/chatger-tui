use anyhow::{Ok, Result, anyhow};
use log::info;

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
#[derive(Debug, Clone, PartialEq)]
pub enum UserStatus {
    Offline = 0x00,
    Online = 0x01,
    Idle = 0x02,
    DoNotDisturb = 0x03,
}

//[channel_id1|8][name_len|1][channel_name][icon_id|8]
impl Deserialize for UserStatus {
    fn deserialize(bytes: &[u8]) -> Result<(Self, usize)> {
        match bytes[0] {
            0x00 => Ok((UserStatus::Offline, 1)),
            0x01 => Ok((UserStatus::Online, 1)),
            0x02 => Ok((UserStatus::Idle, 1)),
            0x03 => Ok((UserStatus::DoNotDisturb, 1)),
            other => Err(anyhow!("Unknown UserStatus value: {}", other)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UserData {
    pub user_id: u64,
    pub status: UserStatus,
    pub username: String,
    pub pfp_id: u64,
    pub bio: String,
}

// [user_id1|8][status_id|1][username_length|1][username][pfp_id|8][bio_length|2][bio]
impl Deserialize for UserData {
    fn deserialize(bytes: &[u8]) -> Result<(Self, usize)> {
        let user_id = u64::from_be_bytes(bytes[0..8].try_into()?);
        let (status, _) = UserStatus::deserialize(&bytes[8..9])?;

        let username_length = u8::from_be_bytes(bytes[9..10].try_into()?) as usize;
        let username = String::from_utf8(bytes[10..10 + username_length].to_vec())?;

        let mut byte_index = 10 + username_length;

        let pfp_id = u64::from_be_bytes(bytes[byte_index..byte_index + 8].try_into()?);
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
pub struct HistoryMessage {
    pub message_id: u64,
    pub sent_timestamp: u64,
    pub user_id: u64,
    pub channel_id: u64,
    pub reply_id: u64,
    pub message_text: String,
    pub media_ids: Vec<u64>,
}

// [message_id1|8][sent_timestamp|8][user_id|8][channel_id|8][reply_id|8][message_len|2][message_text][num_media|1][media_id1|8][media_id2|8]...[media_idnum|8]
impl Deserialize for HistoryMessage {
    fn deserialize(bytes: &[u8]) -> Result<(Self, usize)> {
        let message_id = u64::from_be_bytes(bytes[0..8].try_into()?);
        let sent_timestamp = u64::from_be_bytes(bytes[8..16].try_into()?);
        let user_id = u64::from_be_bytes(bytes[16..24].try_into()?);
        let channel_id = u64::from_be_bytes(bytes[24..32].try_into()?);
        let reply_id = u64::from_be_bytes(bytes[32..40].try_into()?);

        let message_len = u16::from_be_bytes(bytes[40..42].try_into()?) as usize;

        let message_text = String::from_utf8(bytes[42..42 + message_len].to_vec())?;
        let mut byte_index = 42 + message_len;

        let num_media = u8::from_be_bytes(bytes[byte_index..byte_index + 1].try_into()?) as usize;
        byte_index += 1;

        let mut media_ids = Vec::with_capacity(num_media);
        for i in 0..num_media {
            let media_id = u64::from_be_bytes(bytes[byte_index..byte_index + 8].try_into()?);
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
pub struct Channel {
    pub channel_id: u64,
    pub name: String,
    pub icon_id: u64,
}

//[channel_id1|8][name_len|1][channel_name][icon_id|8]
impl Deserialize for Channel {
    fn deserialize(bytes: &[u8]) -> Result<(Self, usize)> {
        let channel_id = u64::from_be_bytes(bytes[0..8].try_into()?);
        let name_len: usize = u8::from_be_bytes(bytes[8..9].try_into()?) as usize;
        let name = String::from_utf8(bytes[9..9 + name_len].to_vec())?;
        let icon_id_start = 8 + name_len + 1;
        let icon_id = u64::from_be_bytes(bytes[icon_id_start..icon_id_start + 8].try_into()?);

        Ok((Channel { channel_id, name, icon_id }, icon_id_start + 8))
    }
}
