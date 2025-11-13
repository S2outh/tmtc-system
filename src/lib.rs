#![no_std]
mod telemetry_value;

use core::marker::PhantomData;

pub use macros::beacon;
pub use macros::TMValue;

pub use telemetry_value::TMValue;

pub trait BeaconDefinition {
    fn transfer_cell(&self, storage: &mut [u8]);
}

pub struct Beacon<DEF: BeaconDefinition, const N: usize> {
    storage: [u8; N],
    _def: PhantomData<DEF>,
}
impl<DEF: BeaconDefinition, const N: usize> Beacon<DEF, N> {
    pub fn new() -> Self {
        Self {
            storage: [0u8; N],
            _def: PhantomData::<DEF>,
        }
    }
    pub fn insert(&mut self, topic: DEF) {
        topic.transfer_cell(&mut self.storage);
    }
    pub fn bytes(&self) -> &[u8] {
        &self.storage
    }
    pub fn flush(&mut self) {
        self.storage.fill(0);
    }
}

