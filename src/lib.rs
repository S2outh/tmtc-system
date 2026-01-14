#![no_std]
#![feature(const_trait_impl)]

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

pub use telemetry_container::BeaconContainer;

pub const trait TelemetryDefinition {
    fn id(&self) -> u16;
    fn address(&self) -> &str;
}
/// Reexports that should only be used by the macro generated code
pub mod internal {
    use crate::TMValue;
    pub const trait InternalTelemetryDefinition: crate::TelemetryDefinition {
        type TMValueType: crate::TMValue;
        const BYTE_SIZE: usize = Self::TMValueType::BYTE_SIZE;
        const ID: u16;
    }

    #[cfg_attr(feature = "serde", derive(serde::Serialize))]
    #[cfg(feature = "serde")]
    pub struct NatsTelemetry<T: serde::Serialize> {
        timestamp: i64,
        value: T,
    }
    #[cfg(feature = "serde")]
    impl<T: serde::Serialize> NatsTelemetry<T> {
        pub fn new(timestamp: i64, value: T) -> Self {
            Self { timestamp, value }
        }
    }
}

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
