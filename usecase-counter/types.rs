use bytes::{Buf, BufMut};
use commonware_codec::{EncodeSize, Read, ReadExt, Write};

#[derive(Debug, Clone, PartialEq)]
pub struct CounterTaskData {
    pub var1: String,
    pub var2: String,
    pub var3: String,
}

impl Default for CounterTaskData {
    fn default() -> Self {
        Self {
            var1: "default_var1".to_string(),
            var2: "default_var2".to_string(),
            var3: "default_var3".to_string(),
        }
    }
}

impl Write for CounterTaskData {
    fn write(&self, buf: &mut impl BufMut) {
        (self.var1.len() as u32).write(buf);
        buf.put_slice(self.var1.as_bytes());
        (self.var2.len() as u32).write(buf);
        buf.put_slice(self.var2.as_bytes());
        (self.var3.len() as u32).write(buf);
        buf.put_slice(self.var3.as_bytes());
    }
}

impl Read for CounterTaskData {
    type Cfg = ();

    fn read_cfg(buf: &mut impl Buf, _: &()) -> Result<Self, commonware_codec::Error> {
        let var1_len = u32::read(buf)? as usize;
        if buf.remaining() < var1_len {
            return Err(commonware_codec::Error::EndOfBuffer);
        }
        let mut var1_bytes = vec![0u8; var1_len];
        buf.copy_to_slice(&mut var1_bytes);
        let var1 = String::from_utf8(var1_bytes)
            .map_err(|_| commonware_codec::Error::Invalid("var1", "decoding from utf8 failed"))?;

        let var2_len = u32::read(buf)? as usize;
        if buf.remaining() < var2_len {
            return Err(commonware_codec::Error::EndOfBuffer);
        }
        let mut var2_bytes = vec![0u8; var2_len];
        buf.copy_to_slice(&mut var2_bytes);
        let var2 = String::from_utf8(var2_bytes)
            .map_err(|_| commonware_codec::Error::Invalid("var2", "decoding from utf8 failed"))?;

        let var3_len = u32::read(buf)? as usize;
        if buf.remaining() < var3_len {
            return Err(commonware_codec::Error::EndOfBuffer);
        }
        let mut var3_bytes = vec![0u8; var3_len];
        buf.copy_to_slice(&mut var3_bytes);
        let var3 = String::from_utf8(var3_bytes)
            .map_err(|_| commonware_codec::Error::Invalid("var3", "decoding from utf8 failed"))?;

        Ok(Self { var1, var2, var3 })
    }
}

impl EncodeSize for CounterTaskData {
    fn encode_size(&self) -> usize {
        const LENGTH_PREFIX_SIZE: usize = std::mem::size_of::<u32>();
        LENGTH_PREFIX_SIZE
            + self.var1.len()
            + LENGTH_PREFIX_SIZE
            + self.var2.len()
            + LENGTH_PREFIX_SIZE
            + self.var3.len()
    }
}
