use super::ChellValue;
use super::ChellValueError;

// Empty type
impl ChellValue for () {
    const MAX_BYTE_SIZE: usize = 0;
    fn read(_bytes: &[u8]) -> Result<(usize, Self), ChellValueError>
    where
        Self: Sized,
    {
        Ok((0, ()))
    }
    fn write(&self, _mem: &mut [u8]) -> Result<usize, ChellValueError> {
        Ok(0)
    }
}

// Arrays
impl<const N: usize, T: ChellValue> ChellValue for [T; N] {
    const MAX_BYTE_SIZE: usize = N * T::MAX_BYTE_SIZE;
    fn read(bytes: &[u8]) -> Result<(usize, Self), ChellValueError> {
        let mut pos = 0;
        let arr = core::array::try_from_fn(|_| {
            if pos >= bytes.len() {
                return Err(ChellValueError::OutOfMemory);
            }
            let (len, value) = T::read(&bytes[pos..])?;
            pos += len;
            Ok(value)
        })?;
        Ok((pos, arr))
    }
    fn write(&self, mem: &mut [u8]) -> Result<usize, ChellValueError> {
        let mut pos = 0;
        for i in 0..N {
            if pos >= mem.len() {
                return Err(ChellValueError::OutOfMemory);
            }
            pos += self[i].write(&mut mem[pos..])?;
        }
        Ok(pos)
    }
}

// Options
impl<T: ChellValue> ChellValue for Option<T> {
    const MAX_BYTE_SIZE: usize = 1 + T::MAX_BYTE_SIZE;
    fn read(bytes: &[u8]) -> Result<(usize, Self), ChellValueError> {
        let mut pos = 1;
        let Some(enum_byte) = bytes.get(0) else {
            return Err(ChellValueError::OutOfMemory);
        };
        match enum_byte {
            0u8 => Ok((pos, None)),
            1u8 => {
                let (len, value) = T::read(&bytes[pos..])?;
                pos += len;
                Ok((pos, Some(value)))
            }
            _ => Err(ChellValueError::BadEnumVariant),
        }
    }
    fn write(&self, mem: &mut [u8]) -> Result<usize, ChellValueError> {
        let mut pos = 1;
        if mem.len() < 1 {
            return Err(ChellValueError::OutOfMemory);
        }
        match self {
            None => {
                mem[0] = 0u8;
            }
            Some(v0) => {
                mem[0] = 1u8;
                pos += v0.write(&mut mem[pos..])?;
            }
        }
        Ok(pos)
    }
}

// Results
impl<T, E> ChellValue for Result<T, E>
where
    T: ChellValue,
    E: ChellValue,
{
    const MAX_BYTE_SIZE: usize = {
        let mut m = 0;
        let variant_size = 1usize + T::MAX_BYTE_SIZE;
        if variant_size > m {
            m = variant_size;
        }
        let variant_size = 1usize + E::MAX_BYTE_SIZE;
        if variant_size > m {
            m = variant_size;
        }
        m
    };
    fn read(bytes: &[u8]) -> Result<(usize, Self), ChellValueError> {
        let mut pos = 1;
        let value = match bytes.first().ok_or(ChellValueError::OutOfMemory)? {
            0u8 => Self::Ok({
                let (len, value) = T::read(&bytes[pos..])?;
                pos += len;
                value
            }),
            1u8 => Self::Err({
                let (len, value) = E::read(&bytes[pos..])?;
                pos += len;
                value
            }),
            _ => return Err(ChellValueError::BadEnumVariant),
        };
        Ok((pos, value))
    }
    fn write(&self, mem: &mut [u8]) -> Result<usize, ChellValueError> {
        let mut pos = 1;
        match self {
            Self::Ok(v0) => {
                *(mem.first_mut().ok_or(ChellValueError::OutOfMemory)?) = 0u8;
                pos += v0.write(&mut mem[pos..])?;
            }
            Self::Err(v0) => {
                *(mem.first_mut().ok_or(ChellValueError::OutOfMemory)?) = 1u8;
                pos += v0.write(&mut mem[pos..])?;
            }
        }
        Ok(pos)
    }
}
