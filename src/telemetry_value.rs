
pub trait TMValue {
    type Bytes: Sized;
    fn from_bytes(bytes: Self::Bytes) -> Self where Self: Sized;
    fn to_bytes(&self) -> Self::Bytes;
}

macro_rules! primitive_value {
    ($type:ident) => {
        impl TMValue for $type {
            type Bytes = [u8; size_of::<$type>()];
            fn from_bytes(bytes: Self::Bytes) -> Self {
                Self::from_le_bytes(bytes)
            }
            fn to_bytes(&self) -> Self::Bytes {
                self.to_le_bytes()
            }
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
