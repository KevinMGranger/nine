# nine &emsp; [![Latest Version]][crates.io] [![Rustc Version 1.29+]][rustc]

[Latest Version]: https://img.shields.io/crates/v/nine.svg
[crates.io]: https://crates.io/crates/nine
[Rustc Version 1.29+]: https://img.shields.io/badge/rustc-1.29+-lightgray.svg
[rustc]: https://blog.rust-lang.org/2018/09/13/Rust-1.29.html

The 9p protocol as a serde format and message types.

This crate contains structs representing the various types of messages in the
9p2000 protocol (with other versions coming soon), as well as a serializer and
deserializer for the wire format for those messages.

There is _not_ an included server implementation, because
[network protocols should be kept separate from IO](https://sans-io.readthedocs.io/).

The purpose of this design is to allow for easy extensibility and experimentation
with the protocol.

# Special Thanks

Casey Rodarmor for collaborating on the initial design.

The countless others in the rust community that have answered my questions.