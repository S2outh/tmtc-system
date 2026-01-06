use heapless::Vec;


#[derive(Debug)]
pub enum TMValueError{
    OutOfMemory,
    BadEnumVariant
}

// # Trait definitions
pub trait DynTMValue {
    fn read(&mut self, bytes: &[u8]) -> Result<usize, TMValueError>;
    fn write(&self, mem: &mut [u8]) -> Result<usize, TMValueError>;
}

pub trait TMValue: DynTMValue {
    const BYTE_SIZE: usize;
    fn from_bytes(bytes: [u8; Self::BYTE_SIZE]) -> Self where Self: Sized {
        unsafe {
            let mut value: Self = core::mem::zeroed();
            Self::read(&mut value, &bytes).unwrap();
            value
        }
    }
    fn to_bytes(&self) -> [u8; Self::BYTE_SIZE] {
        let mut bytes = [0u8; Self::BYTE_SIZE];
        self.write(&mut bytes).unwrap();
        bytes
    }
}

// # Primitives
macro_rules! primitive_value {
    ($type:ident) => {
        impl DynTMValue for $type {
            fn read(&mut self, bytes: &[u8]) -> Result<usize, TMValueError> {
                if bytes.len() < Self::BYTE_SIZE {
                    return Err(TMValueError::OutOfMemory);
                }
                *self = Self::from_le_bytes(bytes[..Self::BYTE_SIZE].try_into().unwrap());
                Ok(Self::BYTE_SIZE)
            }
            fn write(&self, mem: &mut [u8]) -> Result<usize, TMValueError> {
                if mem.len() < Self::BYTE_SIZE {
                    return Err(TMValueError::OutOfMemory)
                }
                let bytes = self.to_le_bytes();
                mem[..Self::BYTE_SIZE].copy_from_slice(&bytes);
                Ok(Self::BYTE_SIZE)
            }
        }
        impl TMValue for $type {
            const BYTE_SIZE: usize = size_of::<Self>();
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
impl<const N: usize, T: TMValue> DynTMValue for [T; N] {
    fn read(&mut self, bytes: &[u8]) -> Result<usize, TMValueError> {
        let mut pos = 0;
        for i in 0..N {
            pos += self[i].read(&bytes[pos..])?;
        }
        Ok(pos)
    }
    fn write(&self, mem: &mut [u8]) -> Result<usize, TMValueError> {
        let mut pos = 0;
        for i in 0..N {
            pos += self[i].write(&mut mem[pos..])?;
        }
        Ok(pos)
    }
}
impl<const N: usize, T: TMValue> TMValue for [T; N] {
    const BYTE_SIZE: usize = N * T::BYTE_SIZE;
}
// # Vectors
impl<const N: usize, T: TMValue> DynTMValue for Vec<T, N> {
    fn read(&mut self, bytes: &[u8]) -> Result<usize, TMValueError> {
        let mut len = 0;
        let mut pos = len.read(bytes)?;
        for i in 0..len {
            unsafe {
                let _ = self.push(core::mem::zeroed());
            }
            pos += self[i].read(&bytes[pos..])?;
        }
        Ok(pos)
    }
    fn write(&self, mem: &mut [u8]) -> Result<usize, TMValueError> {
        let mut pos = 0;
        for i in 0..self.len() {
            pos += self[i].write(&mut mem[pos..])?;
        }
        Ok(pos)
    }
}
impl<const N: usize, T: TMValue> TMValue for Vec<T, N> {
    const BYTE_SIZE: usize = N * T::BYTE_SIZE;
}

// # Options
impl<T: TMValue> DynTMValue for Option<T> {
    fn read(&mut self, bytes: &[u8]) -> Result<usize, TMValueError> {
        let mut pos = 1;
        match bytes[0] {
            0u8 => {
                *self = None;
            }
            1u8 => unsafe {
                let mut v0: T = core::mem::zeroed();
                pos += v0.read(&bytes[pos..])?;
                *self = Some(v0);
            },
            _ => return Err(TMValueError::BadEnumVariant),
        }
        Ok(pos)
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
impl<T: TMValue> TMValue for Option<T> {
    const BYTE_SIZE: usize = 1 + T::BYTE_SIZE;
}
