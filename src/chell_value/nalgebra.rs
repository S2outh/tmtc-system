use super::ChellValue;
use super::ChellValueError;

use nalgebra::ArrayStorage;
use nalgebra::Dim;
use nalgebra::Matrix;
use nalgebra::RawStorage;

macro_rules! newtype_value {
    ($field: tt, $inner: ty, $constructor: path, $($interface: tt)*) => {
        $($interface)* {
            const MAX_BYTE_SIZE: usize = <$inner>::MAX_BYTE_SIZE;

            fn read(bytes: &[u8]) -> Result<(usize, Self), ChellValueError>
            where
                Self: Sized,
            {
                let (len, value) = <$inner>::read(bytes)?;
                Ok((len, $constructor(value)))
            }
            fn write(&self, mem: &mut [u8]) -> Result<usize, ChellValueError> {
                self.$field.write(mem)
            }
        }
    };
}

newtype_value!(0, [[T; R]; C], Self,
    impl<T, const R: usize, const C: usize> ChellValue for ArrayStorage<T, R, C>
        where T: ChellValue,
);

newtype_value!(data, S, Self::from_data,
    impl<T, R, C, S> ChellValue for Matrix<T, R, C, S>
        where
            S: ChellValue,
            R: Dim,
            C: Dim,
            S: RawStorage<T, R, C>
);
