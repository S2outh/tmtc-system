#![feature(const_trait_impl)]
use tmtc_system::*;

#[derive(TMValue, Default, Clone, Copy)]
pub struct TestValue {
    val: u32
}

#[derive(TMValue, Default, Clone, Copy)]
pub struct TestVector {
    x: i16,
    y: f32,
    z: TestValue
}


#[telemetry_definition(id = 0)]
mod telemetry {
    #[tmv(u32, address = "first_value")]
    struct FirstTMValue;
    #[tmv(crate::TestValue)]
    struct SecondTMValue;
    #[tmm(id = 100)]
    mod some_other_mod {
        #[tmv(crate::TestVector)]
        struct ThirdTMValue;
    }
}

beacon!(TestBeacon, crate::telemetry, id = 0, values(FirstTMValue, SecondTMValue, some_other_mod::ThirdTMValue));

use test_beacon::TestBeacon;

#[test]
fn beacon_creation() {
    let mut beacon = TestBeacon::new();

    let sizes = [3, 4, (4), (2 + 4 + 4)];
    assert_eq!(beacon.bytes().len(), sizes.iter().sum());
}

#[test]
fn beacon_insertion() {
    let mut beacon = TestBeacon::new();

    let first_value = 1234u32;
    let second_value = TestValue { val: 3 };
    let third_value = TestVector { x: 3, y: 3.3, z: TestValue { val: 1 }};
    beacon.first_tm_value = first_value;
    beacon.second_tm_value = second_value;
    beacon.third_tm_value = third_value;
    
    assert_eq!(&beacon.bytes()[0..3], [0, 0, 0]);
    assert_eq!(&beacon.bytes()[3..7], first_value.to_le_bytes());
    assert_eq!(&beacon.bytes()[7..11], second_value.val.to_le_bytes());
    assert_eq!(&beacon.bytes()[11..13], third_value.x.to_le_bytes());
    assert_eq!(&beacon.bytes()[13..17], third_value.y.to_le_bytes());
    assert_eq!(&beacon.bytes()[17..21], third_value.z.val.to_le_bytes());
}

#[test]
fn beacon_insertion_id() {
    let mut id_beacon = TestBeacon::new();
    let mut beacon = TestBeacon::new();

    let first_value = 1234u32;
    let second_value = TestValue { val: 3 };
    let third_value = TestVector { x: 3, y: 3.3, z: TestValue { val: 1 }};

    id_beacon.insert_slice(telemetry::from_id(0), &first_value.to_bytes()).unwrap();
    id_beacon.insert_slice(telemetry::from_id(1), &second_value.to_bytes()).unwrap();
    id_beacon.insert_slice(telemetry::from_id(100), &third_value.to_bytes()).unwrap();

    beacon.first_tm_value = first_value;
    beacon.second_tm_value = second_value;
    beacon.third_tm_value = third_value;

    assert_eq!(id_beacon.bytes(), beacon.bytes());
}

#[test]
fn beacon_insertion_address() {
    let mut address_beacon = TestBeacon::new();
    let mut beacon = TestBeacon::new();

    let first_value = 1234u32;
    let second_value = TestValue { val: 3 };
    let third_value = TestVector { x: 3, y: 3.3, z: TestValue { val: 1 }};

    address_beacon.insert_slice(telemetry::from_address("telemetry.first_value"), &first_value.to_bytes()).unwrap();
    address_beacon.insert_slice(telemetry::from_address("telemetry.second_tm_value"), &second_value.to_bytes()).unwrap();
    address_beacon.insert_slice(telemetry::from_address("telemetry.some_other_mod.third_tm_value"), &third_value.to_bytes()).unwrap();

    beacon.first_tm_value = first_value;
    beacon.second_tm_value = second_value;
    beacon.third_tm_value = third_value;

    assert_eq!(address_beacon.bytes(), beacon.bytes());
}
