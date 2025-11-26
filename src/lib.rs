#![no_std]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(const_trait_impl)]

mod telemetry_value;

pub use macros::TMValue;
pub use macros::beacon;
pub use macros::telemetry_definition;

pub use telemetry_value::TMValue;
pub use telemetry_value::DynTMValue;

pub const trait DynTelemetryDefinition {
    fn id(&self) -> u32;
    fn address(&self) -> &str;
}
pub trait TelemetryDefinition: DynTelemetryDefinition {
    type TMValueType: TMValue;
    const BYTE_SIZE: usize = Self::TMValueType::BYTE_SIZE;
}
