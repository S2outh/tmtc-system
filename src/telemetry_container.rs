use crate::{TMValue, TelemetryDefinition};

#[macro_export]
macro_rules! telemetry_container {
    ($($def:tt)+) => {
        TelemetryContainer<{ $($def)+ :: MAX_BYTE_SIZE }>
    };
}

#[derive(Debug)]
pub struct UnsupportedValue;

pub struct TelemetryContainer<const N: usize> {
    id: u16,
    storage: [u8; N],
    len: usize,
}
impl<const N: usize> TelemetryContainer<N> {
    pub fn new(
        definition: &dyn TelemetryDefinition,
        value: &impl TMValue,
    ) -> Result<Self, UnsupportedValue> {
        let mut storage = [0u8; N];
        let len = value.write(&mut storage).map_err(|_| UnsupportedValue)?;
        Ok(Self {
            id: definition.id(),
            storage,
            len,
        })
    }
    pub fn id(&self) -> u16 {
        self.id
    }
    pub fn bytes(&self) -> &[u8] {
        &self.storage[..self.len]
    }
}
