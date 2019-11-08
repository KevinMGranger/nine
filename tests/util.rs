use std::io::{Result, Seek, SeekFrom, Write};

pub struct BlackHoleWriter;

impl Write for BlackHoleWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

impl Seek for BlackHoleWriter {
    fn seek(&mut self, _from: SeekFrom) -> Result<u64> {
        Ok(0)
    }
}
