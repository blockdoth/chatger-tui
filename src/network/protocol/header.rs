use anyhow::{Result, anyhow};

use crate::network::protocol::client::{ClientPacketType, Serialize};
use crate::network::protocol::server::{Deserialize, DeserializeByte, ServerPacketType};

#[derive(Debug)]
pub struct Header {
    pub magic_number: [u8; 4],   // 4 bytes "CHTG"
    pub version: PacketVersion,  // 1 byte
    pub packet_type: PacketType, // 1 byte [is_user|1][packet_id|7]
    pub length: u32,             // 4 bytes length of content in bytes
}

impl Header {
    pub fn new(packet_type: PacketType, length: u32) -> Header {
        Header {
            magic_number: [b'C', b'H', b'T', b'G'],
            version: PacketVersion::V1,
            packet_type,
            length,
        }
    }
}

impl Serialize for Header {
    fn serialize(self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(10);
        bytes.extend_from_slice(&self.magic_number);
        bytes.push(self.version.clone() as u8);
        bytes.extend(self.packet_type.serialize());
        bytes.extend_from_slice(&self.length.to_be_bytes());
        bytes
    }
}

impl Deserialize for Header {
    fn deserialize(bytes: &[u8]) -> Result<(Self, usize)> {
        if bytes.len() < 10 {
            return Err(anyhow!("Not enough bytes to deserialize Header"));
        }

        let magic_number = bytes[0..4].try_into()?;

        if magic_number != [b'C', b'H', b'T', b'G'] {
            return Err(anyhow!("Invalid magic number"));
        }
        let version = PacketVersion::deserialize_byte(bytes[4])?;
        let packet_type = PacketType::deserialize_byte(bytes[5])?;
        let length = u32::from_be_bytes(bytes[6..10].try_into()?);

        Ok((
            Header {
                magic_number,
                version,
                packet_type,
                length,
            },
            10,
        ))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PacketType {
    Server(ServerPacketType),
    Client(ClientPacketType),
}

impl DeserializeByte for PacketType {
    fn deserialize_byte(byte: u8) -> Result<Self> {
        // high bit (0x80) indicates Client
        if byte & 0x80 == 0 {
            let packet_type = ServerPacketType::deserialize_byte(byte)?;
            Ok(packet_type.into())
        } else {
            Err(anyhow!("Can not deserialize client packet, how did it get here {byte}"))
        }
    }
}

impl Serialize for PacketType {
    fn serialize(self) -> Vec<u8> {
        use PacketType::*;
        match self {
            Client(packet_type) => vec![packet_type as u8],
            Server(packet_type) => panic!("Client attempted to send server packet of type {packet_type:?}"),
        }
    }
}

impl From<ServerPacketType> for PacketType {
    fn from(packet_type: ServerPacketType) -> Self {
        PacketType::Server(packet_type)
    }
}

impl From<ClientPacketType> for PacketType {
    fn from(packet_type: ClientPacketType) -> Self {
        PacketType::Client(packet_type)
    }
}

#[repr(u8)]
#[derive(Debug, Clone)]
pub enum PacketVersion {
    V1 = 0x01,
}

impl DeserializeByte for PacketVersion {
    fn deserialize_byte(byte: u8) -> Result<Self> {
        match byte {
            0x01 => Ok(PacketVersion::V1),
            other => Err(anyhow!("Unknown PacketVersion value: {:#04x}", other)),
        }
    }
}
