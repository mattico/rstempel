use byteorder::{ReadBytesExt, BE};
use std::io;

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
