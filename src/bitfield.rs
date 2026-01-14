pub struct Bitfield<const N: usize> {
    storage: [u8; N],
}
impl<const N: usize> Bitfield<N> {
    pub fn new() -> Self {
        Self { storage: [0u8; N] }
    }
    pub fn new_from_bytes(storage: [u8; N]) -> Self {
        Self { storage }
    }
    pub fn bytes(&self) -> &[u8] {
        &self.storage
    }
    pub fn set(&mut self, index: usize) {
        let byte = index / 8;
        let bit = index % 8;
        self.storage[byte] |= 1 << bit;
    }
    pub fn get(&self, index: usize) -> bool {
        let byte = index / 8;
        let bit = index % 8;
        (self.storage[byte] >> bit) & 1 == 1
    }
}
