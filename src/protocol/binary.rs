use byteorder::{BigEndian, WriteBytesExt};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::io::Write;

use super::ProtocolTrait;
use crate::client::Stats;
use crate::codec;
use crate::error::MemcacheError;
use crate::protocol::binary_packet::{self, Magic, Opcode, PacketHeader};
use crate::stream::Stream;

#[allow(missing_debug_implementations)]
pub struct BinaryProtocol {
    pub stream: Stream,
}

impl ProtocolTrait for BinaryProtocol {
    fn auth(&mut self, username: &str, password: &str) -> Result<(), MemcacheError> {
        let key = "PLAIN";
        let request_header = PacketHeader {
            magic: Magic::Request as u8,
            opcode: Opcode::StartAuth as u8,
            key_length: key.len() as u16,
            total_body_length: (key.len() + username.len() + password.len() + 2) as u32,
            ..Default::default()
        };
        request_header.write(&mut self.stream)?;
        self.stream.write_all(key.as_bytes())?;
        write!(&mut self.stream, "\x00{}\x00{}", username, password)?;
        self.stream.flush()?;
        binary_packet::parse_start_auth_response(&mut self.stream).map(|_| ())
    }

    fn version(&mut self) -> Result<String, MemcacheError> {
        let request_header = PacketHeader {
            magic: Magic::Request as u8,
            opcode: Opcode::Version as u8,
            ..Default::default()
        };
        request_header.write(&mut self.stream)?;
        self.stream.flush()?;
        let version = binary_packet::parse_version_response(&mut self.stream)?;
        return Ok(version);
    }

    fn flush(&mut self) -> Result<(), MemcacheError> {
        let request_header = PacketHeader {
            magic: Magic::Request as u8,
            opcode: Opcode::Flush as u8,
            ..Default::default()
        };
        request_header.write(&mut self.stream)?;
        self.stream.flush()?;
        binary_packet::parse_response(&mut self.stream)?.err().map(|_| ())
    }

    fn flush_with_delay(&mut self, delay: u32) -> Result<(), MemcacheError> {
        let request_header = PacketHeader {
            magic: Magic::Request as u8,
            opcode: Opcode::Flush as u8,
            extras_length: 4,
            total_body_length: 4,
            ..Default::default()
        };
        request_header.write(&mut self.stream)?;
        self.stream.write_u32::<BigEndian>(delay)?;
        self.stream.flush()?;
        binary_packet::parse_response(&mut self.stream)?.err().map(|_| ())
    }

    fn get<T: DeserializeOwned>(&mut self, key: &str) -> Result<Option<T>, MemcacheError> {
        let request_header = PacketHeader {
            magic: Magic::Request as u8,
            opcode: Opcode::Get as u8,
            key_length: key.len() as u16,
            total_body_length: key.len() as u32,
            ..Default::default()
        };
        request_header.write(&mut self.stream)?;
        self.stream.write_all(key.as_bytes())?;
        self.stream.flush()?;
        return binary_packet::parse_get_response(&mut self.stream);
    }

    fn cas<V: Serialize>(&mut self, key: &str, value: V, expiration: u32, cas: u64) -> Result<bool, MemcacheError> {
        self.send_request(Opcode::Set, key, value, expiration, Some(cas))?;
        binary_packet::parse_cas_response(&mut self.stream)
    }

    fn set<V: Serialize>(&mut self, key: &str, value: V, expiration: u32) -> Result<(), MemcacheError> {
        return self.store(Opcode::Set, key, value, expiration, None);
    }

    fn add<V: Serialize>(&mut self, key: &str, value: V, expiration: u32) -> Result<(), MemcacheError> {
        return self.store(Opcode::Add, key, value, expiration, None);
    }

    fn replace<V: Serialize>(&mut self, key: &str, value: V, expiration: u32) -> Result<(), MemcacheError> {
        return self.store(Opcode::Replace, key, value, expiration, None);
    }

    fn delete(&mut self, key: &str) -> Result<bool, MemcacheError> {
        let request_header = PacketHeader {
            magic: Magic::Request as u8,
            opcode: Opcode::Delete as u8,
            key_length: key.len() as u16,
            total_body_length: key.len() as u32,
            ..Default::default()
        };
        request_header.write(&mut self.stream)?;
        self.stream.write_all(key.as_bytes())?;
        self.stream.flush()?;
        return binary_packet::parse_delete_response(&mut self.stream);
    }

    fn touch(&mut self, key: &str, expiration: u32) -> Result<bool, MemcacheError> {
        let request_header = PacketHeader {
            magic: Magic::Request as u8,
            opcode: Opcode::Touch as u8,
            key_length: key.len() as u16,
            extras_length: 4,
            total_body_length: (key.len() as u32 + 4),
            ..Default::default()
        };
        request_header.write(&mut self.stream)?;
        self.stream.write_u32::<BigEndian>(expiration)?;
        self.stream.write_all(key.as_bytes())?;
        self.stream.flush()?;
        return binary_packet::parse_touch_response(&mut self.stream);
    }

    fn stats(&mut self) -> Result<Stats, MemcacheError> {
        let request_header = PacketHeader {
            magic: Magic::Request as u8,
            opcode: Opcode::Stat as u8,
            ..Default::default()
        };
        request_header.write(&mut self.stream)?;
        self.stream.flush()?;
        let stats_info = binary_packet::parse_stats_response(&mut self.stream)?;
        return Ok(stats_info);
    }
}

impl BinaryProtocol {
    fn send_request<V: Serialize>(
        &mut self,
        opcode: Opcode,
        key: &str,
        value: V,
        expiration: u32,
        cas: Option<u64>,
    ) -> Result<(), MemcacheError> {
        let encoded = codec::encode(&value)?;

        let request_header = PacketHeader {
            magic: Magic::Request as u8,
            opcode: opcode as u8,
            key_length: key.len() as u16,
            extras_length: 8,
            total_body_length: (8 + key.len() + encoded.len()) as u32,
            cas: cas.unwrap_or(0),
            ..Default::default()
        };
        let extras = binary_packet::StoreExtras { flags: 0, expiration };
        request_header.write(&mut self.stream)?;
        self.stream.write_u32::<BigEndian>(extras.flags)?;
        self.stream.write_u32::<BigEndian>(extras.expiration)?;
        self.stream.write_all(key.as_bytes())?;
        self.stream.write_all(&encoded)?;
        self.stream.flush().map_err(Into::into)
    }

    fn store<V: Serialize>(
        &mut self,
        opcode: Opcode,
        key: &str,
        value: V,
        expiration: u32,
        cas: Option<u64>,
    ) -> Result<(), MemcacheError> {
        self.send_request(opcode, key, value, expiration, cas)?;
        binary_packet::parse_response(&mut self.stream)?.err().map(|_| ())
    }
}
