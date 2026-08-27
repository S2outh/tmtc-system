use super::ChellValue;
use super::ChellValueError;

macro_rules! primitive_value {
    ($type:ident) => {
        impl ChellValue for $type {
            const MAX_BYTE_SIZE: usize = size_of::<Self>();
            fn read(bytes: &[u8]) -> Result<(usize, Self), ChellValueError> {
                if bytes.len() < Self::MAX_BYTE_SIZE {
                    return Err(ChellValueError::OutOfMemory);
                }
                let value = Self::from_le_bytes(bytes[..Self::MAX_BYTE_SIZE].try_into().unwrap());
                Ok((Self::MAX_BYTE_SIZE, value))
            }
            fn write(&self, mem: &mut [u8]) -> Result<usize, ChellValueError> {
                if mem.len() < Self::MAX_BYTE_SIZE {
                    return Err(ChellValueError::OutOfMemory);
                }
                let bytes = self.to_le_bytes();
                mem[..Self::MAX_BYTE_SIZE].copy_from_slice(&bytes);
                Ok(Self::MAX_BYTE_SIZE)
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
