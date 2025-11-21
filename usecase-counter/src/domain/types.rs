use bytes::{Buf, BufMut};
use commonware_codec::{EncodeSize, Read, ReadExt, Write};

#[derive(Clone, Debug, PartialEq)]
pub struct CounterTaskData {
    pub var1: String,
    pub var2: String,
    pub var3: String,
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
        let v1 = {
            let len = u32::read(buf)? as usize;
            let mut bytes = vec![0u8; len];
            buf.copy_to_slice(&mut bytes);
            String::from_utf8(bytes)
                .map_err(|_| commonware_codec::Error::Invalid("var1", "utf8"))?
        };
        let v2 = {
            let len = u32::read(buf)? as usize;
            let mut bytes = vec![0u8; len];
            buf.copy_to_slice(&mut bytes);
            String::from_utf8(bytes)
                .map_err(|_| commonware_codec::Error::Invalid("var2", "utf8"))?
        };
        let v3 = {
            let len = u32::read(buf)? as usize;
            let mut bytes = vec![0u8; len];
            buf.copy_to_slice(&mut bytes);
            String::from_utf8(bytes)
                .map_err(|_| commonware_codec::Error::Invalid("var3", "utf8"))?
        };
        Ok(Self {
            var1: v1,
            var2: v2,
            var3: v3,
        })
    }
}

impl EncodeSize for CounterTaskData {
    fn encode_size(&self) -> usize {
        std::mem::size_of::<u32>() * 3 + self.var1.len() + self.var2.len() + self.var3.len()
    }
}
