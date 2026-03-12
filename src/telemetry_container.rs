use crate::{TMValue, TelemetryDefinition};

#[macro_export]
macro_rules! fd_compat_telemetry_container {
    ($($def:tt)+) => {
        $crate::TelemetryContainer<{
            match $crate::ceil_to_fd_compat($($def)+ :: MAX_BYTE_SIZE) {
                Ok(v) => v,
                Err(_) => panic!("Max byte size too big for Fd frame")
            }
        }>
    };
}

pub const fn ceil_to_fd_compat(len: usize) -> Result<usize, UnsupportedValue> {
    const FD_LEN_OPTS: [usize; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 12, 16, 20, 24, 32, 48, 64];

    let mut i = 0;
    while i < FD_LEN_OPTS.len() {
        if FD_LEN_OPTS[i] >= len {
            return Ok(FD_LEN_OPTS[i]);
        }
        i += 1;
    }
    Err(UnsupportedValue)
}

#[derive(Debug)]
pub struct UnsupportedValue;

/// This is a generic wrapper to hold TMvalues as bytes for transfer via fdcan
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
    pub fn fd_bytes(&self) -> &[u8] {
        let frame_size = ceil_to_fd_compat(self.len).expect("type to big for fd can frame");
        if frame_size > N {
            panic!("Container not compatible with fd byte sizes")
        }
        &self.storage[..frame_size]
    }
}
