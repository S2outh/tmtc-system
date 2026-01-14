#[derive(Debug)]
pub enum TMValueError {
    OutOfMemory,
    BadEnumVariant,
}

// # Trait definitions
pub trait TMValue {
    const BYTE_SIZE: usize;
    fn read(bytes: &[u8]) -> Result<(usize, Self), TMValueError>
    where
        Self: Sized;
    fn write(&self, mem: &mut [u8]) -> Result<usize, TMValueError>;
}

// # Primitives
macro_rules! primitive_value {
    ($type:ident) => {
        impl TMValue for $type {
            const BYTE_SIZE: usize = size_of::<Self>();
            fn read(bytes: &[u8]) -> Result<(usize, Self), TMValueError> {
                if bytes.len() < Self::BYTE_SIZE {
                    return Err(TMValueError::OutOfMemory);
                }
                let value = Self::from_le_bytes(bytes[..Self::BYTE_SIZE].try_into().unwrap());
                Ok((Self::BYTE_SIZE, value))
            }
            fn write(&self, mem: &mut [u8]) -> Result<usize, TMValueError> {
                if mem.len() < Self::BYTE_SIZE {
                    return Err(TMValueError::OutOfMemory);
                }
                let bytes = self.to_le_bytes();
                mem[..Self::BYTE_SIZE].copy_from_slice(&bytes);
                Ok(Self::BYTE_SIZE)
            }
        }
    };
}

primitive_value!(u8);
primitive_value!(u16);
primitive_value!(u32);
primitive_value!(u64);
primitive_value!(u128);
primitive_value!(usize);

primitive_value!(i8);
primitive_value!(i16);
primitive_value!(i32);
primitive_value!(i64);
primitive_value!(i128);
primitive_value!(isize);

primitive_value!(f32);
primitive_value!(f64);

// # Arrays
impl<const N: usize, T: TMValue> TMValue for [T; N] {
    const BYTE_SIZE: usize = N * T::BYTE_SIZE;
    fn read(bytes: &[u8]) -> Result<(usize, Self), TMValueError> {
        unsafe {
            let mut pos = 0;
            let mut arr: Self = core::mem::zeroed();
            for i in 0..N {
                let (len, value) = T::read(&bytes[pos..])?;
                pos += len;
                arr[i] = value;
            }
            Ok((pos, arr))
        }
    }
    fn write(&self, mem: &mut [u8]) -> Result<usize, TMValueError> {
        let mut pos = 0;
        for i in 0..N {
            pos += self[i].write(&mut mem[pos..])?;
        }
        Ok(pos)
    }
}
// # Vectors
// use heapless::Vec;
// impl<const N: usize, T: TMValue> TMValue for Vec<T, N> {
//     const BYTE_SIZE: usize = N * T::BYTE_SIZE;
//     fn read(bytes: &[u8]) -> Result<(usize, Self), TMValueError> {
//         let (mut pos, len) = u8::read(bytes)?;
//         let mut vec = Vec::new();
//         for _ in 0..len {
//             let (len, value) = T::read(&bytes[pos..])?;
//             vec.push(value);
//             pos += len;
//         }
//         Ok((pos, vec))
//     }
//     fn write(&self, mem: &mut [u8]) -> Result<usize, TMValueError> {
//         let mut pos = (self.len() as u8).write(mem)?;
//         for i in 0..self.len() {
//             pos += self[i].write(&mut mem[pos..])?;
//         }
//         Ok(pos)
//     }
// }

// # Options
impl<T: TMValue> TMValue for Option<T> {
    const BYTE_SIZE: usize = 1 + T::BYTE_SIZE;
    fn read(bytes: &[u8]) -> Result<(usize, Self), TMValueError> {
        let mut pos = 1;
        match bytes[0] {
            0u8 => Ok((pos, None)),
            1u8 => {
                let (len, value) = T::read(&bytes[pos..])?;
                pos += len;
                Ok((pos, Some(value)))
            }
            _ => Err(TMValueError::BadEnumVariant),
        }
    }
    fn write(&self, mem: &mut [u8]) -> Result<usize, TMValueError> {
        let mut pos = 1;
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
