# TODO

In decreasing order of importance:

- [ ] add far more testing, especially for edge cases (mainly re: lengths and numeric conversions)
- [ ] make a proper SerializationFail type
- [ ] audit all numeric casts, especially unwrapping overflows and signedness conversion
- [ ] replace the `unimplemented!()` spots with proper errors in both ser and de
- [ ] do the tagged/core idea from [IDEA](IDEAS.md)
- [ ] make a version of `into_bytes()` that takes an existing writer
- [ ] consider making a more detailed DeserializeError type to handle the other ways deserialize impls can complain. Although, this is questionably useful-- when will the deserializer be used with types other than the messages we define?
- [ ] consider having `serialize_seq()` handle sequences of unknown lengths, or at least handle the error better
- [ ] follow serde style guide for ser and de membership / naming

Undecided importance:

- [ ] add types for 9p2000.u
- [ ] add types for 9p2000.l
