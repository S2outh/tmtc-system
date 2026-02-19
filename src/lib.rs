#![no_std]
#![feature(const_trait_impl)]

#[cfg(feature = "ground")]
extern crate alloc;

mod bitfield;
mod telemetry_container;
mod telemetry_value;

// macro reexports
pub use macros::TMValue;
pub use macros::beacon;
pub use macros::telemetry_definition;

// value reexports
pub use telemetry_value::TMValue;
pub use telemetry_value::TMValueError;

// container reexports
pub use telemetry_container::TelemetryContainer;
pub use telemetry_container::UnsupportedValue;

pub const trait TelemetryDefinition {
    fn id(&self) -> u16;
    fn address(&self) -> &str;
}

#[cfg(feature = "ground")]
pub use crate::telemetry_value::ground_tm;
/// Reexports that should only be used by the macro generated code
pub mod _internal {
    use crate::TMValue;
    pub use crate::bitfield::Bitfield;
    #[cfg(feature = "ground")]
    pub use crate::ground_tm::*;
    pub const trait InternalTelemetryDefinition: crate::TelemetryDefinition {
        type TMValueType: crate::TMValue;
        const BYTE_SIZE: usize = Self::TMValueType::BYTE_SIZE;
        const ID: u16;
    }
}

// Error types
#[derive(Debug)]
pub struct NotFoundError;

#[derive(Debug)]
pub enum BeaconOperationError {
    DefNotInBeacon,
    OutOfMemory,
}

#[derive(Debug)]
pub enum ParseError {
    WrongId,
    BadCRC,
    OutOfMemory,
}

// Dynamic beacon trait
pub trait Beacon {
    type Timestamp;
    fn insert_slice(
        &mut self,
        telemetry_definition: &dyn TelemetryDefinition,
        bytes: &[u8],
    ) -> Result<(), BeaconOperationError>;
    fn from_bytes(
        &mut self,
        bytes: &[u8],
        crc_func: &mut dyn FnMut(&[u8]) -> u16,
    ) -> Result<(), ParseError>;
    fn to_bytes(&mut self, crc_func: &mut dyn FnMut(&[u8]) -> u16) -> &[u8];
    fn set_timestamp(&mut self, timestamp: Self::Timestamp);
    fn flush(&mut self);
    fn name(&self) -> &'static str;
    fn id(&self) -> u8;
}
