# nine &emsp; [![Latest Version]][crates.io] [![Rustc Version 1.34.2]][rustc] [![Build Status]][travis_ci]

[Latest Version]: https://img.shields.io/crates/v/nine.svg
[crates.io]: https://crates.io/crates/nine
[Rustc Version 1.34.2]: https://img.shields.io/badge/rustc-1.34.2-lightgray.svg
[rustc]: https://blog.rust-lang.org/2019/05/14/Rust-1.34.2.html
[travis_ci]: https://travis-ci.com/KevinMGranger/nine

The 9p protocol as a serde format and message types.

This crate contains structs representing the various types of messages in the
9p2000 protocol (with other versions coming soon), as well as a serializer and
deserializer for the wire format for those messages.

There is _not_ an included server implementation or abstraction.

The purpose of this design is to allow for easy extensibility and experimentation
with the protocol.

# Stability

This library is in its early stages of development, and thus may have large,
backwards-incompatible changes that occur. User discretion is adviced.

# Usage example

To connect to a 9p server and start version negotiation:

```rust
use std::io::prelude::*;
use std::net::TcpStream;
use nine::ser::*;
use nine::de::*;
use nine::p2000::*;

let connection = TcpStream::connect("127.0.0.1").unwrap();
let version = Tversion { tag: NOTAG, msize: u32::max_value(), version: "9p2000".into() };
let serialized_message: Vec<u8> = into_bytes(&version).unwrap();
connection.write_all(&serialized_message).unwrap();
```

# Client Binary

`nine` can also be used as a simple one-shot 9p client, a-la plan9port's `9p` command.

Currently only non-authed explicitly-attached read to a unix socket is implemented.

## Examples

```bash
# set up a server to listen at /tmp/9ptest, then:
$ nine -a /tmp/9ptest read /foo
bar
```

# Special Thanks

Casey Rodarmor for collaborating on the initial design.

The countless others in the rust community that have answered my questions.