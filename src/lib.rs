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

pub type InsrResult = Result<(), BoundsError>;
pub type ExtrResult<'a> = Result<&'a [u8], BoundsError>;

pub trait DynBeacon {
    fn get_bounds(&self, telemetry_definition: &dyn DynTelemetryDefinition) -> Result<(usize, usize), BoundsError>;
    fn insert_slice(&mut self, telemetry_definition: &dyn DynTelemetryDefinition, data: &[u8]) -> InsrResult;
    fn insert(&mut self, telemetry_definition: &dyn DynTelemetryDefinition, value: &dyn DynTMValue) -> InsrResult;
    fn get_slice<'a>(&'a self, telemetry_definition: &dyn DynTelemetryDefinition) -> ExtrResult<'a>;
    fn bytes(&self) -> &[u8];
    fn flush(&mut self);
}
