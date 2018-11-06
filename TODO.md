# TODO

In decreasing order of importance:

- [ ] figure out how/if to do datetime stuff
- [ ] add far more testing, especially for edge cases (mainly re: lengths and numeric conversions)
- [ ] audit all numeric casts, especially unwrapping overflows and signedness conversion
- [ ] replace the `unimplemented!()` spots with proper errors in both ser and de
- [ ] do the tagged/core idea from [IDEA](IDEAS.md)
- [ ] follow serde style guide for ser and de membership / naming

Undecided importance:

- [ ] add types for 9p2000.u
- [ ] add types for 9p2000.l
