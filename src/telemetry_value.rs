
// # Trait definitions
pub trait DynTMValue {
    fn read(&mut self, bytes: &[u8]) -> usize;
    fn write(&self, mem: &mut [u8]) -> usize;
}

pub trait TMValue: DynTMValue {
    const BYTE_SIZE: usize;
    fn from_bytes(bytes: [u8; Self::BYTE_SIZE]) -> Self where Self: Default {
        let mut value: Self = Self::default();
        Self::read(&mut value, &bytes);
        value
    }
    fn to_bytes(&self) -> [u8; Self::BYTE_SIZE] {
        let mut bytes = [0u8; Self::BYTE_SIZE];
        self.write(&mut bytes);
        bytes
    }
}

// # Primitives
macro_rules! primitive_value {
    ($type:ident) => {
        impl DynTMValue for $type {
            fn read(&mut self, bytes: &[u8]) -> usize {
                *self = Self::from_le_bytes(bytes[..Self::BYTE_SIZE].try_into().expect("wrong memory provided"));
                Self::BYTE_SIZE
            }
            fn write(&self, mem: &mut [u8]) -> usize {
                let bytes = self.to_le_bytes();
                mem[..Self::BYTE_SIZE].copy_from_slice(&bytes);
                Self::BYTE_SIZE
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

primitive_value!(i8);
primitive_value!(i16);
primitive_value!(i32);
primitive_value!(i64);
primitive_value!(i128);

primitive_value!(f32);
primitive_value!(f64);

// # Arrays
impl<const N: usize, T: TMValue> DynTMValue for [T; N] {
    fn read(&mut self, bytes: &[u8]) -> usize {
        let mut pos = 0;
        for i in 0..N {
            pos += self[i].read(&bytes[pos..])
        }
        pos
    }
    fn write(&self, mem: &mut [u8]) -> usize {
        let mut pos = 0;
        for i in 0..N {
            pos += self[i].write(&mut mem[pos..])
        }
        pos
    }
}
impl<const N: usize, T: TMValue> TMValue for [T; N] {
    const BYTE_SIZE: usize = N * T::BYTE_SIZE;
}
