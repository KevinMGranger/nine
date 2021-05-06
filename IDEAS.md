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

    // or heck, even
    pub struct Tagged<T> {
        pub tag: u32,
        #[serde(flatten)]
        pub message: T
    }

    pub type Tattach = Tagged<super::core_types::Tattach>
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

~~We have to do length computation at serialize time because we can't specialize for structs (unless we manually implemented Serialize, which would defeat half of the point of this library, and make the types strange for other formats to deal with). This would be easier with specialization.~~

~~But in the end, what does this get us? The ability to serialize a message without using a buffer-- but wouldn't we always want to use a buffer anyway?~~

We can make a _separate serializer_ that just does length computation!

## Protocol extension

There's plenty of types that don't exist in 9p-- floats, bools, maps, etc. For existing versions of the protocol, we can just ignore that. But part of the purpose of this library is allowing experimentation.

~~Maybe we can make a trait that requires you to expose what the regular (de)serializer needs, but allows you to implement the (de)serialize methods yourself.~~

I forgot about deserialize_with.

## References

~~How do we properly handle the strings within the message types?~~

~~Right now, they're all `Cow<'static, str>` which makes it easy for development.~~
~~However, I don't think it should stay that way in the long run.~~

~~Each message could/should be generic over the string type.~~
~~I'd have to see how having that "hidden lifetime" works with the deserializer, though.~~

~~And I'd need to look at how the serde magic works with generic stuff. Maybe I'd need to write my own functions for it.~~

This is far more cleverness than it's worth. Just make them `String`s.