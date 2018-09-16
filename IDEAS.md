# Future Ideas

## Progressive Type Enhancement

Server implementers may wish for their users to not have to deal with tags. After all, there's essentially only one correct thing to do, so why not have
the server handle it? This means users would only have to return the main part
of the response.

```rust 
mod core_types {
    pub struct Tversion {
        pub msize: u32,
        pub version: String
    }

    pub struct Rversion {
        pub msize: u32,
        pub version: String
    }
}
mod tagged_types {
    pub struct Tversion {
        pub tag: u32,
        #[serde(flatten)]
        pub version: super::core_types::Tversion,
    }
    pub struct Rversion {
        pub tag: u32,
        #[serde(flatten)]
        pub version: super::core_types::Rversion,
    }
}

// then a server could have

pub mod prelude {
    pub use super::tagged_types::{Tversion, Tother}; // etc
    pub use super::core_types::{Rversion, Rother}; // etc
}

// then the method could be

fn version(&mut self, msg: Tversion) -> Result<Rversion>;

// and thus the user has the tag if they want it, but don't have to return it.
```

## Ahead of time lengths

We have to do length computation at serialize time because we can't specialize for structs (unless we manually implemented Serialize, which would defeat half of the point of this library, and make the types strange for other formats to deal with). This would be easier with specialization.

But in the end, what does this get us? The ability to serialize a message without using a buffer-- but wouldn't we always want to use a buffer anyway?

## Protocol extension

There's plenty of types that don't exist in 9p-- floats, bools, maps, etc. For existing versions of the protocol, we can just ignore that. But part of the purpose of this library is allowing experimentation.

Maybe we can make a trait that requires you to expose what the regular (de)serializer needs, but allows you to implement the (de)serialize methods yourself.

## References

How do we properly handle the strings within the message types?

Right now, they're all `Cow<'static, str>` which makes it easy for development.
However, I don't think it should stay that way in the long run.

We could make each message have as many lifetimes as there are strings in itself, and Cow each string.
Or is that too much trouble and they should all just have `String`s?

If we do go with the "one lifetime per string approach", we can always offer a convenience:

```rust
type CowStr<'a> = Cow<'a, str>;
pub struct Stat<'n, 'u, 'g, 'm> {
    pub type_: u16,
    pub dev: u32,
    pub qid: Qid,
    pub mode: FileMode,
    pub atime: u32,
    pub mtime: u32,
    pub length: u64,
    pub name: CowStr<'n>,
    pub uid: CowStr<'u>,
    pub gid: CowStr<'g>,
    pub muid: CowStr<'m>,
}

pub mod owned {
    pub type Stat = super::Stat<'static, 'static, 'static, 'static>;
}
```