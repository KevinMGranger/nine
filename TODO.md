# TODO

In decreasing order of importance:

- [ ] make it work with Bytes
  - If I'm going stateless, how will `in_stat` work?
    - Twstat / Rstat serialize_struct passes a serializer with in_stat = true
    - StructSerializer passes that along with any child calls
- [ ] how do we leave room for optimizations?
  -  for example, alternative versions of Rread or Twrite that can use preexisting BytesMuts
  -  if we have 1:1 message type to implementation that won't work
  -  do we just have alternate versions of the protocol?
  -  how could we eventually do async sendfile?
- [x] simplify strings
- [x] switch to thiserror to simplify error handling (stopgap for custom)
- [ ] figure out how/if to do datetime stuff
- [x] ~~switch away from error handling libs in favor of a custom implementation
    (or reconsider. is reimplimenting thiserrror worth it for the sake of fewer deps?)~~
    yeah, just sticking with thiserror
  

Undecided importance:

- [ ] add types for 9p2000.u
- [ ] add types for 9p2000.l
- [ ] find a way to test example in README?
- [ ] show reading a response in README?
- [ ] do the tagged/core idea from [IDEAS](IDEAS.md)?