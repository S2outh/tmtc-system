use crate::ChellDefinition;
pub mod core_lib;
pub mod primitives;

#[cfg(feature = "heapless")]
pub mod heapless;

#[cfg(feature = "nalgebra")]
pub mod nalgebra;

#[derive(Debug)]
pub enum ChellValueError {
    OutOfMemory,
    BadEnumVariant,
}

// Trait definitions
pub trait ChellValue {
    const MAX_BYTE_SIZE: usize;
    fn read(bytes: &[u8]) -> Result<(usize, Self), ChellValueError>
    where
        Self: Sized;
    fn write(&self, mem: &mut [u8]) -> Result<usize, ChellValueError>;
}

pub trait ParsableChellValue<DEF>: ChellValue
where
    DEF: ChellDefinition,
{
    type Parser;
    fn parser(self, _def: DEF) -> Self::Parser;
}

#[cfg(feature = "ground")]
pub mod ground {
    use crate::{ChellDefinition, ChellValueError};
    use serde::ser::SerializeStruct;
    pub trait SerializableChellValue<DEF>: super::ChellValue + serde::Serialize
    where
        DEF: ChellDefinition,
    {
        fn serialize_ground(
            self,
            _def: DEF,
            timestamp: &dyn erased_serde::Serialize,
            serializer: &dyn Fn(
                &dyn erased_serde::Serialize,
            ) -> Result<alloc::vec::Vec<u8>, erased_serde::Error>,
        ) -> Result<alloc::vec::Vec<(&'static str, alloc::vec::Vec<u8>)>, erased_serde::Error>;
    }
    pub struct GroundTelemetry<'a> {
        timestamp: &'a dyn erased_serde::Serialize,
        value: &'a dyn erased_serde::Serialize,
    }
    impl<'a> GroundTelemetry<'a> {
        pub fn new(
            timestamp: &'a dyn erased_serde::Serialize,
            value: &'a dyn erased_serde::Serialize,
        ) -> Self {
            Self { timestamp, value }
        }
    }
    impl<'a> serde::Serialize for GroundTelemetry<'a> {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            let mut s = serializer.serialize_struct("GroundTelemetry", 2)?;
            s.serialize_field("timestamp", self.timestamp)?;
            s.serialize_field("value", self.value)?;
            s.end()
        }
    }
    #[derive(Debug)]
    pub enum ReserializeError {
        ChellValueError(ChellValueError),
        SerdeError(erased_serde::Error),
    }
}
