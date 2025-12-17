#![no_std]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(const_trait_impl)]

mod telemetry_value;

// macro reexports
pub use macros::TMValue;
pub use macros::beacon;
pub use macros::telemetry_definition;

pub use telemetry_value::TMValue;
pub use telemetry_value::DynTMValue;

pub const trait DynTelemetryDefinition {
    fn id(&self) -> u16;
    fn address(&self) -> &str;
}
/// Reexports that should only be used by the macro generated code
pub mod internal {
    use crate::TMValue;
    pub trait TelemetryDefinition: crate::DynTelemetryDefinition {
        type TMValueType: crate::TMValue;
        const BYTE_SIZE: usize = Self::TMValueType::BYTE_SIZE;
        const ID: u16;
    }
}

#[derive(Debug)]
pub struct BoundsError;

#[derive(Debug)]
pub enum ParseError {
    WrongId,
    TooShort,
    BadLayout,
    ValueParseError,
}

pub trait DynBeacon {
    fn insert_slice(&mut self, telemetry_definition: &dyn DynTelemetryDefinition, bytes: &[u8]) -> Result<(), BoundsError>;
    fn get_slice<'a>(&'a mut self, telemetry_definition: &dyn DynTelemetryDefinition) -> Result<&'a [u8], BoundsError>;
    fn from_bytes(&mut self, bytes: &[u8]) -> Result<(), ParseError>;
    fn bytes(&mut self) -> &[u8];
    fn flush(&mut self);
}
