redis-cli
====
Redis CLI.

[![Crates version][version-image]][version-url]
[![Build Status][travis-image]][travis-url]
[![Coverage Status][coveralls-image]][coveralls-url]
[![Crates downloads][downloads-image]][downloads-url]

### Build

```sh
git clone https://github.com/iorust/redis-cli.git && cd redis-cli && cargo build --release
```

### Run

```sh
target/release/redis-cli -h 127.0.0.1 -p 6379
```

More help:
```sh
target/release/redis-cli --help
```

## Use as a crate

```rust
extern crate redis_cli;
// exports:
use redis_cli::{create_client, Client, COMMANDS, Value, encode_slice, Decoder};
```

#### Value, encode_slice, Decoder
Re-exports from the https://github.com/iorust/resp

#### `fn create_client(host: &str, port: u16, password: &str, db: u16) -> io::Result<Client>`
```Rust
let mut client = create_client("127.0.0.1", 6379, "", 0).expect("Failed to connect");
client.cmd(&["set", "test", "hello!"]).unwrap();
```

### Client
```Rust
struct Client {
    // some fields omitted
}
```

#### impl Client

##### `fn new<A: ToSocketAddrs>(addrs: A) -> Self`
```Rust
let mut client = Client::new((hostname, port));
```

##### `fn cmd(&mut self, slice: &[&str]) -> Result<Value>`
```Rust
client.cmd(&["sget", "test"]).unwrap(); // Value::String("hello!")
```

### COMMANDS
`Pub/Sub`, `monitor` are not available currently.

[version-image]: https://img.shields.io/crates/v/redis-cli.svg
[version-url]: https://crates.io/crates/redis-cli

[travis-image]: http://img.shields.io/travis/iorust/redis-cli.svg
[travis-url]: https://travis-ci.org/iorust/redis-cli

[coveralls-image]: https://coveralls.io/repos/github/iorust/redis-cli/badge.svg?branch=master
[coveralls-url]: https://coveralls.io/github/iorust/redis-cli?branch=master

[downloads-image]: https://img.shields.io/crates/d/redis-cli.svg
[downloads-url]: https://crates.io/crates/redis-cli
