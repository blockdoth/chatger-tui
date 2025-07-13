use std::vec;

use anyhow::{Ok, Result, anyhow};

use crate::network::protocol::client::Serialize;
use crate::network::protocol::server::DeserializeByte;
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

impl Serialize for MediaType {
    fn serialize(self) -> Vec<u8> {
        vec![self as u8]
    }
}

impl DeserializeByte for MediaType {
    fn deserialize_byte(byte: u8) -> Result<Self> {
        match byte {
            0x00 => Ok(MediaType::Raw),
            0x01 => Ok(MediaType::Text),
            0x02 => Ok(MediaType::Audio),
            0x03 => Ok(MediaType::Image),
            0x04 => Ok(MediaType::Video),
            other => Err(anyhow!("Unknown MediaType value: {}", other)),
        }
    }
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
impl DeserializeByte for UserStatus {
    fn deserialize_byte(byte: u8) -> Result<Self> {
        match byte {
            0x00 => Ok(UserStatus::Offline),
            0x01 => Ok(UserStatus::Online),
            0x02 => Ok(UserStatus::Idle),
            0x03 => Ok(UserStatus::DoNotDisturb),
            other => Err(anyhow!("Unknown UserStatus value: {}", other)),
        }
    }
}
impl Serialize for UserStatus {
    fn serialize(self) -> Vec<u8> {
        vec![self as u8]
    }
}
