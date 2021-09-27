use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::io;

use crate::codec;
use crate::error::{CommandError, MemcacheError, ServerError};

const OK_STATUS: u16 = 0x0;

#[allow(dead_code)]
pub(crate) enum Opcode {
    Get = 0x00,
    Set = 0x01,
    Add = 0x02,
    Replace = 0x03,
    Delete = 0x04,
    Increment = 0x05,
    Decrement = 0x06,
    Flush = 0x08,
    Stat = 0x10,
    Noop = 0x0a,
    Version = 0x0b,
    GetKQ = 0x0d,
    Append = 0x0e,
    Prepend = 0x0f,
    Touch = 0x1c,
    StartAuth = 0x21,
}

pub(crate) enum Magic {
    Request = 0x80,
    Response = 0x81,
}

#[derive(Debug, Default)]
pub(crate) struct PacketHeader {
    pub(crate) magic: u8,
    pub(crate) opcode: u8,
    pub(crate) key_length: u16,
    pub(crate) extras_length: u8,
    pub(crate) data_type: u8,
    pub(crate) vbucket_id_or_status: u16,
    pub(crate) total_body_length: u32,
    pub(crate) opaque: u32,
    pub(crate) cas: u64,
}

#[derive(Debug)]
pub(crate) struct StoreExtras {
    pub(crate) flags: u32,
    pub(crate) expiration: u32,
}

impl PacketHeader {
    pub(crate) fn write<W: io::Write>(self, writer: &mut W) -> Result<(), io::Error> {
        writer.write_u8(self.magic)?;
        writer.write_u8(self.opcode)?;
        writer.write_u16::<BigEndian>(self.key_length)?;
        writer.write_u8(self.extras_length)?;
        writer.write_u8(self.data_type)?;
        writer.write_u16::<BigEndian>(self.vbucket_id_or_status)?;
        writer.write_u32::<BigEndian>(self.total_body_length)?;
        writer.write_u32::<BigEndian>(self.opaque)?;
        writer.write_u64::<BigEndian>(self.cas)?;
        return Ok(());
    }

    pub(crate) fn read<R: io::Read>(reader: &mut R) -> Result<PacketHeader, MemcacheError> {
        let magic = reader.read_u8()?;
        if magic != Magic::Response as u8 {
            return Err(ServerError::BadMagic(magic).into());
        }
        let header = PacketHeader {
            magic,
            opcode: reader.read_u8()?,
            key_length: reader.read_u16::<BigEndian>()?,
            extras_length: reader.read_u8()?,
            data_type: reader.read_u8()?,
            vbucket_id_or_status: reader.read_u16::<BigEndian>()?,
            total_body_length: reader.read_u32::<BigEndian>()?,
            opaque: reader.read_u32::<BigEndian>()?,
            cas: reader.read_u64::<BigEndian>()?,
        };
        return Ok(header);
    }
}

pub(crate) struct Response {
    header: PacketHeader,
    key: Vec<u8>,
    value: Vec<u8>,
}

impl Response {
    pub(crate) fn err(self) -> Result<Self, MemcacheError> {
        let status = self.header.vbucket_id_or_status;
        if status == OK_STATUS {
            Ok(self)
        } else {
            Err(CommandError::from(status))?
        }
    }
}

pub(crate) fn parse_response<R: io::Read>(reader: &mut R) -> Result<Response, MemcacheError> {
    let header = PacketHeader::read(reader)?;
    let mut extras = vec![0x0; header.extras_length as usize];
    reader.read_exact(extras.as_mut_slice())?;

    let mut key = vec![0x0; header.key_length as usize];
    reader.read_exact(key.as_mut_slice())?;

    // TODO: return error if total_body_length < extras_length + key_length
    let mut value =
        vec![0x0; (header.total_body_length - u32::from(header.key_length) - u32::from(header.extras_length)) as usize];
    reader.read_exact(value.as_mut_slice())?;

    Ok(Response { header, key, value })
}

pub(crate) fn parse_cas_response<R: io::Read>(reader: &mut R) -> Result<bool, MemcacheError> {
    match parse_response(reader)?.err() {
        Err(MemcacheError::CommandError(e)) if e == CommandError::KeyNotFound || e == CommandError::KeyExists => {
            Ok(false)
        }
        Ok(_) => Ok(true),
        Err(e) => Err(e),
    }
}

pub(crate) fn parse_version_response<R: io::Read>(reader: &mut R) -> Result<String, MemcacheError> {
    let Response { value, .. } = parse_response(reader)?.err()?;
    Ok(String::from_utf8(value)?)
}

pub(crate) fn parse_get_response<T: DeserializeOwned, R: io::Read>(reader: &mut R) -> Result<Option<T>, MemcacheError> {
    match parse_response(reader)?.err() {
        Ok(Response { value, .. }) => Ok(Some(codec::decode(value)?)),
        Err(MemcacheError::CommandError(CommandError::KeyNotFound)) => Ok(None),
        Err(e) => Err(e),
    }
}

pub(crate) fn parse_delete_response<R: io::Read>(reader: &mut R) -> Result<bool, MemcacheError> {
    match parse_response(reader)?.err() {
        Ok(_) => Ok(true),
        Err(MemcacheError::CommandError(CommandError::KeyNotFound)) => Ok(false),
        Err(e) => Err(e),
    }
}

pub(crate) fn parse_touch_response<R: io::Read>(reader: &mut R) -> Result<bool, MemcacheError> {
    match parse_response(reader)?.err() {
        Ok(_) => Ok(true),
        Err(MemcacheError::CommandError(CommandError::KeyNotFound)) => Ok(false),
        Err(e) => Err(e),
    }
}

pub(crate) fn parse_stats_response<R: io::Read>(reader: &mut R) -> Result<HashMap<String, String>, MemcacheError> {
    let mut result = HashMap::new();
    loop {
        let Response { key, value, .. } = parse_response(reader)?.err()?;
        let key = String::from_utf8(key)?;
        let value = String::from_utf8(value)?;
        if key.is_empty() && value.is_empty() {
            break;
        }
        let _ = result.insert(key, value);
    }
    Ok(result)
}

pub(crate) fn parse_start_auth_response<R: io::Read>(reader: &mut R) -> Result<bool, MemcacheError> {
    parse_response(reader)?.err().map(|_| true)
}
