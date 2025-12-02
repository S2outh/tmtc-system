#![feature(const_trait_impl)]
use telemetry_system::*;

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


#[telemetry_definition]
mod telemetry {
    #[tmv(u32, id = 12, address = "first_value")]
    struct FirstTMValue;
    #[tmv(crate::TestValue, id = 1)]
    struct SecondTMValue;
    mod some_other_mod {
        #[tmv(crate::TestVector, id = 3)]
        struct ThirdTMValue;
    }
}

beacon!(TestBeacon, telemetry, FirstTMValue, SecondTMValue, some_other_mod::ThirdTMValue);

#[test]
fn beacon_creation() {
    let beacon = TestBeacon::new();

    let sizes = [4, (4), (2 + 4 + 4)];
    assert_eq!(beacon.bytes().len(), sizes.iter().sum());
    assert_eq!(TestBeacon::SIZES, sizes);
}

#[test]
fn beacon_insertion() {
    let mut beacon = TestBeacon::new();

    let first_value = 1234u32;
    let second_value = TestValue { val: 3 };
    let third_value = TestVector { x: 3, y: 3.3, z: TestValue { val: 1 }};
    beacon.insert(&telemetry::FirstTMValue, &first_value).unwrap();
    beacon.insert(&telemetry::SecondTMValue, &second_value).unwrap();
    beacon.insert(&telemetry::some_other_mod::ThirdTMValue, &third_value).unwrap();
    
    assert_eq!(&beacon.bytes()[0..4], first_value.to_le_bytes());
    assert_eq!(&beacon.bytes()[4..8], second_value.val.to_le_bytes());
    assert_eq!(&beacon.bytes()[8..10], third_value.x.to_le_bytes());
    assert_eq!(&beacon.bytes()[10..14], third_value.y.to_le_bytes());
    assert_eq!(&beacon.bytes()[14..18], third_value.z.val.to_le_bytes());
}

#[test]
fn beacon_insertion_id() {
    let mut id_beacon = TestBeacon::new();
    let mut beacon = TestBeacon::new();

    let first_value = 1234u32;
    let second_value = TestValue { val: 3 };
    let third_value = TestVector { x: 3, y: 3.3, z: TestValue { val: 1 }};

    id_beacon.insert(telemetry::from_id(12), &first_value).unwrap();
    id_beacon.insert(telemetry::from_id(1), &second_value).unwrap();
    id_beacon.insert(telemetry::from_id(3), &third_value).unwrap();

    beacon.insert(&telemetry::FirstTMValue, &first_value).unwrap();
    beacon.insert(&telemetry::SecondTMValue, &second_value).unwrap();
    beacon.insert(&telemetry::some_other_mod::ThirdTMValue, &third_value).unwrap();

    assert_eq!(id_beacon.bytes(), beacon.bytes());
}

#[test]
fn beacon_insertion_address() {
    let mut address_beacon = TestBeacon::new();
    let mut beacon = TestBeacon::new();

    let first_value = 1234u32;
    let second_value = TestValue { val: 3 };
    let third_value = TestVector { x: 3, y: 3.3, z: TestValue { val: 1 }};

    address_beacon.insert(telemetry::from_address("telemetry.first_value"), &first_value).unwrap();
    address_beacon.insert(telemetry::from_address("telemetry.second_tm_value"), &second_value).unwrap();
    address_beacon.insert(telemetry::from_address("telemetry.some_other_mod.third_tm_value"), &third_value).unwrap();

    beacon.insert(&telemetry::FirstTMValue, &first_value).unwrap();
    beacon.insert(&telemetry::SecondTMValue, &second_value).unwrap();
    beacon.insert(&telemetry::some_other_mod::ThirdTMValue, &third_value).unwrap();

    assert_eq!(address_beacon.bytes(), beacon.bytes());
}
