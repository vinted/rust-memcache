use r2d2::Pool;
use r2d2::PooledConnection;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;

use crate::connection::ConnectionManager;
use crate::error::{ClientError, MemcacheError};
use crate::protocol::ProtocolTrait;

/// Stats of memcache
pub type Stats = HashMap<String, String>;

/// Memcache client wrapping r2d2 pooled memcache connection
#[derive(Clone, Debug)]
pub struct Client(Pool<ConnectionManager>);

pub(crate) fn check_key_len(key: &str) -> Result<(), MemcacheError> {
    if key.len() > 250 {
        Err(ClientError::KeyTooLong)?
    }
    Ok(())
}

impl Client {
    /// Initialize client with given pool
    pub fn with_pool(pool: Pool<ConnectionManager>) -> Self {
        Self(pool)
    }

    /// Get pool connection
    pub fn get_connection(&self) -> Result<PooledConnection<ConnectionManager>, MemcacheError> {
        Ok(self.0.get()?)
    }

    /// Get ConnectionManager pool
    pub fn get_pool(&self) -> Pool<ConnectionManager> {
        self.0.clone()
    }

    /// Get the memcached server version.
    ///
    /// Example:
    ///
    /// ```rust
    /// let pool = memcache::Pool::builder()
    /// .connection_timeout(std::time::Duration::from_secs(1))
    /// .build(memcache::ConnectionManager::new("memcache://localhost:12345").unwrap())
    /// .unwrap();
    /// let client = memcache::Client::with_pool(pool);
    /// client.version().unwrap();
    /// ```
    pub fn version(&self) -> Result<String, MemcacheError> {
        self.get_connection()?.version()
    }

    /// Flush all cache on memcached server immediately.
    ///
    /// Example:
    ///
    /// ```rust
    /// let pool = memcache::Pool::builder()
    /// .connection_timeout(std::time::Duration::from_secs(1))
    /// .build(memcache::ConnectionManager::new("memcache://localhost:12345").unwrap())
    /// .unwrap();
    /// let client = memcache::Client::with_pool(pool);
    /// client.flush().unwrap();
    /// ```
    pub fn flush(&self) -> Result<(), MemcacheError> {
        self.get_connection()?.flush()
    }

    /// Flush all cache on memcached server with a delay seconds.
    ///
    /// Example:
    ///
    /// ```rust
    /// let pool = memcache::Pool::builder()
    /// .connection_timeout(std::time::Duration::from_secs(1))
    /// .build(memcache::ConnectionManager::new("memcache://localhost:12345").unwrap())
    /// .unwrap();
    /// let client = memcache::Client::with_pool(pool);
    /// client.flush_with_delay(10).unwrap();
    /// ```
    pub fn flush_with_delay(&self, delay: u32) -> Result<(), MemcacheError> {
        self.get_connection()?.flush_with_delay(delay)
    }

    /// Get a key from memcached server.
    ///
    /// Example:
    ///
    /// ```rust
    /// let pool = memcache::Pool::builder()
    /// .connection_timeout(std::time::Duration::from_secs(1))
    /// .build(memcache::ConnectionManager::new("memcache://localhost:12345").unwrap())
    /// .unwrap();
    /// let client = memcache::Client::with_pool(pool);
    /// let _: Option<String> = client.get("foo").unwrap();
    /// ```
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, MemcacheError> {
        check_key_len(key)?;
        self.get_connection()?.get(key)
    }

    /// Set a key with associate value into memcached server with expiration seconds.
    ///
    /// Example:
    ///
    /// ```rust
    /// let pool = memcache::Pool::builder()
    /// .connection_timeout(std::time::Duration::from_secs(1))
    /// .build(memcache::ConnectionManager::new("memcache://localhost:12345").unwrap())
    /// .unwrap();
    /// let client = memcache::Client::with_pool(pool);
    /// client.set("foo", "bar", 10).unwrap();
    /// # client.flush().unwrap();
    /// ```
    pub fn set<V: Serialize>(&self, key: &str, value: V, expiration: u32) -> Result<(), MemcacheError> {
        check_key_len(key)?;
        self.get_connection()?.set(key, value, expiration)
    }

    /// Compare and swap a key with the associate value into memcached server with expiration seconds.
    /// `cas_id` should be obtained from a previous `gets` call.
    ///
    /// Example:
    ///
    /// ```rust
    /// use std::collections::HashMap;
    /// let pool = memcache::Pool::builder()
    /// .connection_timeout(std::time::Duration::from_secs(1))
    /// .build(memcache::ConnectionManager::new("memcache://localhost:12345").unwrap())
    /// .unwrap();
    /// let client = memcache::Client::with_pool(pool);
    /// client.set("foo", "bar", 10).unwrap();
    /// // TODO: fix CAS & gets
    /// // let result: HashMap<String, (Vec<u8>, u32, Option<u64>)> = client.gets(&["foo"]).unwrap();
    /// // let (_, _, cas) = result.get("foo").unwrap();
    /// // let cas = cas.unwrap();
    /// // assert_eq!(true, client.cas("foo", "bar2", 10, cas).unwrap());
    /// # client.flush().unwrap();
    /// ```
    pub fn cas<V: Serialize>(&self, key: &str, value: V, expiration: u32, cas_id: u64) -> Result<bool, MemcacheError> {
        check_key_len(key)?;
        self.get_connection()?.cas(key, value, expiration, cas_id)
    }

    /// Add a key with associate value into memcached server with expiration seconds.
    ///
    /// Example:
    ///
    /// ```rust
    /// let pool = memcache::Pool::builder()
    /// .connection_timeout(std::time::Duration::from_secs(1))
    /// .build(memcache::ConnectionManager::new("memcache://localhost:12345").unwrap())
    /// .unwrap();
    /// let client = memcache::Client::with_pool(pool);
    /// let key = "add_test";
    /// client.delete(key).unwrap();
    /// client.add(key, "bar", 100000000).unwrap();
    /// # client.flush().unwrap();
    /// ```
    pub fn add<V: Serialize>(&self, key: &str, value: V, expiration: u32) -> Result<(), MemcacheError> {
        check_key_len(key)?;
        self.get_connection()?.add(key, value, expiration)
    }

    /// Replace a key with associate value into memcached server with expiration seconds.
    ///
    /// Example:
    ///
    /// ```rust
    /// let pool = memcache::Pool::builder()
    /// .connection_timeout(std::time::Duration::from_secs(1))
    /// .build(memcache::ConnectionManager::new("memcache://localhost:12345").unwrap())
    /// .unwrap();
    /// let client = memcache::Client::with_pool(pool);
    /// let key = "replace_test";
    /// client.set(key, "bar", 0).unwrap();
    /// client.replace(key, "baz", 100000000).unwrap();
    /// # client.flush().unwrap();
    /// ```
    pub fn replace<V: Serialize>(&self, key: &str, value: V, expiration: u32) -> Result<(), MemcacheError> {
        check_key_len(key)?;
        self.get_connection()?.replace(key, value, expiration)
    }

    /// Delete a key from memcached server.
    ///
    /// Example:
    ///
    /// ```rust
    /// let pool = memcache::Pool::builder()
    /// .connection_timeout(std::time::Duration::from_secs(1))
    /// .build(memcache::ConnectionManager::new("memcache://localhost:12345").unwrap())
    /// .unwrap();
    /// let client = memcache::Client::with_pool(pool);
    /// client.delete("foo").unwrap();
    /// # client.flush().unwrap();
    /// ```
    pub fn delete(&self, key: &str) -> Result<bool, MemcacheError> {
        check_key_len(key)?;
        self.get_connection()?.delete(key)
    }

    /// Set a new expiration time for a exist key.
    ///
    /// Example:
    ///
    /// ```rust
    /// let pool = memcache::Pool::builder()
    /// .connection_timeout(std::time::Duration::from_secs(1))
    /// .build(memcache::ConnectionManager::new("memcache://localhost:12345").unwrap())
    /// .unwrap();
    /// let client = memcache::Client::with_pool(pool);
    /// assert_eq!(client.touch("not_exists_key", 12345).unwrap(), false);
    /// client.set("foo", "bar", 123).unwrap();
    /// assert_eq!(client.touch("foo", 12345).unwrap(), true);
    /// # client.flush().unwrap();
    /// ```
    pub fn touch(&self, key: &str, expiration: u32) -> Result<bool, MemcacheError> {
        check_key_len(key)?;
        self.get_connection()?.touch(key, expiration)
    }

    /// Get all servers' statistics.
    ///
    /// Example:
    /// ```rust
    /// let pool = memcache::Pool::builder()
    /// .connection_timeout(std::time::Duration::from_secs(1))
    /// .build(memcache::ConnectionManager::new("memcache://localhost:12345").unwrap())
    /// .unwrap();
    /// let client = memcache::Client::with_pool(pool);
    /// let stats = client.stats().unwrap();
    /// ```
    pub fn stats(&self) -> Result<Stats, MemcacheError> {
        self.get_connection()?.stats()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn connect(target: &str) -> Result<Client, MemcacheError> {
        let pool = r2d2::Pool::builder()
            .max_size(20)
            .connection_timeout(Duration::from_millis(500))
            .build(ConnectionManager::new(target)?)?;

        Ok(Client::with_pool(pool))
    }

    #[cfg(unix)]
    #[test]
    fn unix() {
        let client = connect("memcache:///tmp/memcached.sock").unwrap();
        assert!(client.version().unwrap() != "");
    }

    #[cfg(feature = "tls")]
    #[test]
    fn ssl_noverify() {
        let client = connect("memcache+tls://localhost:12350?verify_mode=none").unwrap();
        assert!(client.version().unwrap() != "");
    }

    #[cfg(feature = "tls")]
    #[test]
    fn ssl_verify() {
        let client =
            connect("memcache+tls://localhost:12350?ca_path=tests/assets/RUST_MEMCACHE_TEST_CERT.crt").unwrap();
        assert!(client.version().unwrap() != "");
    }

    #[cfg(feature = "tls")]
    #[test]
    fn ssl_client_certs() {
        let client = connect("memcache+tls://localhost:12351?key_path=tests/assets/client.key&cert_path=tests/assets/client.crt&ca_path=tests/assets/RUST_MEMCACHE_TEST_CERT.crt").unwrap();
        assert!(client.version().unwrap() != "");
    }

    #[test]
    fn delete() {
        let client = connect("memcache://localhost:12345").unwrap();
        client.set("an_exists_key", "value", 0).unwrap();
        assert_eq!(client.delete("an_exists_key").unwrap(), true);
        assert_eq!(client.delete("a_not_exists_key").unwrap(), false);
    }
}
