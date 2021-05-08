# nine-memfs

A simple in-memory 9p filesystem.

Currently only works over unix sockets, is single threaded, and probably* buggy.

\* definitely buggy


# TODO

- handle multiplexing connections & attach properly
  - (don't do connections, force 9pserve to mux it)
  - handle multiple sessions viewing the same file tree
- exclusive open
- permissions audit
- numeric / size correctness audit

# Edge cases to test

## Dir bytes

1. have a dir with some amount of children
2. open the dir for reading, read some of the bytes
3. invalidate the bytes somehow (write to child changing length, wstat child, etc.)
4. read the dir again from where you left off