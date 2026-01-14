#![feature(const_trait_impl)]
#![feature(const_cmp)]
use tmtc_system::*;

#[derive(TMValue, Default, Clone, Copy)]
pub struct TestValue {
    val: u32,
}

#[derive(TMValue, Default, Clone, Copy)]
pub struct TestVector {
    x: i16,
    y: f32,
    z: TestValue,
}

#[telemetry_definition(id = 0)]
mod telemetry {
    #[tmv(u32)]
    struct FirstTMValue;
    #[tmv(crate::TestVector)]
    struct SecondTMValue;
    #[tmm(id = 100)]
    mod some_other_mod {
        #[tmv(u64)]
        struct ThirdTMValue;
        #[tmv(i32)]
        struct FourthTMValue;
        #[tmv(crate::TestValue)]
        struct FifthTMValue;
    }
}

type PartialTestContainer = telemetry_container!(telemetry::some_other_mod);
type FullTestContainer = telemetry_container!(telemetry);

#[test]
fn partial_container_creation() {
    assert_eq!(telemetry::some_other_mod::MAX_BYTE_SIZE, 8);

    let container =
        PartialTestContainer::new(&telemetry::some_other_mod::FourthTMValue, &42).unwrap();
    assert_eq!(container.id(), 101);

    assert_eq!(container.bytes().len(), 4);
    assert_eq!(container.bytes()[0..4], 42i32.to_le_bytes());
}

#[test]
fn full_container_creation() {
    assert_eq!(telemetry::MAX_BYTE_SIZE, 10);

    let container = FullTestContainer::new(
        &telemetry::SecondTMValue,
        &TestVector {
            x: 12,
            y: 24.,
            z: TestValue { val: 36 },
        },
    )
    .unwrap();
    assert_eq!(container.id(), 1);

    assert_eq!(container.bytes().len(), 10);
    assert_eq!(container.bytes()[0..2], 12i16.to_le_bytes());
    assert_eq!(container.bytes()[2..6], 24f32.to_le_bytes());
    assert_eq!(container.bytes()[6..10], 36u32.to_le_bytes());
}
