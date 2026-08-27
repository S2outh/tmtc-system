use super::ChellValue;
use super::ChellValueError;
use heapless::Vec;

// Vectors
impl<const N: usize, T: ChellValue> ChellValue for Vec<T, N> {
    const MAX_BYTE_SIZE: usize = N * T::MAX_BYTE_SIZE;
    fn read(bytes: &[u8]) -> Result<(usize, Self), ChellValueError> {
        let (mut pos, len) = u8::read(bytes)?;
        let mut vec = Vec::new();
        for _ in 0..len {
            let (len, value) = T::read(&bytes[pos..])?;
            let _ = vec.push(value);
            pos += len;
        }
        Ok((pos, vec))
    }
    fn write(&self, mem: &mut [u8]) -> Result<usize, ChellValueError> {
        let mut pos = (self.len() as u8).write(mem)?;
        for i in 0..self.len() {
            pos += self[i].write(&mut mem[pos..])?;
        }
        Ok(pos)
    }
}
