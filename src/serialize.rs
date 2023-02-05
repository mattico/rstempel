use byteorder::{ReadBytesExt, WriteBytesExt, BE};
use std::io;

pub trait JavaSerialize {
    fn serialize<W: io::Write>(&self, writer: &mut DataOutput<W>) -> io::Result<()>;
}

pub trait JavaDeserialize: Sized {
    fn deserialize<R: io::Read>(reader: &mut DataInput<R>) -> io::Result<Self>;
}

/// Reads binary data in a manner compatible with
/// [Java's DataInput class](https://docs.oracle.com/javase/7/docs/api/java/io/DataInput.html).
pub struct DataInput<R: io::Read> {
    inner: R,
}

impl<R: io::Read> DataInput<R> {
    pub fn new(reader: R) -> Self {
        Self { inner: reader }
    }

    pub fn read<T: JavaDeserialize>(&mut self) -> io::Result<T> {
        T::deserialize(self)
    }

    /// Like Java's `readBoolean`.
    pub fn read_bool(&mut self) -> io::Result<bool> {
        Ok(self.inner.read_u8()? != 0)
    }

    /// Like Java's `readInt` but casts the result to `u32`, returning [`std::io::ErrorKind::InvalidData`] if the value
    /// is negative.
    pub fn read_u32(&mut self) -> io::Result<u32> {
        self.inner
            .read_i32::<BE>()?
            .try_into()
            .map_err(|_| io::Error::from(io::ErrorKind::InvalidData))
    }

    /// Like Java's `readInt`
    pub fn read_i32(&mut self) -> io::Result<i32> {
        self.inner.read_i32::<BE>()
    }

    /// Like Java's `readInt` but casts the result to `usize`, returning [`std::io::ErrorKind::InvalidData`] if the value
    /// is negative.
    pub fn read_usize(&mut self) -> io::Result<usize> {
        Ok(self.read_u32()? as usize)
    }

    /// Like Java's `readInt` but casts the result to `u32`, returning [`Option::None`] if the value is negative.
    pub fn read_u32_opt(&mut self) -> io::Result<Option<u32>> {
        Ok(self.inner.read_i32::<BE>()?.try_into().ok())
    }

    /// Like Java's `readChar`. Reads a UTF-16 code unit and converts it to a rust [`char`], returning
    /// [`std::io::ErrorKind::InvalidData`] if the single UTF-16 code unit is not a valid UTF-16 code point.
    pub fn read_char(&mut self) -> io::Result<char> {
        let utf16_char = self.inner.read_u16::<BE>()?;
        char::decode_utf16(std::iter::once(utf16_char))
            .next()
            .unwrap()
            .map_err(|_| io::Error::from(io::ErrorKind::InvalidData))
    }

    /// Like Java's `readUTF`. Reads a modified UTF-8 string with length. Returns [`std::io::ErrorKind::InvalidData`]
    /// if the value is not a valid UTF-8 string.
    pub fn read_string(&mut self) -> io::Result<String> {
        let len = self.inner.read_u16::<BE>()? as usize;
        if len == 0 {
            return Ok(String::new());
        }
        let mut buf = vec![0u8; len];
        self.inner.read_exact(&mut buf)?;
        let str = cesu8::from_java_cesu8(&buf)
            .map_err(|_| io::Error::from(io::ErrorKind::InvalidData))?;
        Ok(str.into_owned())
    }
}

/// Writes binary data in a manner compatible with
/// [Java's DataOutput class](https://docs.oracle.com/javase/7/docs/api/java/io/DataOutput.html).
pub struct DataOutput<R: io::Write> {
    pub inner: R,
}

impl<R: io::Write> DataOutput<R> {
    pub fn new(writer: R) -> Self {
        Self { inner: writer }
    }

    pub fn write<T: JavaSerialize>(&mut self, data: &T) -> io::Result<()> {
        data.serialize(self)
    }

    /// Like Java's `writeBoolean`.
    pub fn write_bool(&mut self, x: bool) -> io::Result<()> {
        self.inner.write_u8(x as u8)
    }

    /// Like Java's `writeInt`
    pub fn write_i32(&mut self, x: i32) -> io::Result<()> {
        self.inner.write_i32::<BE>(x)
    }

    /// Like Java's `writeInt`, returning [`std::io::ErrorKind::InvalidData`] if the value
    /// is negative.
    pub fn write_u32(&mut self, x: u32) -> io::Result<()> {
        self.inner.write_i32::<BE>(
            x.try_into()
                .map_err(|_| io::Error::from(io::ErrorKind::InvalidData))?,
        )
    }

    /// Like Java's `writeInt`, returning [`std::io::ErrorKind::InvalidData`] if the value
    /// is negative.
    pub fn write_usize(&mut self, x: usize) -> io::Result<()> {
        self.inner.write_i32::<BE>(
            x.try_into()
                .map_err(|_| io::Error::from(io::ErrorKind::InvalidData))?,
        )
    }

    /// Like Java's `writeInt` but casts the result to `u32`, returning [`Option::None`] if the value is negative.
    pub fn write_u32_opt(&mut self, x: Option<u32>) -> io::Result<()> {
        let x = match x {
            Some(x) => x
                .try_into()
                .map_err(|_| io::Error::from(io::ErrorKind::InvalidData))?,
            None => -1,
        };
        self.inner.write_i32::<BE>(x)?;
        Ok(())
    }

    /// Like Java's `writeChar`. Writes a single UTF-16 code unit from a [`char`], returning
    /// [`std::io::ErrorKind::InvalidData`] if the [`char`] cannot be encoded as a single UTF-16 code unit.
    pub fn write_char(&mut self, x: char) -> io::Result<()> {
        let mut buf = [0u16; 2];
        let buf = x.encode_utf16(&mut buf);
        if buf.len() != 1 {
            return Err(io::ErrorKind::InvalidData.into());
        }
        self.inner.write_u16::<BE>(buf[0])
    }

    /// Like Java's `writeUTF`. Writes a modified UTF-8 string with length. Returns [`std::io::ErrorKind::InvalidData`]
    /// if the string is longer than `u16::MAX`.
    pub fn write_string(&mut self, x: &str) -> io::Result<()> {
        let len: u16 = x
            .len()
            .try_into()
            .map_err(|_| io::Error::from(io::ErrorKind::InvalidData))?;
        self.inner.write_u16::<BE>(len)?;
        if len > 0 {
            let x = cesu8::to_java_cesu8(x);
            self.inner.write_all(&x)?;
        }
        Ok(())
    }
}
