/*!
rust-memcache is a [memcached](https://memcached.org/) client written in pure rust.

# Install:

The crate is called `memcache` and you can depend on it via cargo:

```ini
[dependencies]
memcache = "*"
```

# Features:

- <input type="checkbox"  disabled checked /> All memcached supported protocols
  - <input type="checkbox"  disabled checked /> Binary protocol
  - <input type="checkbox"  disabled checked /> ASCII protocol
- <input type="checkbox"  disabled checked /> All memcached supported connections
  - <input type="checkbox"  disabled checked /> TCP connection
  - <input type="checkbox"  disabled checked /> UDP connection
  - <input type="checkbox"  disabled checked/> UNIX Domain socket connection
  - <input type="checkbox"  disabled checked/> TLS connection
- <input type="checkbox"  disabled /> Encodings
  - <input type="checkbox"  disabled checked /> Typed interface
  - <input type="checkbox"  disabled /> Automatically compress
  - <input type="checkbox"  disabled /> Automatically serialize to JSON / msgpack etc
- <input type="checkbox"  disabled checked /> Mutiple server support with custom key hash algorithm
- <input type="checkbox"  disabled checked /> Authority
  - <input type="checkbox"  disabled checked /> Binary protocol (plain SASL authority)
  - <input type="checkbox"  disabled checked /> ASCII protocol

# Basic usage:

```rust
// create connection with to memcached server node:
let pool = memcache::Pool::builder()
  .connection_timeout(std::time::Duration::from_secs(1))
  .build(memcache::ConnectionManager::new("memcache://127.0.0.1:12345?timeout=10&tcp_nodelay=true").unwrap())
  .unwrap();

let client = memcache::Client::with_pool(pool);

// flush the database:
client.flush().unwrap();

// set a string value:
client.set("foo", "bar", 0).unwrap();

// retrieve from memcached:
let value: Option<String> = client.get("foo").unwrap();
assert_eq!(value, Some(String::from("bar")));
assert_eq!(value.unwrap(), "bar");

// prepend, append:
client.prepend("foo", "foo").unwrap();
client.append("foo", "baz").unwrap();
let value: String = client.get("foo").unwrap().unwrap();
assert_eq!(value, "foobarbaz");

// delete value:
client.delete("foo").unwrap();

// using counter:
client.set("counter", 40, 0).unwrap();
let answer: i32 = client.get("counter").unwrap().unwrap();
assert_eq!(answer, 42);
```
!*/
#![deny(
    bad_style,
    const_err,
    dead_code,
    deprecated,
    improper_ctypes,
    missing_debug_implementations,
    missing_docs,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_in_public,
    trivial_casts,
    trivial_numeric_casts,
    unconditional_recursion,
    unknown_lints,
    unreachable_code,
    unreachable_pub,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_extern_crates,
    unused_import_braces,
    unused_mut,
    unused_parens,
    unused_qualifications,
    unused_results,
    warnings,
    while_true
)]

mod client;
mod codec;
mod connection;
mod error;
mod protocol;
mod stream;

pub use crate::client::{Client, Stats};
pub use crate::connection::ConnectionManager;
pub use crate::error::{ClientError, CommandError, MemcacheError, ServerError};
pub use r2d2::Error as PoolError;

/// R2D2 connection pool
pub type Pool = r2d2::Pool<ConnectionManager>;
