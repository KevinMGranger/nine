# 9pfuse specific, possibly
- ls doesn't show new files under 9pfuse but does under 9p standalone command
- can't cat > to a new file in 9pfuse (and had a panic when unmounting after testing this?)
- 9pfuse fails if a walk to a single element Rerrors, even though that's what the spec says to do