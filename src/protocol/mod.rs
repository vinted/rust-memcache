use serde::de::DeserializeOwned;
use serde::Serialize;

mod ascii;
mod binary;
mod binary_packet;

use crate::client::Stats;
use crate::error::MemcacheError;
pub(crate) use crate::protocol::ascii::AsciiProtocol;
pub(crate) use crate::protocol::binary::BinaryProtocol;
use crate::stream::Stream;
use enum_dispatch::enum_dispatch;

#[allow(missing_debug_implementations)]
#[enum_dispatch]
pub enum Protocol {
    Ascii(AsciiProtocol<Stream>),
    Binary(BinaryProtocol),
}

#[enum_dispatch(Protocol)]
pub(crate) trait ProtocolTrait {
    fn auth(&mut self, username: &str, password: &str) -> Result<(), MemcacheError>;
    fn version(&mut self) -> Result<String, MemcacheError>;
    fn flush(&mut self) -> Result<(), MemcacheError>;
    fn flush_with_delay(&mut self, delay: u32) -> Result<(), MemcacheError>;
    fn get<T: DeserializeOwned>(&mut self, key: &str) -> Result<Option<T>, MemcacheError>;
    fn set<V: Serialize>(&mut self, key: &str, value: V, expiration: u32) -> Result<(), MemcacheError>;
    fn cas<V: Serialize>(&mut self, key: &str, value: V, expiration: u32, cas: u64) -> Result<bool, MemcacheError>;
    fn add<V: Serialize>(&mut self, key: &str, value: V, expiration: u32) -> Result<(), MemcacheError>;
    fn replace<V: Serialize>(&mut self, key: &str, value: V, expiration: u32) -> Result<(), MemcacheError>;
    fn delete(&mut self, key: &str) -> Result<bool, MemcacheError>;
    fn touch(&mut self, key: &str, expiration: u32) -> Result<bool, MemcacheError>;
    fn stats(&mut self) -> Result<Stats, MemcacheError>;
}
